#

## Pipeline API

To start inference on `web-rwkv-axum`, you need to construct a pipeline first. A pipeline describes how your inference should be handled, like you might want to disable a certain token, use some kind of sampling method with specified parameters, or let the output to be following a predefined grammar.

A pipeline contains 4 components, which are:

- `Transformer`: transforms a logits distribution from one to another.
- `Normalizer`: normalizes the logits into a probability distribution of tokens. (By default `softmax`).
- `Sampler`: samples a token from the probs distribution.
- `Terminal` determines if an inference should be ended by a given condition (e.g. inferred N tokens, or a certain token is generated).

All components are owned by the pipeline, so you *can not* modify any of them after the pipeline is built, however, you can always create more pipelines.

A pipeline and its components is stateful. So, the pipeline will retain its state between infer requests. If you have a penalty set up, and inferred some tokens, then next inference you request will start with the previously accumulated penalties. You can always copy a pipeline from an original pipeline, or reset a pipeline's state, however.
