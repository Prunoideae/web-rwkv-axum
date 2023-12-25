#

## `dump_state`

This command dumps a state to a dump on server storage.

Dumps can be overriden by dumping on a same dump id.

If the state ID does not exist, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "dump_state",

    // Specify the ID of the state in a JSON string.
    // There's no limitation of the string, as long as you
    // can handle it.
    "data": {
        "state_id": "infer_state_1",
        "dump_id": "dump_id_1"
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
