#

## `copy_sampler`

This command copies a sampler to create a new sampler with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_sampler",

    "data": {
        "source": "sampler1_backup",
        "destination": "sampler1",
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
