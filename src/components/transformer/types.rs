use anyhow::Result;
use std::fmt::Debug;

use crate::components::InferenceInterruption;

/// Transforms a logits distribution.
///
/// A good use of it is penalties.
///
/// #### Registration
///
/// A transformer type needs to be registered before it can be constructed by the Websocket
/// API.
///
/// To register a transformer, put the type_id (a literal string) with the constructor (which
/// is a `Fn(SharedState, Option<Value>)->Result<Box<dyn Transformer>>`) in the `new()` of
/// `Transformers`.
///
/// Refer to `GlobalPenalty` for a complete example of transformer implementation.
pub trait Transformer: Send + Sync + Debug {
    ///Updates the internal state of the transformer by accepting a list of tokens.
    ///
    /// The update will be called for both the prompt and autoregressive generation. There
    /// will be no way to know if the call is from a prompt or an autoregressive generation,
    /// there're other Websocket APIs or infer params which can bypass the update.
    ///
    /// Once the update is completed, the **infer** will start **without any interrution**, so transformer must preceive if it can or can not accept any further input, and interrupt the generation by returning `Err(InferenceInterruption::Exhaustion)`.
    #[allow(unused_variables)]
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        Ok(())
    }

    ///Transform a logits distribution to another by mutating the input mutable reference of logits. This occurss *before* `softmax` to ensure a probs sum of 1 at `sampling`.
    ///
    /// The transformer is only responsible to *1* logits distribution, and user can specify multiple transformers for different states/infer requests in the pipeline.
    ///
    /// This function must be **infallible**, as any interruption is checked when updated.
    fn transform(&self, logits: Vec<f32>) -> Vec<f32>;

    /// Clears the `Transformer`. This will reset the internal state of the Transformer to *when it*
    /// *is just constructed from params*.
    fn clear(&mut self) {}

    /// Copies the internal state (no matter if it's from construction or temporal calculation),
    /// and construct a new `Transformer` from the state.
    ///
    /// You can retain a part of the data (by using Arc, etc). As long as you are sure that those
    /// shared data are *immutable*, or any multi-threaded write access is *controlled*.
    fn clone(&self) -> Box<dyn Transformer>;
}
