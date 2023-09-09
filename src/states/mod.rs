use anyhow::Error;

pub mod infer;
pub mod permit;
pub mod pipeline;
pub mod sampler;
pub mod softmax;
pub mod transformer;

pub enum InferenceInterruption {
    Exhaustion,
    Error(Error),
}
