#

## `copy_state`

This command copies a state to create a new state with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

This command is `synced`, which means that it will force a download from the pooled GPU memory (if there is any) to ensure that the state copied is fresh.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_state",

    "data": {
        "source": "state1_backup",
        "destination": "state1",
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
