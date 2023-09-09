#

## `create_state`

This command creates a state with a ID specified.

The ID must be unique, and it will be the identifier of any subsequent commands related to the state.

If an ID already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_state",

    // Specify the ID of the state in a JSON string.
    // There's no limitation of the string, as long as you
    // can handle it.
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
