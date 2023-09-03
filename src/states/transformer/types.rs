use std::fmt::Debug;

use web_rwkv::tensor::TensorCpu;

/// Transforms a logits distribution.
///
/// A good use of it is penalties.
pub trait Transformer: Send + Sync + Debug {
    fn update_prompt(&mut self, prompt: Vec<u16>);
    fn update_token(&mut self, token: u16);
    fn transform(&mut self, logits: &mut TensorCpu<'_, '_, f32>);
    fn clear(&mut self);
}
