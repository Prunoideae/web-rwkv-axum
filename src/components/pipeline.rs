use std::{sync::Arc, time::Duration};

use anyhow::{Error, Result};
use tokio::{
    sync::{
        mpsc::{self, error::TryRecvError},
        oneshot,
    },
    task::JoinHandle,
};
use web_rwkv::{
    context::Context,
    model::{BackedState, Model, ModelState},
    tensor::shape::Shape,
};

use crate::helper::{Logits, State};

use super::{
    infer::{InferContext, InferRequest, InferResult},
    permit::BatchRequest,
};

struct Slots {
    slots: Vec<Option<oneshot::Sender<InferResult>>>,
    batch_tokens: Vec<Vec<u16>>,
    batch_state_callbacks: Vec<Option<oneshot::Sender<Option<State>>>>,
    batch_state_ids: Vec<Option<String>>,
    batch_request: BatchRequest,
    batch_count: usize,
    batch: ModelState,
    model: Arc<Model<'static>>,
}

impl Slots {
    async fn new(
        batch_count: usize,
        context: &Context,
        model: Arc<Model<'static>>,
        batch_request: BatchRequest,
    ) -> Slots {
        Slots {
            slots: (0..batch_count).map(|_| None).collect(),
            batch_tokens: (0..batch_count).map(|_| Vec::new()).collect(),
            batch_state_callbacks: (0..batch_count).map(|_| None).collect(),
            batch_state_ids: vec![None; batch_count],
            batch: ModelState::new(&context, model.info(), batch_count),
            model,
            batch_count,
            batch_request,
        }
    }

    #[inline(always)]
    fn get_requests_count(&self) -> usize {
        self.slots.iter().filter(|x| x.is_some()).count()
    }

    #[inline(always)]
    /// Can the slots start infer or not
    ///
    /// The infer will be started if requested slots are full
    /// or the slots are full
    fn can_start_infer(&self) -> bool {
        self.is_full() || self.batch_request.get() <= self.get_requests_count()
    }

    fn is_full(&self) -> bool {
        self.slots.iter().all(|c| c.is_some())
    }

    #[allow(dead_code)]
    fn is_clear(&self) -> bool {
        self.slots.iter().all(|c| c.is_none())
    }

    /// Load requests into the slot
    ///
    /// If the slot is full, load to queue instead (which will be loaded to slot when available)
    #[inline(always)]
    fn load_or_queue(
        &mut self,
        requests: Vec<InferRequest>,
        queue: &mut Vec<InferRequest>,
    ) -> Result<()> {
        for request in requests {
            if self.is_full() {
                queue.push(request);
            } else {
                self.insert(request)?;
            }
        }
        Ok(())
    }

    /// Swaps a state to a (potentially different) state in slot
    fn swap(
        &mut self,
        index: usize,
        state: Option<State>,
        state_id: Option<String>,
        state_callback: Option<oneshot::Sender<Option<State>>>,
    ) -> Result<()> {
        if &self.batch_state_ids[index] == &state_id {
            // State id matches, no need to update state
            let callback =
                std::mem::replace(&mut self.batch_state_callbacks[index], state_callback).unwrap();
            callback
                .send(None)
                .map_err(|_| Error::msg("Error when sending state!"))?;
            return Ok(());
        }

        // Update the state since mismatch or empty slot
        self.batch_state_ids[index] = state_id;
        if let Some(callback) =
            std::mem::replace(&mut self.batch_state_callbacks[index], state_callback)
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

        // Try to reuse the slot
        for idx in 0..self.batch_count {
            if self.slots[idx].is_none() {
                if let Some(slot_id) = &self.batch_state_ids[idx] {
                    if slot_id == &state_id {
                        self.slots[idx] = callback;
                        self.batch_tokens[idx] = tokens;
                        return self.swap(idx, state, Some(state_id), Some(state_callback));
                    }
                }
            }
        }

        // Try to find the empty slot with empty state id (not occupied)
        for idx in 0..self.batch_count {
            if self.slots[idx].is_none() && self.batch_state_ids[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                return self.swap(idx, state, Some(state_id), Some(state_callback));
            }
        }

        // Find the first index being empty
        // Assertion for performance: last state should have a long cooldown
        for idx in (0..self.batch_count).rev() {
            if self.slots[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                return self.swap(idx, state, Some(state_id), Some(state_callback));
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
    /// Runs the pipeline, this should be used internally.
    ///
    /// Use `Pipeline::start` instead.
    pub async fn start(
        batch_size: usize,
        context: Context,
        model: Arc<Model<'static>>,
        request_lock: BatchRequest,
    ) -> (mpsc::Sender<Vec<InferRequest>>, JoinHandle<()>) {
        let (sender, mut receiver) = mpsc::channel::<Vec<InferRequest>>(batch_size);
        let handle = tokio::spawn(async move {
            let mut slots = Slots::new(batch_size, &context, model, request_lock).await;
            let mut queued_requests: Vec<InferRequest> = Vec::new();

            // When something arrives in the channel.
            // This has an assumption that the batch is empty. (just initialized/ finished all inference)
            while let Some(requests) = receiver.recv().await {
                // Load the request
                slots.load_or_queue(requests, &mut queued_requests).unwrap();

                // Insert until slots full or no more requests
                if !slots.is_full() {
                    loop {
                        match receiver.try_recv() {
                            // Push requests and start infer if it's ok
                            Ok(requests) => {
                                slots.load_or_queue(requests, &mut queued_requests).unwrap();
                                if slots.can_start_infer() {
                                    break;
                                }
                            }
                            // Channel empty, wait until requests are loaded
                            Err(TryRecvError::Empty) => {
                                if slots.can_start_infer() {
                                    break;
                                } else {
                                    tokio::time::sleep(Duration::from_micros(100)).await;
                                }
                            }
                            // Channel closed, need to return
                            _ => return,
                        }
                    }
                }
                loop {
                    // Infer till at least 1 slot is done
                    slots.infer().unwrap();

                    // Release queued requests into the slots
                    while let Some(queued) = queued_requests.pop() {
                        slots.insert(queued).unwrap();
                        if slots.is_full() {
                            break;
                        }
                    }

                    // If there're empty slot, try to load from receiver
                    if !slots.can_start_infer() {
                        while let Ok(requests) = receiver.try_recv() {
                            slots.load_or_queue(requests, &mut queued_requests).unwrap();
                            if slots.can_start_infer() {
                                break;
                            }
                        }
                        // No requests anymore, release the lock
                        if slots.is_clear() {
                            break;
                        }
                    }
                }
            }
        });
        (sender, handle)
    }
}
