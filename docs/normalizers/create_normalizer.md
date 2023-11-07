#

## `create_normalizer`

This command creates a normalizer with an ID, a normalizer type id, and extra params to create a normalizer with specified settings.

The ID must be unique, and it will be the identifier of any subsequent commands related to the normalizer.

If an ID already exists, an error will be returned.

For detailed information about how to create each normalizer, check out [here](/docs/normalizers/types/), or just read the code.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_normalizer",

    "data": {
        // Specify the ID of the normalizer in a JSON string.
        "id": "cfg1",
        "data": {
            // The normalizer type and params needed to construct it.
            // Refer to other docs for more detailed information about
            // all normalizers.
            "type_id": "cfg",
            "params": ...
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
