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
        context: InferContext,
        sender: mpsc::Sender<InferRequest>,
    ) -> Result<InferResult> {
        let (callback, receiver) = oneshot::channel();
        let request = InferRequest { context, callback };
        sender.send(request).await?;
        Ok(receiver.await?)
    }
}
