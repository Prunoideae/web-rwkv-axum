#

## `delete_normalizer`

This command deletes an existing normalizer with the ID.

If the normalizer ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "delete_normalizer",

    // Specify the ID of the normalizer in a JSON string.
    "data": "normalizer_1"
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
