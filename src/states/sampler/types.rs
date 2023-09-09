use std::fmt::Debug;

use anyhow::Result;

use crate::states::InferenceInterruption;

/// Sample a token from probablities (after softmax).
///
/// Multiple logits might present (in case of CFG).
pub trait Sampler: Send + Sync + Debug {

    /// Updates the sampler and mark it ready for next sampling.
    /// 
    /// If the sampler returns Exhaustion, the inference will stop and
    /// return at where it is, or an error will be thrown.
    fn update(&mut self, tokens: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption>;
    fn sample(& self, probs: Vec<Vec<f32>>) -> u16;
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Sampler>;
}
