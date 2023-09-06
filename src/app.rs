use std::sync::Arc;

use anyhow::{Error, Ok, Result};
use dashmap::DashMap;
use tokio::sync::mpsc::Sender;
use web_rwkv::tokenizer::Tokenizer;

use crate::{
    config::ModelConfig,
    helper::{Logits, State},
    states::{
        infer::{InferContext, InferRequest, InferResult},
        sampler::Samplers,
        transformer::Transformers,
    },
};

/// Global state holder of the entire app.
pub struct AppState {
    pub config: ModelConfig,
    pub samplers: Arc<Samplers>,
    pub transformers: Arc<Transformers>,
    infer_queue: Sender<InferRequest>,
    // State holders
    // Can be None to represent state not created by pipeline yet
    infer_states: Arc<DashMap<String, Option<State>>>,
    pub tokenizer: Arc<Tokenizer>,
}

impl AppState {
    pub async fn new(config: &ModelConfig, queue: Sender<InferRequest>) -> Result<Self> {
        Ok(AppState {
            config: config.clone(),
            samplers: Arc::new(Samplers::new()),
            transformers: Arc::new(Transformers::new()),
            infer_queue: queue,
            infer_states: Arc::new(DashMap::with_capacity(128)),
            tokenizer: Arc::new(config.tokenizer.load_tokenizer().await?),
        })
    }

    pub async fn update_state(&self, id: String, tokens: Vec<u16>) -> Result<()> {
        let _ = self.infer(id, tokens).await?;
        Ok(())
    }

    pub async fn create_state(&self, id: String) -> Result<()> {
        if self.infer_states.contains_key(&id) {
            return Err(Error::msg("State already exists!"));
        }
        self.infer_states.insert(id, None);
        Ok(())
    }

    pub async fn copy_state(&self, src: String, dst: String) -> Result<()> {
        if self.infer_states.contains_key(&dst) {
            return Err(Error::msg("Destination state id already exists!"));
        }
        let src = self
            .infer_states
            .get(&src)
            .ok_or(Error::msg("State doesn't exist!"))?
            .clone();
        self.infer_states.insert(dst, src);
        Ok(())
    }

    pub async fn delete_state(&self, id: String) -> Result<()> {
        self.infer_states
            .remove(&id)
            .ok_or(Error::msg("State doesn't exist!"))
            .map(|_| ())
    }

    pub async fn tokenize(&self, input: &Vec<u8>) -> Result<Vec<u16>> {
        Ok(self.tokenizer.encode(&input)?)
    }

    async fn infer(&self, state_key: String, tokens: Vec<u16>) -> Result<Logits> {
        let state = self
            .infer_states
            .get(&state_key)
            .ok_or(Error::msg(format!("State {} doesn't exist!", state_key)))?
            .clone();
        let context = InferContext { state, tokens };
        let InferResult { state, logits } =
            InferRequest::send(context, self.infer_queue.clone()).await?;
        self.infer_states.insert(state_key, Some(state));
        Ok(logits)
    }
}

pub type SharedState = Arc<AppState>;
