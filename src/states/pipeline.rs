use std::{sync::Arc, time::Duration};

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
    batch: ModelState,
    model: Arc<Model<'static>>,
    batch_count: usize,
}

impl Slots {
    async fn new(batch_count: usize, context: &Context, model: Arc<Model<'static>>) -> Slots {
        Slots {
            slots: (0..batch_count).map(|_| None).collect(),
            batch_tokens: (0..batch_count).map(|_| Vec::new()).collect(),
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

    /// Inserts a request into the batch
    /// Also uploads the state to the buffer
    async fn insert(&mut self, request: InferRequest) -> Result<()> {
        if self.is_full() {
            panic!("Batch is full while inserting new requests!")
        }

        let InferRequest {
            context: InferContext { state, tokens },
            callback,
        } = request;
        let callback = Some(callback);

        // Find the first index being empty
        for idx in 0..self.batch_count {
            // Sleep for 5us to make things batched
            tokio::time::sleep(Duration::from_micros(5)).await;
            // then update the callback and tokens
            if self.slots[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                let info = self.model.info();
                let backed_state = if let Some(state) = state {
                    let shape = Shape::new(info.num_emb, 5 * info.num_layers, 1);
                    BackedState {
                        shape,
                        data: state.0,
                    }
                } else {
                    BackedState::new(info, 1)
                };
                self.batch.load_batch(&backed_state, idx)?;
                break;
            }
        }
        Ok(())
    }

    /// Infer until any of the batch is completed.
    async fn infer(&mut self) -> Result<()> {
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
                    state: State(self.batch.back_batch(idx)?.data),
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
    async fn load_or_queue(
        requests: Vec<InferRequest>,
        slots: &mut Slots,
        queue: &mut Vec<InferRequest>,
    ) -> Result<()> {
        for request in requests {
            if slots.is_full() {
                queue.push(request);
            } else {
                slots.insert(request).await?;
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
    ) -> (mpsc::Sender<Vec<InferRequest>>, JoinHandle<Result<()>>) {
        let (sender, mut receiver) = mpsc::channel::<Vec<InferRequest>>(batch_size);
        let handle = tokio::spawn(async move {
            println!("Model is loaded!");
            let mut slots = Slots::new(batch_size, &context, model.clone()).await;
            let mut queued_requests: Vec<InferRequest> = Vec::new();
            loop {
                // When something arrives in the channel.
                // This has an assumption that the batch is empty. (just initialized/ finished all inference)
                if let Some(requests) = receiver.recv().await {
                    // Load the request
                    Pipeline::load_or_queue(requests, &mut slots, &mut queued_requests).await?;

                    // Start an infer loop until all slots are done again with no incoming requests
                    loop {
                        // Insert until slots full or no more requests
                        if !slots.is_full() {
                            while let Ok(requests) = receiver.try_recv() {
                                Pipeline::load_or_queue(requests, &mut slots, &mut queued_requests)
                                    .await?;
                                if slots.is_full() {
                                    break;
                                }
                            }
                        }

                        // Infer till at least 1 slot is done
                        slots.infer().await?;

                        // Release queued requests into the slots
                        while let Some(queued) = queued_requests.pop() {
                            slots.insert(queued).await?;
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
                                    .await?;
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
            anyhow::Ok(())
        });
        (sender, handle)
    }
}
