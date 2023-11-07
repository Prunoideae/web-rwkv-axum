use std::fmt::Debug;

use anyhow::Result;

use crate::components::InferenceInterruption;

/// Sample a token from probablities (after softmax).
///
/// Multiple logits might present (in case of CFG).
///
/// #### Registration
///
/// A sampler type needs to be registered before it can be constructed by the Websocket API``.
///
/// To register a sampler, put the type_id (a literal string) with the constructor (which
/// is a `Fn(SharedState, Option<Value>)->Result<Box<dyn Sampler>>`) in the `new()` of
/// `Samplers`.
///
/// Refer to `TypicalSampler` for a complete example of sampler implementation.
pub trait Sampler: Send + Sync + Debug {
    /// Updates the internal state of the sampler by accepting a list of tokens.
    ///
    /// The update will be called for both the prompt and autoregressive generation.
    /// There will be no way to know if the call is from a prompt or an autoregressive
    /// generation, there're other Websocket APIs or infer params which can bypass
    /// the update.
    ///
    /// Once the update is completed, the **infer** will start, so sampler must **preceive**
    /// if it can or can not accept any further input, and interrupt the generation by
    /// returning `Err(InferenceInterruption::Exhaustion)`.
    #[allow(unused_variables)]
    fn update(&mut self, tokens: &Vec<Vec<u16>>) -> Result<(), InferenceInterruption> {
        Ok(())
    }
    /// Samples a token from one or *more* probabilities, which is `softmax`ed from one
    /// or more states. For each probs distribution, it is guaranteed to have a sum of 1.
    ///
    /// Usually, the sampler would only accept 1 probs `Vec` -> 1 token mapping
    /// like `typical` or `nucleus` would do, but there are also sampling methods like
    /// `CFG Sampling` which samples from multiple parallel states. Note that only 1
    /// token will be sampled from the list and selected as the next token for *all states*.
    fn sample(&self, probs: Vec<Vec<f32>>) -> u16;
    /// Clears the `Sampler`. This will reset the internal state of the sampler to *when it
    /// is just constructed from params*.
    fn clear(&mut self) {}
    /// Copies the internal state (no matter if it's from construction or temporal calculation),
    /// and construct a new `Sampler` from the state.
    ///
    /// Deep copy or shallow copy should be handled by the sampler itself, if you can ensure
    /// that the state mutated in `update` will not mutate the cloned state, it is safe to
    /// share internal state by using `Arc`, etc.
    fn clone(&self) -> Box<dyn Sampler>;
}
