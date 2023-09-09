#

## `copy_transformer`

This command copies a transformer to create a new transformer with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_transformer",

    "data": {
        "source": "transformer1_backup",
        "destination": "transformer1",
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
