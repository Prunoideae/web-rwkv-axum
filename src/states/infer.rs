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
    pub state: State,
    pub logits: Logits,
}

#[derive(Debug)]
/// Represents a request to infer pipeline. Not meant to be constructed on user side.
///
/// Use `InferRequest::send` instead.
pub struct InferRequest {
    pub context: InferContext,
    pub callback: oneshot::Sender<InferResult>,
}

impl InferRequest {
    /// Queue an infer request to the pipeline.
    pub async fn send(
        contexts: Vec<InferContext>,
        sender: mpsc::Sender<Vec<InferRequest>>,
    ) -> Result<Vec<InferResult>> {
        let (receivers, requests): (Vec<oneshot::Receiver<InferResult>>, Vec<InferRequest>) =
            contexts
                .into_iter()
                .map(|context| {
                    let (callback, receiver) = oneshot::channel();
                    (receiver, InferRequest { context, callback })
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
