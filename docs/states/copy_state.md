#

## `copy_state`

This command copies a state to create a new state with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

This command is `synced`, which means that it will force a download from the pooled GPU memory (if there is any) to ensure that the state copied is fresh.

By setting `shallow` to `true`, you can duplicate the state without copying its content, thus you create a reference to the state. The reference is on the same level as the original state, so if you update the state through the reference, all references holding the state will be updated.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_state",

    "data": {
        "source": "state1_backup",
        "destination": "state1",
        "shallow": false,
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
