use std::{
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    thread::spawn,
};

use itertools::Itertools;
use lru::LruCache;
use nohash_hasher::BuildNoHashHasher;
use tokio::sync::mpsc;
use web_rwkv::context::Context;

use crate::components::model::{AxumModel, AxumModelState};

use super::state::NamedState;

/// Represents a *continuous* infer request.
///
/// A continuous infer request will have to send tokens sequentially
/// without much interruption between each send. The infer loop will
/// block for each active infer request to wait for their tokens to
/// arrive, so the infer request must feed sampled tokens quickly back
/// to the infer loop.
///
/// An active infer request will prevent other states to enter a same
/// slot to reduce copying and waiting to mininum. An infer request
/// is considered inactive when the `callback` sender is closed.
///
/// The state will not be updated instantly when the request ends. It
/// is possible for another request from client side enters before
/// current request state is swapped out. So the state will not be
/// synced unless explicitly called or swapped out and have update_state
/// being true.
pub struct InferRequest {
    state: NamedState,
    tokens: mpsc::Receiver<Vec<u16>>,
    callback: mpsc::Sender<Vec<f32>>,
}

struct InferIO {
    tokens: mpsc::Receiver<Vec<u16>>,
    callback: mpsc::Sender<Vec<f32>>,
}

#[derive(Clone)]
struct InferState {
    state: NamedState,
}

impl InferRequest {
    pub fn new(
        state: NamedState,
        tokens: mpsc::Receiver<Vec<u16>>,
        callback: mpsc::Sender<Vec<f32>>,
    ) -> Self {
        Self {
            state,
            tokens,
            callback,
        }
    }

    fn split(self) -> (InferIO, InferState) {
        (
            InferIO {
                tokens: self.tokens,
                callback: self.callback,
            },
            InferState { state: self.state },
        )
    }
}

impl InferState {
    fn get_id<'a>(&'a self) -> &'a String {
        self.state.get_id()
    }
}

type Cache = Arc<RwLock<LruCache<usize, InferState, BuildNoHashHasher<usize>>>>;

struct Slots {
    ios: Vec<Option<InferIO>>,
    tokens_cache: Vec<Vec<u16>>,
    pool: Arc<AxumModelState>,
    cache: Cache,
}

impl Slots {
    pub fn new(batch_size: usize, pool: Arc<AxumModelState>, cache: Cache) -> Self {
        Self {
            ios: (0..batch_size).map(|_| None).collect_vec(),
            tokens_cache: (0..batch_size).map(|_| Vec::new()).collect_vec(),
            pool,
            cache,
        }
    }

    fn insert(&mut self, request: InferRequest) {
        let (io, state) = request.split();

        let empty_slots = self
            .ios
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_none())
            .map(|(x, _)| x)
            .collect_vec();

        let selected_slot = {
            let mut cache = self.cache.write().unwrap();
            // Check if the state is in slot already
            if let Some(index) = empty_slots
                .iter()
                .map(|x| cache.peek(x).map(|s| (x, s)))
                .flatten()
                .filter(|(_, s)| s.state == state.state)
                .map(|(x, _)| *x)
                .next()
            {
                cache.promote(&index);
                index
            } else if let Some(&index) = empty_slots
                .iter()
                //Find an empty slot and load the state
                .filter(|x| !cache.contains(x))
                .next()
            {
                if let Some(state) = cache.put(index, state.clone()) {
                    if state.state.is_valid() {
                        state.state.back_from(&self.pool, index);
                    }
                }
                state.state.load_to(&self.pool, index);
                index
            } else if let Some(index) = cache
                .iter()
                // Find the least used slot
                .rev()
                .filter(|(x, _)| empty_slots.contains(x))
                .next()
                .map(|(x, _)| *x)
            {
                if let Some(state) = cache.put(index, state.clone()) {
                    if state.state.is_valid() {
                        state.state.back_from(&self.pool, index);
                    }
                };
                state.state.load_to(&self.pool, index);
                index
            } else {
                unreachable!()
            }
        };

        // Set io
        self.ios[selected_slot] = Some(io);
    }

    fn infer(&mut self, model: &AxumModel) {
        // Try to get tokens from infer loop
        for (tokens, io) in self.tokens_cache.iter_mut().zip(self.ios.iter_mut()) {
            if let Some(io) = io {
                if let Some(incoming_tokens) = {
                    if tokens.is_empty() {
                        // If no token is in cache (but it has an active ticket)
                        // we block until token arrives.
                        //
                        // This assumes that pipeline loop time << infer time, so
                        // blocking here makes the infer more batched.
                        io.tokens.blocking_recv()
                    } else {
                        // We also peek into the tokens.
                        io.tokens.try_recv().ok()
                    }
                } {
                    tokens.extend(incoming_tokens);
                }
            } else if !tokens.is_empty() {
                // Clear up tokens in case of accidental shutdown.
                tokens.clear();
            }
        }

        // Ensure that we still have something to send
        if self.tokens_cache.iter().all(|x| x.is_empty()) {
            return;
        }
        for (index, logits) in model
            .infer(&mut self.tokens_cache, &self.pool)
            .unwrap()
            .into_iter()
            .enumerate()
        {
            if let Some(logits) = logits {
                self.ios[index]
                    .as_mut()
                    .unwrap()
                    .callback
                    // We don't care if it's disconnected already
                    .try_send(logits)
                    .ok();
            }
        }
    }

    /// Is there no active ticket in the infer loop?
    fn all_clear(&self) -> bool {
        self.ios.iter().all(|x| x.is_none())
    }

    /// Remove invalid tickets where IOs are closed.
    fn clear(&mut self) {
        for io in self.ios.iter_mut() {
            // If the sender is dropped, then the ticket is no longer valid.
            if io.as_ref().is_some_and(|io| io.callback.is_closed()) {
                *io = None;
            }
        }
    }
}

struct InnerPool {
    pool: Arc<AxumModelState>,
    model: Arc<AxumModel>,
    cache: Cache,
    batch_size: usize,
}

#[derive(Clone)]
pub struct InferPool(Arc<InnerPool>);

impl InferPool {
    pub fn new(
        context: Context,
        model: Arc<AxumModel>,
        batch_size: usize,
        state_size: Option<usize>,
    ) -> Self {
        let pool = Arc::new(AxumModelState::new_sized(
            &context, &model, batch_size, state_size,
        ));
        Self(Arc::new(InnerPool {
            pool,
            model,
            cache: Arc::new(RwLock::new(LruCache::with_hasher(
                NonZeroUsize::new(batch_size).unwrap(),
                BuildNoHashHasher::default(),
            ))),
            batch_size,
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
            state.state.back_from(&self.0.pool, *index);
        }
    }

    fn infer_loop(&self, mut queue: mpsc::Receiver<Vec<InferRequest>>) {
        let mut slots = Slots::new(self.0.batch_size, self.0.pool.clone(), self.0.cache.clone());

        // When something arrives in the channel.
        // This has an assumption that the batch is empty.
        while let Some(requests) = queue.blocking_recv() {
            // Clear up the slots to remove done requests (sender closed)
            slots.clear();
            requests
                .into_iter()
                .for_each(|request| slots.insert(request));
            loop {
                // Infer till at least 1 slot is done, this blocks on all
                // active infer requests to wait for token input from all
                // requests
                slots.infer(&self.0.model);

                // Try to receive more requests, note this is guarded by
                // a semaphore elsewhere
                slots.clear();
                while let Ok(requests) = queue.try_recv() {
                    requests
                        .into_iter()
                        .for_each(|request| slots.insert(request));
                }
                // Break if everything is done so we continue waiting
                if slots.all_clear() {
                    break;
                }
            }
        }
    }

    pub async fn start_loop(&self) -> mpsc::Sender<Vec<InferRequest>> {
        let (sender, receiver) = mpsc::channel(self.0.batch_size);
        let looped = self.clone();
        spawn(move || {
            looped.infer_loop(receiver);
        });
        sender
    }
}
