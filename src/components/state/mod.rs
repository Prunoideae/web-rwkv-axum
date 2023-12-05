use std::{path::PathBuf, sync::Arc};

use anyhow::{Error, Result};
use dashmap::{mapref::one::Ref, DashMap};
use futures_util::future::join_all;
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};
use web_rwkv::context::Context;

use crate::config::ModelConfig;

use self::{
    pool::{InferPool, InferRequest},
    state::NamedState,
};

use super::model::AxumModel;

mod pool;
mod serde;
mod state;

struct InnerStates {
    context: Context,
    model: Arc<AxumModel>,
    pool: InferPool,
    states: DashMap<String, NamedState>,
    request_queue: mpsc::Sender<Vec<InferRequest>>,
    state_size: Option<usize>,
    task_lock: Arc<Semaphore>,
}

pub struct InferTicket {
    token_senders: Vec<mpsc::Sender<Vec<u16>>>,
    logits_receivers: Vec<mpsc::Receiver<Vec<f32>>>,
    // When this is dropped, the semaphore is released
    // so no need to r/w anything here
    _permit: OwnedSemaphorePermit,
}

impl InferTicket {
    fn create_ticket(
        states: Vec<NamedState>,
        permit: OwnedSemaphorePermit,
    ) -> (Self, Vec<InferRequest>) {
        let mut sender_vec = Vec::with_capacity(states.len());
        let mut receiver_vec = Vec::with_capacity(states.len());
        let mut requests_vec = Vec::with_capacity(states.len());
        for state in states.into_iter() {
            let (token_sender, token_receiver) = mpsc::channel(256);
            let (logits_sender, logits_receiver) = mpsc::channel(256);
            sender_vec.push(token_sender);
            receiver_vec.push(logits_receiver);
            requests_vec.push(InferRequest::new(state, token_receiver, logits_sender));
        }
        (
            InferTicket {
                token_senders: sender_vec,
                logits_receivers: receiver_vec,
                _permit: permit,
            },
            requests_vec,
        )
    }

    pub async fn infer(&mut self, tokens: Vec<Vec<u16>>) -> Vec<Vec<f32>> {
        for (tokens, sender) in tokens.into_iter().zip(self.token_senders.iter()) {
            sender.send(tokens).await.unwrap();
        }

        join_all(self.logits_receivers.iter_mut().map(|r| r.recv()))
            .await
            .into_iter()
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn state_size(&self) -> usize {
        self.token_senders.len()
    }
}

#[derive(Clone)]
pub struct InferStates(Arc<InnerStates>);

impl InferStates {
    pub fn new(config: &ModelConfig, context: Context, model: Arc<AxumModel>) -> Result<Self> {
        let pool = InferPool::new(
            context.clone(),
            model.clone(),
            config.model.get_batch_size(),
            config.model.get_max_state_size(),
        );
        let sender = pool.start_loop();
        Ok(Self(Arc::new(InnerStates {
            context,
            model,
            pool,
            states: DashMap::with_capacity(128),
            request_queue: sender,
            state_size: config.model.get_max_state_size(),
            task_lock: Arc::new(Semaphore::new(config.model.get_max_concurrency())),
        })))
    }

    pub async fn create_ticket(&self, states: Vec<String>) -> Result<InferTicket> {
        let states = states
            .into_iter()
            .map(|x| {
                self.0
                    .states
                    .get(&x)
                    .map(|x| x.clone())
                    .ok_or(Error::msg("State not found!"))
            })
            .collect::<Result<Vec<_>>>()?;

        let permit = self
            .0
            .task_lock
            .clone()
            .acquire_many_owned(states.len() as u32)
            .await
            .unwrap();

        let (ticket, request) = InferTicket::create_ticket(states, permit);
        self.0.request_queue.send(request).await.unwrap();
        Ok(ticket)
    }

    pub fn create_state(&self, state_id: &str) -> Result<()> {
        if self.0.states.contains_key(state_id) {
            return Err(Error::msg("State already exists!"));
        }
        self.0.states.insert(
            state_id.to_string(),
            NamedState::new(
                state_id.to_string(),
                self.0.context.clone(),
                self.0.model.clone(),
                self.0.state_size,
            ),
        );
        Ok(())
    }

    pub async fn load_state(&self, state_id: &str, dump_path: PathBuf) -> Result<()> {
        if self.0.states.contains_key(state_id) {
            return Err(Error::msg("State already exists!"));
        }
        self.0.states.insert(
            state_id.to_string(),
            NamedState::new_from(state_id.to_string(), dump_path).await?,
        );
        Ok(())
    }

    pub async fn copy_state(&self, src: &str, dst: &str, _shallow: bool) -> Result<()> {
        if self.0.states.contains_key(dst) {
            return Err(Error::msg("Destination state already exists!"));
        }
        if !self.0.states.contains_key(src) {
            return Err(Error::msg("Source state id doesn't exist!"));
        }
        self.0.pool.sync(src).await;
        let src_state = self
            .0
            .states
            .get(src)
            .ok_or(Error::msg("State was deleted after it is synced!"))?;
        let dst_state = src_state.clone_new(dst.to_string()).await?;
        self.0.states.insert(dst.to_string(), dst_state);
        Ok(())
    }

    pub async fn delete_state(&self, state_id: &str) -> Result<()> {
        match self.0.states.remove(state_id) {
            Some(_) => Ok(()),
            None => Err(Error::msg("State ID does not exist!")),
        }
    }

    #[inline(always)]
    pub fn has_state(&self, state_id: &str) -> bool {
        self.0.states.contains_key(state_id)
    }

    pub fn get_state<'a>(&'a self, state_id: &str) -> Option<Ref<'a, String, NamedState>> {
        self.0.states.get(state_id)
    }
}
