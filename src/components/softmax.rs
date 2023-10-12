use anyhow::{Error, Result};
use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use super::model::TypelessModel;

#[derive(Clone)]
pub struct Softmax {
    model: Arc<TypelessModel>,
    max_batch_size: usize,
}

impl Softmax {
    pub async fn new(model: Arc<TypelessModel>, max_batch_size: usize) -> Self {
        Self {
            model,
            max_batch_size,
        }
    }

    async fn softmax_loop(
        &self,
        mut receiver: mpsc::Receiver<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    ) {
        let mut queue = Vec::with_capacity(self.max_batch_size);
        while let Some(requests) = receiver.recv().await {
            queue.extend(requests);

            // Sleep for 5us to make things batched
            tokio::time::sleep(Duration::from_micros(10)).await;

            while let Ok(requests) = receiver.try_recv() {
                queue.extend(requests);
                if queue.len() >= self.max_batch_size {
                    break;
                }
            }

            let (softmax_queue, sender_queue): (Vec<Vec<f32>>, Vec<oneshot::Sender<Vec<f32>>>) =
                queue
                    .split_off(if self.max_batch_size > queue.len() {
                        0
                    } else {
                        queue.len() - self.max_batch_size
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
        }
    }

    pub async fn run(
        self,
    ) -> (
        mpsc::Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
        JoinHandle<()>,
    ) {
        let (sender, receiver) = mpsc::channel(self.max_batch_size);
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
}
