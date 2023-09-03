use std::fmt::Debug;

use anyhow::Result;

use crate::helper::Logits;

/// Sample a token from probablities (after softmax).
///
/// Multiple logits might present (in case of CFG).
pub trait Sampler: Send + Sync + Debug {
    fn update(&mut self, tokens: Vec<u16>);
    fn sample(&mut self, probs: Vec<Logits>) -> Result<u16>;
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Sampler>;
}
