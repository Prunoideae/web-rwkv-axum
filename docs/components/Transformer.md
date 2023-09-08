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
    fn update(&mut self, prompt: &Vec<u16>);
    fn transform(&self, logits: &mut Vec<f32>);
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Transformer>;
}
```
