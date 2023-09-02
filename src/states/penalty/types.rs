/// Penaltize the logits distribution basing on history.
pub trait Penalty: Send + Sync {
    /// Initialize the sampler.
    ///
    /// This is done separately due to access to AppState is needed.
    fn update_prompt(&mut self, prompt: Vec<u16>);
    fn update_token(&mut self, token: u16);
    fn transform(&mut self, logits: Vec<f32>) -> Vec<f32>;
}
