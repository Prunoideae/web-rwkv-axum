#

## `update_state`

This command updates one or more states with a list of tokens.

If any of the state ID is not present in the server, or any error occurred in tokenization/inference, an error will be returned.

Tokens can either be a string or a list of integers. Each set of tokens will be fed into the corresponding state.

There is no sampling or other process done on the process, so nothing will be returned. If you want to have some tokens, you will need to build a pipeline and start infer via [the infer command](/docs/infer/infer.md).

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "update_state",

    // Each state id will receive tokens at a same index.
    "data": {
        "states": ["state1", "state2", "state3"],
        "tokens": ["state1_token", [114, 514], "state3_token"]
    }
}
```

#### Response

```jsonc
{
    "echo_id": ...,
    "status": "success",
    "duration_ms": ...,

    // If the command is successful, `null` will be returned.
    "result": null
}
```
