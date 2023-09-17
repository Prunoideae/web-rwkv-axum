#

## Terminal Managing

This folder contains commands related to terminal management, you can create, delete or copy a terminal.

A terminal is the stop condition of an inference request. The inference will stop and return current inferred tokens if:

- There's no partial token left. (Inferred tokens can be converted to a string without residue)
- The terminal condition is met.

The terminal might be stateful, which means that its state will persist between different inference requests, but usually it should not.
