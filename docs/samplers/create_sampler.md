#

## `create_sampler`

This command creates a sampler with an ID, a sampler type id, and extra params to create a sampler with specified settings.

The ID must be unique, and it will be the identifier of any subsequent commands related to the sampler.

If an ID already exists, an error will be returned.

For detailed information about how to create each sampler, check out [here](/docs/samplers/types/), or just read the code.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_sampler",

    "data": {
        // Specify the ID of the sampler in a JSON string.
        "id": "typical_1",
        "data": {
            // The sampler type and params needed to construct it.
            // Refer to other docs for more detailed information about
            // all samplers.
            "type_id": "typical",
            "params":{
                "temp": 2.5,
                "top_p": 0.6
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
