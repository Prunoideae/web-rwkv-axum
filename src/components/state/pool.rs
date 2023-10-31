use std::sync::{Arc, RwLock};

use anyhow::{Error, Result};
use itertools::Itertools;
use tokio::sync::{mpsc, oneshot};
use web_rwkv::context::Context;

use crate::components::{
    model::{AxumModel, AxumModelState},
    permit::BatchRequest,
};

use super::state::InferState;

pub struct InferRequest {
    state: InferState,
    tokens: Vec<u16>,
    callback: oneshot::Sender<Vec<f32>>,
}

impl InferRequest {
    pub fn new(state: InferState, tokens: Vec<u16>, callback: oneshot::Sender<Vec<f32>>) -> Self {
        Self {
            state,
            tokens,
            callback,
        }
    }
}

type BatchSlots = Arc<RwLock<Vec<Option<InferState>>>>;

struct Slots {
    tasks: usize,
    batch_lock: BatchRequest,
    max_concurrent: usize,
    token_slots: Vec<Vec<u16>>,
    batch_slots: BatchSlots,
    callback_slots: Vec<Option<oneshot::Sender<Vec<f32>>>>,
    pool: Arc<AxumModelState>,
}

struct InnerPool {
    max_concurrent: usize,
    batch_size: usize,
    batch_lock: BatchRequest,
    pool: Arc<AxumModelState>,
    batch_slots: BatchSlots,
    #[allow(dead_code)] // probably one day we will use it
    context: Context,
    model: Arc<AxumModel>,
}

#[derive(Clone)]
pub struct InferPool(Arc<InnerPool>);

impl Slots {
    pub fn new(
        batch_slots: BatchSlots,
        pool: Arc<AxumModelState>,
        lock: BatchRequest,
        max_concurrent: usize,
    ) -> Self {
        let batch_size = batch_slots.read().unwrap().len();
        Self {
            tasks: 0,
            token_slots: (0..batch_size).map(|_| Vec::new()).collect_vec(),
            batch_slots,
            callback_slots: (0..batch_size).map(|_| None).collect_vec(),
            pool,
            batch_lock: lock,
            max_concurrent,
        }
    }

    #[inline(always)]
    fn empty_slots(&self) -> usize {
        self.callback_slots.len() - self.tasks
    }

    fn can_start_infer(&self) -> bool {
        return self.tasks >= self.max_concurrent
            || self.tasks >= self.batch_lock.get()
            || self.empty_slots() == 0;
    }

    #[inline(always)]
    fn swap(&mut self, index: usize, state: InferState) {
        let mut writelock = self.batch_slots.write().unwrap();
        if let Some(slot) = std::mem::replace(&mut writelock[index], Some(state)) {
            slot.back_from(&self.pool, index).unwrap();
        };
        writelock[index]
            .as_ref()
            .unwrap()
            .load_to(&self.pool, index)
            .unwrap();
    }

    /// Insert a request into the slot
    ///
    /// Must ensure that the slot is not full
    fn insert(&mut self, request: InferRequest) {
        let InferRequest {
            state,
            tokens,
            callback,
        } = request;

        let empty_slots = self
            .callback_slots
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_none())
            .map(|(x, _)| x)
            .collect::<Vec<_>>();

        let selected_slot = 'slot: {
            let readlock = self.batch_slots.read().unwrap();
            // Try to find an empty slot with a same name
            for index in empty_slots.iter() {
                if let Some(batch_state) = readlock[*index].as_ref() {
                    if &state == batch_state {
                        break 'slot *index;
                    }
                }
            }
            // Try to find an empty slot that is not occupied by any state
            for index in empty_slots.iter() {
                if let None = readlock[*index] {
                    break 'slot *index;
                }
            }
            // Try to find the first empty slot
            empty_slots[0]
        };

        self.token_slots[selected_slot] = tokens;
        self.callback_slots[selected_slot] = Some(callback);
        self.swap(selected_slot, state);
        self.tasks += 1;
    }

    fn finish(&mut self, index: usize, logits: Vec<f32>) {
        if let Some(sender) = std::mem::replace(&mut self.callback_slots[index], None) {
            sender.send(logits).unwrap();
            self.tasks -= 1;
        } else {
            panic!("Sender is empty!")
        }
    }

    fn infer(&mut self, model: &AxumModel) {
        let logits = loop {
            let logits_internal = model.run(&mut self.token_slots, &self.pool).unwrap();
            if logits_internal.iter().any(|l| l.is_some()) {
                break logits_internal;
            }
        };

        for (index, logits) in logits.into_iter().enumerate() {
            if logits.is_some() {
                self.finish(index, logits.unwrap())
            }
        }
    }

    fn load_or_queue(&mut self, mut requests: Vec<InferRequest>, queue: &mut Vec<InferRequest>) {
        for _ in 0..self.empty_slots() {
            if let Some(request) = requests.pop() {
                if !self.can_start_infer() {
                    self.insert(request);
                    continue;
                }
            }
            break;
        }
        queue.extend(requests);
    }
}

impl InferPool {
    pub fn new(
        context: Context,
        model: Arc<AxumModel>,
        max_concurrent: usize,
        batch_size: usize,
        batch_lock: BatchRequest,
        max_state_size: Option<usize>,
    ) -> Self {
        let slots = Arc::new(RwLock::new((0..batch_size).map(|_| None).collect_vec()));
        Self(Arc::new(InnerPool {
            max_concurrent,
            batch_lock,
            batch_size,
            pool: Arc::new(AxumModelState::new_sized(
                &context,
                &model,
                batch_size,
                max_state_size,
            )),
            context,
            model,
            batch_slots: slots,
        }))
    }

    pub fn sync(&self, state_id: &str) -> Result<()> {
        for (index, state) in self
            .0
            .batch_slots
            .read()
            .map_err(|_| Error::msg("Lock is poisoned!"))?
            .iter()
            .enumerate()
        {
            if let Some(state) = state.as_ref() {
                if state_id == state.get_id() {
                    state.back_from(&self.0.pool, index)?;
                    break;
                }
            }
        }
        Ok(())
    }

    async fn infer_loop(&self, mut queue: mpsc::Receiver<Vec<InferRequest>>) {
        let mut slots = Slots::new(
            self.0.batch_slots.clone(),
            self.0.pool.clone(),
            self.0.batch_lock.clone(),
            self.0.max_concurrent,
        );
        let mut queue_buffer: Vec<InferRequest> = Vec::with_capacity(self.0.batch_size);

        // When something arrives in the channel.
        // This has an assumption that the batch is empty. (just initialized/ finished all inference)
        while let Some(requests) = queue.recv().await {
            slots.load_or_queue(requests, &mut queue_buffer);

            // If there're more to come
            if !slots.can_start_infer() {
                // Wait for next request to arrive in
                while let Some(requests) = queue.recv().await {
                    slots.load_or_queue(requests, &mut queue_buffer);
                    // Since it's ready, break the waiting
                    if slots.can_start_infer() {
                        break;
                    }
                }
            }

            loop {
                // Infer till at least 1 slot is done
                slots.infer(&self.0.model);

                // Release queued requests into the slots
                while let Some(queued) = queue_buffer.pop() {
                    slots.insert(queued);
                    if slots.can_start_infer() {
                        break;
                    }
                }

                // If there're empty slot, try to load from receiver
                if !slots.can_start_infer() {
                    while let Ok(requests) = queue.try_recv() {
                        slots.load_or_queue(requests, &mut queue_buffer);
                        if slots.can_start_infer() {
                            break;
                        }
                    }
                    // No requests anymore, release the lock
                    if slots.tasks == 0 {
                        break;
                    }
                }
            }
        }
    }

    pub async fn start_loop(&self) -> mpsc::Sender<Vec<InferRequest>> {
        let (sender, receiver) = mpsc::channel(self.0.batch_size);
        let looped = self.clone();
        tokio::spawn(async move {
            looped.infer_loop(receiver).await;
        });
        sender
    }
}
