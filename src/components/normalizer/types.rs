use crate::components::InferenceInterruption;
use anyhow::{Error, Result};
use std::fmt::Debug;

pub trait Normalizer: Send + Sync + Debug {
    /// Updates the internal state of normalizer by accepting a list of tokens.
    ///
    /// The update will be called for both the prompt and autoregressive generations.
    /// There will be no way to know if the call is from a prompt or an autoregressive
    /// generation, there're other Websocket APIs or infer params which can bypass
    /// the update.
    ///
    /// Once the update is completed, the **infer** will start, so normalizer must **preceive**
    /// if it can or can not accept any further input, and interrupt the generation by
    /// returning `Err(InferenceInterruption::Exhaustion)`.
    #[allow(unused_variables)]
    fn update(&mut self, tokens: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption> {
        Ok(())
    }

    /// Normalizes a logits distribution.
    ///
    /// Returns a `Vec<Vec<f32>>` where at least first `Vec<f32>` is normalized. A normalized
    /// logits will have a sum of 1, making it safe for samplers to sample.
    ///
    /// You can call state.softmax_blocking to issue a softmax to model loop.
    ///
    /// A default implementation will have all logits normalized, but depending on the use case,
    /// some logits can be left out since the sample process only need to sample 1 logits and
    /// use others as some modifier.
    fn normalize(&self, logits: Vec<Vec<f32>>) -> Vec<Vec<f32>>;

    /// Clears the `Normalizer`. This will reset the internal state of the normalizer to *when it
    /// is just constructed from params*.
    fn clear(&mut self) {}

    /// Copies the internal state (no matter if it's from construction or temporal calculation),
    /// and construct a new `Normalizer` from the state.
    ///
    /// Deep copy or shallow copy should be handled by the normalizer itself, if you can ensure
    /// that the state mutated in `update` will not mutate the cloned state, it is safe to
    /// share internal state by using `Arc`, etc.
    fn clone(&self) -> Box<dyn Normalizer>;
    /// Implements a special update for prompt, where it will not throw InferenceInterruption::Exhaustion
    /// if the exhaustion happened. This is for special components like BNF which might exhaust when
    /// reading prompt.
    ///
    /// By default the exhaustion is considered as error as there's no correct way to handle exhaustion
    /// by default.
    fn update_prompt(&mut self, tokens: &Vec<Vec<u16>>) -> Result<()> {
        self.update(tokens).map_err(|e| match e {
            InferenceInterruption::Exhaustion => {
                Error::msg("Prompt exhaustion before infer starts.")
            }
            InferenceInterruption::Error(err) => err,
        })
    }
}
