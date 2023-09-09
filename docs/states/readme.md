#

## State Managing

This folder contains commands related to state management, you can create, delete, copy or update a state.

A state is a piece of GPU memory which contains the runtime data of a `web-rwkv` session. It's pretty tiny (~2MB in RWKV4, ~32MB in RWKV5) so you should not worry about spamming them.

However, you still need to avoid swapping between different states too much. `web-rwkv` runs inference in a large, continuous GPU memory (usually in 32x or 64x of a model state), so if you want to load an remote state into the memory, or download the state back to the remote GPU memory, it will cost some time.

`web-rwkv-axum` tries to avoid this problem by desyncing the state - it will not swap out the GPU state after the inference is done, but instead wait until a new state comes in, if that state has no other empty slot to occupy. This is effective, but with limitations, which is that you should not make more than pool size concurrent requests, or a severe swapping problem might occur.
