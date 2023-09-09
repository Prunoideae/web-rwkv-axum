#

## Sampler Managing

This folder contains commands related to sampler management, you can create, delete, copy, update or reset a sampler.

A sampler is a stateful component which will select one (or a list of) token from one (or a list of) probablity distributions. Due to the *non-blocking* design of the inference pipeline, a sampler will hold its state between individual inference requests, so you can continue to infer without specifying a lot of params after first inference request is done.

A sampler might also be **exhausted**. An exhaustion means that the sampler perceives that it will *NOT* select any token in the next sampling, leading to an early termination of inference. But usually you don't need to worry in inference, when a sampler is exhausted in an inference pipeline, it will automatically reset itself by default (though still terminates the inference process, of course).
