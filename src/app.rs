use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc::Sender, oneshot};
use web_rwkv::{context::Context, model::Model, tokenizer::Tokenizer};

use crate::{
    components::{
        permit::BatchRequest, sampler::Samplers, softmax::Softmax, state::InferStates,
        transformer::Transformers,
    },
    config::ModelConfig,
};

pub struct InnerState {
    pub config: ModelConfig,
    pub samplers: Arc<Samplers>,
    pub transformers: Arc<Transformers>,
    pub states: InferStates,
    softmax_queue: Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    pub tokenizer: Arc<Tokenizer>,
    pub context: Context,
    pub model: Arc<Model<'static>>,
    pub batch_request: BatchRequest,
}

#[derive(Clone)]
/// Global state holder of the entire app.
pub struct AppState(pub Arc<InnerState>);

impl AppState {
    pub async fn new(
        config: &ModelConfig,
        softmax_queue: Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
        context: Context,
        model: Arc<Model<'static>>,
        batch_request: BatchRequest,
    ) -> Result<Self> {
        Ok(AppState(Arc::new(InnerState {
            config: config.clone(),
            samplers: Arc::new(Samplers::new()),
            transformers: Arc::new(Transformers::new()),
            softmax_queue,
            tokenizer: Arc::new(config.tokenizer.load_tokenizer().await?),
            context,
            model,
            states: InferStates::new(config, batch_request.clone()).await?,
            batch_request: batch_request.clone(),
        })))
    }

    pub async fn update_state(&self, id: Vec<String>, tokens: Vec<Vec<u16>>) -> Result<()> {
        let _ = self.infer(id, tokens).await?;
        Ok(())
    }

    pub async fn create_state(&self, id: String) -> Result<()> {
        self.0.states.create_state(&id)
    }

    #[inline(always)]
    pub fn has_state(&self, id: &String) -> bool {
        self.0.states.has_state(&id)
    }

    pub async fn copy_state(&self, src: String, dst: String) -> Result<()> {
        self.0.states.copy_state(&src, &dst)
    }

    pub async fn delete_state(&self, id: String) -> Result<()> {
        self.0.states.delete_state(&id)
    }

    pub fn tokenize(&self, input: &Vec<u8>) -> Result<Vec<u16>> {
        Ok(self.0.tokenizer.encode(&input)?)
    }

    pub async fn infer(
        &self,
        state_keys: Vec<String>,
        token_vecs: Vec<Vec<u16>>,
    ) -> Result<Vec<Vec<f32>>> {
        self.0.states.infer(&state_keys, token_vecs).await
    }

    /// This must not fail, or the implementation is severly bugged
    pub async fn softmax(&self, logits: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        Softmax::softmax(logits, self.0.softmax_queue.clone()).await
    }
}
