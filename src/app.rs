use std::sync::Arc;

use anyhow::{Error, Result};
use tokio::sync::{mpsc::Sender, oneshot};
use web_rwkv::{context::Context, tokenizer::Tokenizer};

use crate::{
    components::{
        model::AxumModel, normalizer::Normalizers, sampler::Samplers, softmax::Softmax,
        state::InferStates, terminal::Terminals, transformer::Transformers,
    },
    config::ModelConfig,
};

pub struct InnerState {
    pub config: ModelConfig,
    pub samplers: Arc<Samplers>,
    pub transformers: Arc<Transformers>,
    pub terminals: Arc<Terminals>,
    pub normalizers: Arc<Normalizers>,
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

        let softmax = Softmax::new(model.clone(), config.model.get_batch_size()).await;
        let (softmax_sender, _) = softmax.run().await;

        Ok(AppState(Arc::new(InnerState {
            config: config.clone(),
            samplers: Arc::new(Samplers::new()),
            transformers: Arc::new(Transformers::new()),
            terminals: Arc::new(Terminals::new()),
            normalizers: Arc::new(Normalizers::new()),
            softmax_queue: softmax_sender,
            tokenizer: Arc::new(config.tokenizer.load_tokenizer().await?),
            context: context.clone(),
            model: model.clone(),
            states: InferStates::new(config, context.clone(), model.clone()).await?,
        })))
    }

    pub async fn update_state(&self, id: Vec<String>, tokens: Vec<Vec<u16>>) -> Result<()> {
        let flags = id.iter().map(|_| true).collect();
        let mut ticket = self.0.states.create_ticket(id, flags).await?;
        ticket.infer(tokens).await;
        Ok(())
    }

    pub async fn dump_state(&self, id: String, dump_id: String) -> Result<()> {
        self.0
            .states
            .get_state(&id)
            .ok_or(Error::msg("State not found!"))?
            .dump(self.0.config.axum.state_dump.join(dump_id))
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
