#

## Sampler Managing

This folder contains commands related to normalizer management, you can create, delete, copy, update or reset a normalizer.

A normalizer is a stateful component which will create a probablity distribution from one or more logits distributions. Due to the *non-blocking* design of the inference pipeline, a normalizer will hold its state between individual inference requests, so you can continue to infer without specifying a lot of params after first inference request is done.
