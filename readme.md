# web-rwkv-axum

A `axum` web backend for `web-rwkv`, built on `websocket`.

Supports BNF-constrained grammar, CFG sampling, etc., all streamed over network.

Still under heavy development, PRs and suggestions are welcome.

## Testing

- Run by `RUSTFLAGS="--cfg tokio_unstable" cargo run --release ./config.toml`. Wait for `Model is loaded!` to popup.
- Run the `/tests/curl_ws.py "{any prompt input}"` in the `tests` folder.
- Or, with now-implemented (but not published yet) Python API:
  - Build and install the package by running `python setup.py build && python setup.py install` in `wra-py`
  - Run the `tests/test_pipeline.py` and check the code.

## Logging

- By default log messages are sent to stderr with level=info.
- To change the level, set environment variable by `RUST_LOG=<level>`.
  - Possible levels are `debug,info,warning,error,off`.

## Protocol

Since it's built based on `websocket`, and supports highly varied pipeline customizations including complex logits transformations and sampling methods, `web-rwkv-axum` is built on a new protocol.

For specification, please refer to the `docs` folder.
