use std::{
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};

use itertools::Itertools;
use lru::LruCache;
use nohash_hasher::BuildNoHashHasher;
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
type Cache = Arc<RwLock<LruCache<usize, InferState, BuildNoHashHasher<usize>>>>;

struct Slots {
    tasks: usize,
    batch_lock: BatchRequest,
    max_concurrency: usize,
    token_slots: Vec<Vec<u16>>,
    callback_slots: Vec<Option<oneshot::Sender<Vec<f32>>>>,
    pool: Arc<AxumModelState>,
    cache: Cache,
}

struct InnerPool {
    max_concurrency: usize,
    batch_size: usize,
    batch_lock: BatchRequest,
    model: Arc<AxumModel>,
    pool: Arc<AxumModelState>,
    cache: Cache,
}

#[derive(Clone)]
pub struct InferPool(Arc<InnerPool>);

impl Slots {
    pub fn new(
        batch_size: usize,
        pool: Arc<AxumModelState>,
        lock: BatchRequest,
        max_concurrency: usize,
        cache: Cache,
    ) -> Self {
        Self {
            tasks: 0,
            max_concurrency,
            token_slots: (0..batch_size).map(|_| Vec::new()).collect_vec(),
            callback_slots: (0..batch_size).map(|_| None).collect_vec(),
            batch_lock: lock,
            pool,
            cache,
        }
    }

    #[inline(always)]
    fn empty_slots(&self) -> usize {
        self.callback_slots.len() - self.tasks
    }

    #[inline(always)]
    fn can_start_infer(&self) -> bool {
        return self.tasks >= self.max_concurrency
            || self.tasks >= self.batch_lock.get()
            || self.empty_slots() == 0;
    }

    /// Insert a request into the slot
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

        let selected_slot = {
            let mut cache = self.cache.write().unwrap();
            // Check if state is in the slot already
            if let Some(index) = {
                empty_slots
                    .iter()
                    .map(|x| cache.peek(x).map(|s| (x, s)))
                    .flatten()
                    .filter(|(_, s)| s.get_id() == state.get_id())
                    .map(|(x, _)| *x)
                    .next()
            } {
                cache.promote(&index);
                index
            } else if let Some(&index) =
                { empty_slots.iter().filter(|x| !cache.contains(x)).next() }
            {
                // Find a unused slot
                if let Some(state) = cache.put(index, state.clone()) {
                    state.back_from(&self.pool, index);
                };
                state.load_to(&self.pool, index);
                index
            } else {
                // Try to find the least used slot
                if let Some(index) = {
                    cache
                        .iter()
                        .rev()
                        .filter(|(x, _)| empty_slots.contains(x))
                        .next()
                        .map(|(x, _)| *x)
                } {
                    if let Some(state) = cache.put(index, state.clone()) {
                        state.back_from(&self.pool, index);
                    };
                    state.load_to(&self.pool, index);
                    index
                } else {
                    unreachable!()
                }
            }
        };

        // Set tokens and sender
        self.token_slots[selected_slot] = tokens;
        self.callback_slots[selected_slot] = Some(callback);
        self.tasks += 1;
    }

    fn finish(&mut self, index: usize, logits: Vec<f32>) {
        if let Some(sender) = std::mem::replace(&mut self.callback_slots[index], None) {
            sender.send(logits).unwrap();
            self.tasks -= 1;
        } else {
            unreachable!()
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
        max_concurrency: usize,
        batch_size: usize,
        state_size: Option<usize>,
        batch_lock: BatchRequest,
    ) -> Self {
        Self(Arc::new(InnerPool {
            max_concurrency,
            batch_size,
            batch_lock,
            model: model.clone(),
            pool: Arc::new(AxumModelState::new_sized(&context, &model, batch_size, state_size)),
            cache: Arc::new(RwLock::new(LruCache::with_hasher(
                NonZeroUsize::new(batch_size).unwrap(),
                BuildNoHashHasher::default(),
            ))),
        }))
    }

    pub fn sync(&self, state_id: &str) {
        if let Some((index, state)) = self
            .0
            .cache
            .read()
            .unwrap()
            .iter()
            .filter(|(_, state)| state.get_id() == state_id)
            .next()
        {
            state.back_from(&self.0.pool, *index)
        }
    }

    async fn infer_loop(&self, mut queue: mpsc::Receiver<Vec<InferRequest>>) {
        let mut slots = Slots::new(
            self.0.batch_size,
            self.0.pool.clone(),
            self.0.batch_lock.clone(),
            self.0.max_concurrency,
            self.0.cache.clone(),
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
