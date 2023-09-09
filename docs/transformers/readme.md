#

## Transformer Managing

A `Transformer` in the `web-rwkv-axum` is *not that transformer* known by many people. The full name of it is the `Logits Transformer`, and what it does is to transform a logits distribution to another, right before the `softmax` is applied.

The major usage of it is to alter the distribution of token probabilities without the worry of re-normalizing. You can easily disable a token by setting it to `negative infinity`, or increase the token probability by adding some value - and `softmax` will handle everything after the transformation is done.

The `Transformer` is stateful due to the *non-blocking* design of the inference pipeline, so it will keep its internal state between different inference requests. And by it you can continue to infer without specifying a lot of params after first inference request is done.

A transformer might also be **exhausted**. An exhaustion means that the transformer perceives that it will *deny* all tokens in the next transformation, leading to an early termination of inference. But usually you don't need to worry in inference, when a transformer is exhausted in an inference pipeline, it will automatically reset itself by default (though still terminates the inference process, of course).
