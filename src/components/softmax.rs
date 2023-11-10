use anyhow::{Error, Result};
use std::sync::Arc;

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use super::model::AxumModel;

#[derive(Clone)]
pub struct Softmax {
    model: Arc<AxumModel>,
    max_concurrency: usize,
}

impl Softmax {
    pub async fn new(model: Arc<AxumModel>, max_concurrency: usize) -> Self {
        Self {
            model,
            max_concurrency,
        }
    }

    async fn softmax_loop(
        &self,
        mut receiver: mpsc::Receiver<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    ) {
        let mut queue = Vec::with_capacity(self.max_concurrency);
        while let Some(requests) = receiver.recv().await {
            queue.extend(requests);
            loop {
                while let Ok(requests) = receiver.try_recv() {
                    queue.extend(requests);
                    if queue.len() >= self.max_concurrency {
                        break;
                    }
                }

                let (softmax_queue, sender_queue): (Vec<Vec<f32>>, Vec<oneshot::Sender<Vec<f32>>>) =
                    queue
                        .split_off(if self.max_concurrency > queue.len() {
                            0
                        } else {
                            queue.len() - self.max_concurrency
                        })
                        .into_iter()
                        .unzip();
                let softmax_queue = self.model.softmax(softmax_queue).unwrap();
                softmax_queue
                    .into_iter()
                    .zip(sender_queue.into_iter())
                    .map(|(result, sender)| {
                        sender
                            .send(result)
                            .map_err(|_| Error::msg("Can't send data due to channel is dropped!"))
                    })
                    .collect::<Result<Vec<_>>>()
                    .unwrap();

                if queue.is_empty() {
                    break;
                }
            }
        }
    }

    pub async fn run(
        self,
    ) -> (
        mpsc::Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
        JoinHandle<()>,
    ) {
        let (sender, receiver) = mpsc::channel(self.max_concurrency);
        (
            sender,
            tokio::spawn(async move { self.softmax_loop(receiver).await }),
        )
    }

    pub async fn softmax(
        logits: Vec<Vec<f32>>,
        sender: mpsc::Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    ) -> Vec<Vec<f32>> {
        let (receivers, requests): (
            Vec<oneshot::Receiver<Vec<f32>>>,
            Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>,
        ) = logits
            .into_iter()
            .map(|logits| {
                let (sender, receiver) = oneshot::channel();
                (receiver, (logits, sender))
            })
            .unzip();
        sender.send(requests).await.unwrap();
        let mut results = Vec::with_capacity(receivers.len());
        for receiver in receivers {
            results.push(receiver.await.unwrap());
        }
        results
    }

    /// A blocking (sync) version of softmax.
    /// This would block the current thread, so better use in blocking threads.
    pub fn blocking_softmax(
        logits: Vec<Vec<f32>>,
        sender: mpsc::Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    ) -> Vec<Vec<f32>> {
        let (receivers, requests): (
            Vec<oneshot::Receiver<Vec<f32>>>,
            Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>,
        ) = logits
            .into_iter()
            .map(|logits| {
                let (sender, receiver) = oneshot::channel();
                (receiver, (logits, sender))
            })
            .unzip();
        sender.blocking_send(requests).unwrap();
        let mut results = Vec::with_capacity(receivers.len());
        for receiver in receivers {
            results.push(receiver.blocking_recv().unwrap());
        }
        results
    }
}
