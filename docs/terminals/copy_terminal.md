#

## `copy_terminal`

This command copies a terminal to create a new terminal with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_terminal",

    "data": {
        "source": "terminal1_backup",
        "destination": "terminal1",
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
