use anyhow::Result;
use std::fmt::Debug;

use crate::states::InferenceInterruption;

/// Transforms a logits distribution.
///
/// A good use of it is penalties.
pub trait Transformer: Send + Sync + Debug {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption>;
    fn transform(&self, logits: Vec<f32>) -> Vec<f32>;
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Transformer>;
}
