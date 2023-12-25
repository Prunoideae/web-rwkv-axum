#

## `reset_pipeline`

This command resets a pipeline's internal state. So, things like recorded penalties or half-finished PDAs will be reset.

If the ID is not present, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "reset_pipeline",

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
