use anyhow::Error;

pub mod infer;
pub mod permit;
pub mod sampler;
pub mod softmax;
pub mod state;
pub mod terminal;
pub mod transformer;

pub enum InferenceInterruption {
    Exhaustion,
    Error(Error),
}
