#

## `create_terminal`

This command creates a terminal with an ID specified.

The ID must be unique, and it will be the identifier of any subsequent commands related to the terminal.

If an ID already exists, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_terminal",

    // Specify the ID of the terminal in a JSON string.
    // There's no limitation of the string, as long as you
    // can handle it.
    "data": {
        "id": "terminal_1",
        "data": {
            "type_id": "lengthed",
            "params": {
                "length": 128,
            },
        },
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
