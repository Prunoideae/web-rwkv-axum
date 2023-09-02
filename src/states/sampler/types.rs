use anyhow::Result;

/// Sample a token from logits.
///
/// Multiple logits might present (in case of CFG).
pub trait Sampler {
    /// Initialize the sampler.
    ///
    /// This is done separately due to access to AppState is needed.
    fn sample(&mut self, probs: Vec<Vec<f32>>) -> Result<u16>;
}



