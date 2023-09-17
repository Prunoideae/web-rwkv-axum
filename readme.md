# web-rwkv-axum

A `axum` web backend for `web-rwkv`, built on `websocket`.

Supports BNF-constrained grammar, CFG sampling, etc., all streamed over network.

Still under heavy development, PRs and suggestions are welcome.

## Testing

- Run by `cargo run --release ./config.toml`. Wait for `Model is loaded!` to popup.
- Run the `/tests/curl_ws.py "{any prompt input}"` in the `tests` folder.

## Protocol

Since it's built based on `websocket`, and supports highly varied pipeline customizations including complex logits transformations and sampling methods, `web-rwkv-axum` is built on a new protocol.

For specification, please refer to the `docs` folder.
