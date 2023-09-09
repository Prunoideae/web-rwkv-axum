#

## Transformer

Well, it's not that *Transformer*, this is the **Logits Transformer** which is used to
*mutate* a logits distribution into another. This is useful for implementing penalties,
or other transformations.

A typical example is the *BNF Grammar constrain* which is a powerful and bulletproof
tool to force the model to generate text following a certain format.

The transforming process is computation-heavy and blocking, thus no `async` is present
in the transformer trait (and I don't really like `async-trait` because it just sucks).

## Trait

The `Transformer` trait is defined as followed, the `Transformer` must be able to made
into a `trait object` in order to be dynamically dispatched:

```rust
/// Transforms a logits distribution.
///
/// A good use of it is penalties.
pub trait Transformer: Send + Sync + Debug {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption>;
    fn transform(&self, logits: &mut Vec<f32>);
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Transformer>;
}
```

### `fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption>;`

Updates the internal state of the transformer by accepting a list of tokens.

The update will be called for both the prompt and autoregressive generation. There
will be no way to know if the call is from a prompt or an autoregressive generation,
there're other Websocket APIs or infer params which can bypass the update.

Once the update is completed, the **infer** will start without any interrution, so transformer must preceive if it can or can not accept any further input, and interrupt the generation by returning `Err(InferenceInterruption::Exhaustion)`.

### `fn transform(&self, logits: &mut Vec<f32>);`

Transform a logits distribution to another by mutating the input mutable reference of logits. This occurss *before* `softmax` to ensure a probs sum of 1 at `sampling`.

The transformer is only responsible to *1* logits distribution, and user can specify multiple transformers for different states/infer requests in your pipeline.

This function must be **infallible**, as any interruption is checked when updated.

### `fn clear(&mut self);`

Clears the `Transformer`. This will reset the internal state of the Transformer to *when it*
*is just constructed from params*.

### `fn clone(&self) -> Box<dyn Transformer>;`

Copies the internal state (no matter if it's from construction or temporal calculation),
and construct a new `Transformer` from the state.
