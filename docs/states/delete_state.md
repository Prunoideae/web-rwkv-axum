#

## `delete_state`

This command deletes an existing state with the ID.

If the state ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "delete_state",

    // Specify the ID of the state in a JSON string.
    "data": "infer_state_1"
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
