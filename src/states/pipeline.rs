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

use crate::{
    config::ModelConfig,
    helper::{Logits, State},
};

use super::infer::{InferContext, InferRequest, InferResult};

struct Slots<'a, 'b> {
    slots: Vec<Option<oneshot::Sender<InferResult>>>,
    batch_tokens: Vec<Vec<u16>>,
    batch: ModelState<'a>,
    model: &'a Model<'a, 'b>,
    batch_count: usize,
}

impl Slots<'_, '_> {
    fn new<'a, 'b>(
        batch_count: usize,
        context: &'a Context,
        model: &'a Model<'a, 'b>,
    ) -> Slots<'a, 'b> {
        Slots {
            slots: (0..batch_count).map(|_| None).collect(),
            batch_tokens: (0..batch_count).map(|_| Vec::new()).collect(),
            batch: ModelState::new(context, model.info(), batch_count),
            model,
            batch_count,
        }
    }

    fn get_remained(&self) -> usize {
        self.slots.iter().filter(|x| x.is_none()).count()
    }

    fn is_full(&self) -> bool {
        self.slots.iter().all(|c| c.is_some())
    }

    fn is_clear(&self) -> bool {
        self.slots.iter().all(|c| c.is_none())
    }

    /// Inserts a request into the batch
    /// Also uploads the state to the buffer
    fn insert(&mut self, request: InferRequest, model: &Model<'_, '_>) -> Result<()> {
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
            // then update the callback and tokens
            if self.slots[idx].is_none() {
                self.slots[idx] = callback;
                self.batch_tokens[idx] = tokens;
                let info = model.info();
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
    /// Runs the pipeline, this should be used internally.
    ///
    /// Use `Pipeline::start` instead.

    pub async fn start(
        config: &ModelConfig,
    ) -> (mpsc::Sender<InferRequest>, JoinHandle<Result<()>>) {
        let (sender, mut receiver) =
            mpsc::channel::<InferRequest>(config.model.get_batch_size() * 64);
        let config = config.clone();

        let handle = tokio::spawn(async move {
            let context = config.model.create_context().await.unwrap();
            let model = config.model.load_model(&context).await.unwrap();

            let mut slots = Slots::new(config.model.get_batch_size(), &context, &model);
            loop {
                // When something arrives in the channel.
                // This has an assumption that the batch is empty. (just initialized/ finished all inference)
                if let Some(request) = receiver.recv().await {
                    slots.insert(request, &model)?;

                    // Start an infer loop until all slots are done again with no incoming requests
                    loop {
                        // Insert until slots full or no more requests
                        for _ in 0..slots.get_remained() {
                            if let Ok(request) = receiver.try_recv() {
                                slots.insert(request, &model)?;
                            } else {
                                break; // We just continue with current slots
                            }
                        }

                        // Infer till at least 1 slot is done
                        slots.infer().await?;

                        // Check if any more requests are coming during the infer
                        // If yes, insert and continue
                        // If no, continue with current batches until all are clear
                        if let Ok(request) = receiver.try_recv() {
                            slots.insert(request, &model)?;
                        } else if slots.is_clear() {
                            break;
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
