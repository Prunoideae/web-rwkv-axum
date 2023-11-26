#

## `delete_pipeline`

This command deletes an existing pipeline with the ID.

If the pipeline ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "delete_pipeline",

    // Specify the ID of the pipeline in a JSON string.
    "data": "infer_pipeline_1"
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
