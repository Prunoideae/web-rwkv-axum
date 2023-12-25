use anyhow::Result;
use std::fmt::Debug;

pub trait Terminal: Send + Sync + Debug {
    /// Determines if the generation should be stopped.
    ///
    /// If the terminate returns True, then current generation loop will
    /// be stopped, and the result string will be returned.
    ///
    /// However, this does not mean the generation itself is stopped -
    /// clients can still make autoregressive calls to continue the
    /// generation.
    fn terminate(&mut self, result: &Vec<u16>, token_count: usize) -> Result<bool>;
    /// Clears the `Terminal`. This will reset the internal state of the
    /// terminal to *when it is just constructed from params*.
    fn clear(&mut self) {}
    /// Copies the internal state (no matter if it's from construction or
    /// temporal calculation) and construct a new `Terminal` from the state.
    ///
    /// Deep copy or shallow copy should be handled by the sampler itself,
    /// if you can ensure that the state mutated in `update` will not mutate
    /// the cloned state, it is safe to share internal state by using `Arc`,
    /// etc.
    fn clone(&self) -> Box<dyn Terminal>;
}
