#

## `update_transformer`

This command updates a transformer with a list of tokens.

If the transformer ID does not exist, or any error occurred in tokenization/update, an error will be returned.

Tokens can either be a string or a list of integers.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "update_transformer",

    "data": {
        "transformer": "transformer1",
        "tokens": "lorem ipsum dolor sit amet"
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
