#

## `reset_transformer`

This command resets an existing transformer with the ID.

If the transformer ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "reset_transformer",

    // Specify the ID of the transformer in a JSON string.
    "data": "transformer_1"
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
