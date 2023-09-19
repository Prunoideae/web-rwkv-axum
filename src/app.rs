use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc::Sender, oneshot};
use web_rwkv::{context::Context, model::Model, tokenizer::Tokenizer};

use crate::{
    components::{
        permit::BatchRequest, sampler::Samplers, softmax::Softmax, state::InferStates,
        terminal::Terminals, transformer::Transformers,
    },
    config::ModelConfig,
};

pub struct InnerState {
    pub config: ModelConfig,
    pub samplers: Arc<Samplers>,
    pub transformers: Arc<Transformers>,
    pub terminals: Arc<Terminals>,
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
    pub async fn new(config: &ModelConfig) -> Result<Self> {
        let context = config.model.create_context().await?;
        let model = Arc::new(config.model.load_model(&context).await?);
        println!("Model is loaded.");
        let batch_request = BatchRequest::new();

        let softmax = Softmax::new(model.clone(), config.model.get_batch_size()).await;
        let (softmax_sender, _) = softmax.run().await;

        Ok(AppState(Arc::new(InnerState {
            config: config.clone(),
            samplers: Arc::new(Samplers::new()),
            transformers: Arc::new(Transformers::new()),
            terminals: Arc::new(Terminals::new()),
            softmax_queue: softmax_sender,
            tokenizer: Arc::new(config.tokenizer.load_tokenizer().await?),
            context: context.clone(),
            model: model.clone(),
            states: InferStates::new(
                config,
                context.clone(),
                model.clone(),
                batch_request.clone(),
            )
            .await?,
            batch_request: batch_request.clone(),
        })))
    }

    pub async fn update_state(&self, id: Vec<String>, tokens: Vec<Vec<u16>>) -> Result<()> {
        let _ = self.infer(id, tokens).await?;
        Ok(())
    }

    pub fn tokenize(&self, input: &Vec<u8>) -> Result<Vec<u16>> {
        Ok(self.0.tokenizer.encode(&input)?)
    }

    #[inline(always)]
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
