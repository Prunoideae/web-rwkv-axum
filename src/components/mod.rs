use anyhow::Error;

pub mod infer;
pub mod model;
pub mod serde;
pub mod normalizer;
pub mod sampler;
pub mod softmax;
pub mod state;
pub mod terminal;
pub mod transformer;

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
