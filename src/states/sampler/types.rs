use std::fmt::Debug;

use anyhow::Result;

/// Sample a token from probablities (after softmax).
///
/// Multiple logits might present (in case of CFG).
pub trait Sampler: Send + Sync + Debug {
    fn update(&mut self, tokens: &Vec<Vec<u16>>) -> Result<()>;
    fn sample(& self, probs: Vec<Vec<f32>>) -> u16;
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Sampler>;
}
