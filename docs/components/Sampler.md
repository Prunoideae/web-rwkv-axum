#

## Sampler

A `Sampler` will accept a `Vec` of probablities, which is possibly from 1 or *more*
states (depending on the use case) inferred in the current pipeline, and output a `u16`
which is the logits sampled.

The sampling process is computation-heavy and blocking, thus no `async` is present
in the sampler trait (and I don't really like `async-trait` because it just sucks).

## Trait

The `Sampler` trait is defined as followed, the `Sampler` must be able to made into
a `trait object` in order to be dynamically dispatched:

```rust
/// Sample a token from probablities (after softmax).
///
/// Multiple logits might present (in case of CFG).
pub trait Sampler: Send + Sync + Debug {
    fn update(&mut self, tokens: &Vec<Vec<u16>>);
    fn sample(&self, probs: Vec<Vec<f32>>) -> Result<u16>;
    fn clear(&mut self);
    fn clone(&self) -> Box<dyn Sampler>;
}
```

### `fn update(&mut self, tokens: &Vec<Vec<u16>>);`

Updates the internal state of the sampler by accepting a list of tokens.

The update will be called for both the prompt and autoregressive generation. There
will be no way to know if the call is from a prompt or an autoregressive generation,
there're other Websocket APIs or infer params which can bypass the update.

### `fn sample(&self, probs: Vec<Vec<f32>>) -> Result<u16>;`

Samples a token from one or *more* probabilities, which is softmaxed from one or
more states.

Usually, the sampler would only accept 1 probs `Vec` like `typical` or `nucleus` would
do, but there are also sampling methods like `CFG` which samples from multiple parallel
states. Note that only 1 token will be sampled from the list and selected as the next
token for *all states*.

### `fn clear(&mut self);`

Clears the `Sampler`. This will reset the internal state of the sampler to *when it*
*is just constructed from params*.

### `fn clone(&self) -> Box<dyn Sampler>;`

Copies the internal state (no matter if it's from construction or temporal calculation),
and construct a new `Sampler` from the state.

## Registration

A sampler type need to be registered before it can be constructed by the Websocket API. To
register a Sampler, first create a method, let's use the `TypicalSampler` as an example:

```rust
/// Typical sampler for logits
#[derive(Debug, Clone, Deserialize)]
pub struct TypicalSampler {
    top_p: f32,
    temp: f32,
}

pub fn initialize_typical(_state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
    Ok(Box::new(serde_json::from_value::<TypicalSampler>(
        data.ok_or(Error::msg("Field must present to specify top_p and temp!"))?,
    )?))
}
```

Where the first param `state` specifies the global state of the `axum` which might be useful
(to things like accessing model info or context). And the second param `data` specifies data
passed from the Websocket API invocation, which can then be used to construct the `Sampler`.

And then, register the `initialize_typical` at the `new()` of `Samplers`, which is located
[here](/src/states/sampler/mod.rs):

```rust
pub fn new() -> Self {
    Samplers {
        registry: hashmap_ex! {
            HashMap<&'static str, fn(AppState, Option<Value>) -> Result<Box<dyn Sampler>>>,
                {
                    // The `"typical"` specified will be the `type_id` for the API to construct the `Sampler`.
                    "typical" => typical::initialize_typical
                }
        },
        map: DashMap::with_capacity(128),
    }
}
```
