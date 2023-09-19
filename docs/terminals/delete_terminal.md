#

## `delete_terminal`

This command deletes an existing terminal with the ID.

If the terminal ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "delete_terminal",

    // Specify the ID of the terminal in a JSON string.
    "data": "infer_terminal_1"
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
