use anyhow::{Ok, Result};
use tokio::sync::{mpsc, oneshot};

use crate::helper::{Logits, State};

#[derive(Debug)]
pub struct InferContext {
    pub state: Option<State>,
    pub tokens: Vec<u16>,
}

#[derive(Debug)]
pub struct InferResult {
    pub logits: Logits,
}

#[derive(Debug)]
/// Represents a request to infer pipeline. Not meant to be constructed on user side.
///
/// Use `InferRequest::send` instead.
pub struct InferRequest {
    pub context: InferContext,
    pub callback: oneshot::Sender<InferResult>,
    pub state_id: String,
    pub state_callback: oneshot::Sender<Option<State>>,
}

impl InferRequest {
    /// Queue an infer request to the pipeline.
    pub async fn send(
        contexts: Vec<InferContext>,
        sender: mpsc::Sender<Vec<InferRequest>>,
        state_ids: Vec<String>,
        state_callbacks: Vec<oneshot::Sender<Option<State>>>,
    ) -> Result<Vec<InferResult>> {
        let (receivers, requests): (Vec<oneshot::Receiver<InferResult>>, Vec<InferRequest>) =
            contexts
                .into_iter()
                .zip(state_ids.into_iter())
                .zip(state_callbacks.into_iter())
                .map(|((context, id), state_callback)| {
                    let (callback, receiver) = oneshot::channel();
                    (
                        receiver,
                        InferRequest {
                            context,
                            callback,
                            state_id: id,
                            state_callback,
                        },
                    )
                })
                .unzip();

        sender.send(requests).await?;
        let mut results = Vec::new();
        for receiver in receivers {
            results.push(receiver.await?);
        }
        Ok(results)
    }
}
