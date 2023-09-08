use anyhow::Result;
use std::fmt::Debug;

/// Transforms a logits distribution.
///
/// A good use of it is penalties.
pub trait Transformer: Send + Sync + Debug {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<()>;
    fn transform(&self, logits: &mut Vec<f32>);
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Transformer>;
}
