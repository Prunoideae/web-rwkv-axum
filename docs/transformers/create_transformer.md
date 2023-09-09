#

## `create_transformer`

This command creates a transformer with an ID, a transformer type id, and extra params to create a transformer with specified settings.

The ID must be unique, and it will be the identifier of any subsequent commands related to the transformer.

If an ID already exists, an error will be returned.

For detailed information about how to create each transformer, check out [here](/docs/transformers/types/), or just read the code.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_transformer",

    "data": {
        // Specify the ID of the transformer in a JSON string.
        "id": "global_1",
        "data": {
            // The transformer type and params needed to construct it.
            // Refer to other docs for more detailed information about
            // all transformers.
            "type_id": "global_penalty",
            "params":{
                "alpha_presence": 0.3,
                "alpha_occurrence": 0.3
            }
        }
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
