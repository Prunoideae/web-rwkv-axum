#

## `copy_normalizer`

This command copies a normalizer to create a new normalizer with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_normalizer",

    "data": {
        "source": "normalizer1_backup",
        "destination": "normalizer1",
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
