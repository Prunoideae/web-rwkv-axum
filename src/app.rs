use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use tokio::sync::{mpsc::Sender, oneshot};
use web_rwkv::{context::Context, tokenizer::Tokenizer};

use crate::{
    components::{
        model::AxumModel, pipeline::Pipelines, softmax::Softmax, state::InferStates, Registry,
    },
    config::ModelConfig,
};

pub struct InnerState {
    pub config: ModelConfig,
    pub pipelines: Arc<Pipelines>,
    pub registry: Arc<Registry>,
    pub states: InferStates,
    softmax_queue: Sender<Vec<(Vec<f32>, oneshot::Sender<Vec<f32>>)>>,
    pub tokenizer: Arc<Tokenizer>,
    pub context: Context,
    pub model: Arc<AxumModel>,
}
#[derive(Clone)]
/// Global state holder of the entire app.
pub struct AppState(pub Arc<InnerState>);

impl AppState {
    pub async fn new(config: &ModelConfig) -> Result<Self> {
        let context = config.model.create_context().await?;
        let model = Arc::new(config.model.load_model(&context).await?);
        println!("Model is loaded.");

        let softmax = Softmax::new(model.clone(), config.model.get_max_concurrency()).await;
        let (softmax_sender, _) = softmax.run().await;

        Ok(AppState(Arc::new(InnerState {
            config: config.clone(),
            pipelines: Arc::new(Pipelines::new()),
            registry: Arc::new(Registry::new()),
            softmax_queue: softmax_sender,
            tokenizer: Arc::new(config.tokenizer.load_tokenizer().await?),
            context: context.clone(),
            model: model.clone(),
            states: InferStates::new(config, context.clone(), model.clone())?,
        })))
    }

    pub async fn update_state(
        &self,
        id: Vec<String>,
        tokens: Vec<Vec<u16>>,
        token_probs: Option<Vec<u16>>,
    ) -> Result<Vec<Vec<f32>>> {
        let mut ticket = self.0.states.create_ticket(id).await?;
        let logits = ticket.infer(tokens).await;
        if let Some(token_probs) = token_probs {
            let probs = self.softmax(logits).await;
            Ok(probs
                .into_iter()
                .map(|probs| token_probs.iter().map(|&i| probs[i as usize]).collect())
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn dump_state(&self, id: String, dump_id: String) -> Result<()> {
        self.0
            .states
            .dump_state(&id, self.0.config.axum.state_dump.join(dump_id))
            .await
    }

    pub async fn load_state(&self, id: String, dump_id: String) -> Result<()> {
        self.0
            .states
            .load_state(&id, self.0.config.axum.state_dump.join(dump_id))
            .await
    }

    pub fn tokenize(&self, input: &Vec<u8>) -> Result<Vec<u16>> {
        Ok(self.0.tokenizer.encode(&input)?)
    }
    /// This must not fail, or the implementation is severely bugged
    pub async fn softmax(&self, logits: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        Softmax::softmax(logits, self.0.softmax_queue.clone()).await
    }

    pub fn softmax_blocking(&self, logits: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        Softmax::blocking_softmax(logits, self.0.softmax_queue.clone())
    }
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AppState").finish()
    }
}
