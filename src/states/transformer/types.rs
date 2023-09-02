/// Transforms a logits distribution to another.
pub trait Transformer {
    /// Initialize the sampler.
    ///
    /// This is done separately due to access to AppState is needed.
    fn transform(&mut self, logits: Vec<f32>) -> Vec<f32>;
}
