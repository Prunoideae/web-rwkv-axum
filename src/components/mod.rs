use anyhow::Error;

pub mod model;
pub mod infer;
pub mod permit;
pub mod sampler;
pub mod softmax;
pub mod state;
pub mod state_new;
pub mod terminal;
pub mod transformer;
pub mod normalizer;

pub enum InferenceInterruption {
    Exhaustion,
    Error(Error),
}

impl InferenceInterruption {
    pub fn exhausted(&self) -> bool {
        match self {
            InferenceInterruption::Exhaustion => true,
            InferenceInterruption::Error(_) => false,
        }
    }
}
