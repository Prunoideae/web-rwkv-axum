use std::sync::Arc;

use anyhow::{Error, Result};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use web_rwkv::{
    context::Context,
    model::{BackedState, Model, ModelState},
    tensor::shape::Shape,
};

use crate::helper::{Logits, State};

use super::infer::{InferContext, InferRequest, InferResult};

struct Slots {
    slots: Vec<Option<oneshot::Sender<InferResult>>>,
    batch_tokens: Vec<Vec<u16>>,
    batch_state_callbacks: Vec<Option<oneshot::Sender<Option<State>>>>,
    batch_state_ids: Vec<Option<String>>,
    batch_count: usize,
    batch: ModelState,
    model: Arc<Model<'static>>,
}

impl Slots {
    async fn new(batch_count: usize, context: &Context, model: Arc<Model<'static>>) -> Slots {
        Slots {
            slots: (0..batch_count).map(|_| None).collect(),
            batch_tokens: (0..batch_count).map(|_| Vec::new()).collect(),
            batch_state_callbacks: (0..batch_count).map(|_| None).collect(),
            batch_state_ids: vec![None; batch_count],
            batch: ModelState::new(&context, model.info(), batch_count),
            model,
            batch_count,
        }
    }

    fn is_full(&self) -> bool {
        self.slots.iter().all(|c| c.is_some())
    }

    fn is_clear(&self) -> bool {
        self.slots.iter().all(|c| c.is_none())
    }

    /// Swaps a state to a (potentially different) state in slot
    fn swap(
        &mut self,
        index: usize,
        state: Option<State>,
        state_id: String,
        state_callback: oneshot::Sender<Option<State>>,
    ) -> Result<()> {
        if let Some(slot_state_id) = &self.batch_state_ids[index] {
            // State id matches, no need to update state
            if *slot_state_id == state_id {
                let callback =
                    std::mem::replace(&mut self.batch_state_callbacks[index], Some(state_callback))
                        .unwrap();
                callback
                    .send(None)
                    .map_err(|_| Error::msg("Error when sending state!"))?;
                return Ok(());
            }
        }

        // Update the state since mismatch or empty slot
        self.batch_state_ids[index] = Some(state_id);
        if let Some(callback) =
            std::mem::replace(&mut self.batch_state_callbacks[index], Some(state_callback))
        {
            callback
                .send(Some(State(self.batch.back_batch(index)?.data)))
                .map_err(|_| Error::msg("Error when sending state!"))?;
        }
        let info = self.model.info();
        let state = if let Some(state) = state {
            let shape = Shape::new(info.num_emb, 5 * info.num_layers, 1);
            BackedState {
                shape,
                data: state.0,
            }
        } else {
            BackedState::new(info, 1)
        };
        self.batch.load_batch(&state, index)?;
        Ok(())
    }

    /// Inserts a request into the batch
    /// Also uploads the state to the buffer
    fn insert(&mut self, request: InferRequest) -> Result<()> {
        if self.is_full() {
            panic!("Batch is full while inserting new requests!")
        }

        let InferRequest {
            context: InferContext { state, tokens },
            callback,
            state_id,
            state_callback,
        } = request;
        let callback = Some(callback);

        // Try to find the empty slot with same slot id
        for idx in 0..self.batch_count {
            if self.slots[idx].is_none() {
                if let Some(slot_id) = &self.batch_state_ids[idx] {
                    if slot_id == &state_id {
                        self.slots[idx] = callback;
                        self.batch_tokens[idx] = tokens;
                        return self.swap(idx, state, state_id, state_callback);
                    }
                }
            }
        }

        // Try to find the empty slot with empty state id (not occupied)
        for idx in 0..self.batch_count {
            if self.slots[idx].is_none() && self.batch_state_ids[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                return self.swap(idx, state, state_id, state_callback);
            }
        }

        // Find the first index being empty
        for idx in 0..self.batch_count {
            if self.slots[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                return self.swap(idx, state, state_id, state_callback);
            }
        }
        Ok(())
    }

    /// Infer until any of the batch is completed.
    fn infer(&mut self) -> Result<()> {
        let logits = loop {
            let logits_internal = self
                .model
                .run(&mut self.batch_tokens, &self.batch)
                .expect("Failed to run infer!");
            if logits_internal.iter().any(|l| !l.is_empty()) {
                break logits_internal;
            }
        };

        for idx in 0..self.batch_count {
            if !logits[idx].is_empty() {
                let result = InferResult {
                    logits: Logits(logits[idx].clone()),
                };
                self.finish(idx, result)?
            }
        }
        Ok(())
    }

    /// Finishs a request by sending back the result.
    fn finish(&mut self, index: usize, result: InferResult) -> Result<()> {
        let channel = std::mem::replace(&mut self.slots[index], None)
            .ok_or(Error::msg("Called finish on empty channel!"))?;
        self.batch_tokens[index].clear();
        channel
            .send(result)
            .map_err(|_| Error::msg("Can't send result to channel!"))
    }
}

/// The pipeline.
/// Currently holds no data, might hold some data later.
pub struct Pipeline();

impl Pipeline {
    /// Load requests into the slot
    ///
    /// If the slot is full, load to queue instead (which will be loaded to slot when available)
    #[inline(always)]
    fn load_or_queue(
        requests: Vec<InferRequest>,
        slots: &mut Slots,
        queue: &mut Vec<InferRequest>,
    ) -> Result<()> {
        for request in requests {
            if slots.is_full() {
                queue.push(request);
            } else {
                slots.insert(request)?;
            }
        }
        Ok(())
    }

    /// Runs the pipeline, this should be used internally.
    ///
    /// Use `Pipeline::start` instead.
    pub async fn start(
        batch_size: usize,
        context: Context,
        model: Arc<Model<'static>>,
    ) -> (mpsc::Sender<Vec<InferRequest>>, JoinHandle<()>) {
        let (sender, mut receiver) = mpsc::channel::<Vec<InferRequest>>(batch_size);
        let handle = tokio::spawn(async move {
            let mut slots = Slots::new(batch_size, &context, model.clone()).await;
            let mut queued_requests: Vec<InferRequest> = Vec::new();
            loop {
                // When something arrives in the channel.
                // This has an assumption that the batch is empty. (just initialized/ finished all inference)
                if let Some(requests) = receiver.recv().await {
                    // Load the request
                    Pipeline::load_or_queue(requests, &mut slots, &mut queued_requests).unwrap();
                    // Start an infer loop until all slots are done again with no incoming requests
                    loop {
                        // Insert until slots full or no more requests
                        if !slots.is_full() {
                            while let Ok(requests) = receiver.try_recv() {
                                Pipeline::load_or_queue(requests, &mut slots, &mut queued_requests)
                                    .unwrap();
                                if slots.is_full() {
                                    break;
                                }
                            }
                        }
                        // Infer till at least 1 slot is done
                        slots.infer().unwrap();
                        // Release queued requests into the slots
                        while let Some(queued) = queued_requests.pop() {
                            slots.insert(queued).unwrap();
                            if slots.is_full() {
                                break;
                            }
                        }

                        // Check if any more requests are coming during the infer
                        // If yes, insert and continue
                        // If no, continue with current batches until all are clear
                        if !slots.is_full() {
                            while let Ok(requests) = receiver.try_recv() {
                                Pipeline::load_or_queue(requests, &mut slots, &mut queued_requests)
                                    .unwrap();
                                if slots.is_full() {
                                    break;
                                }
                            }
                            // If all clear, break the loop and await for next request to arrive.
                            if slots.is_clear() {
                                break;
                            }
                        }
                    }
                } else {
                    // Channels are all closed, exit the loop.
                    break;
                }
            }
            ()
        });
        (sender, handle)
    }
}
