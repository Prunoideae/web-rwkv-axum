#

## `copy_pipeline`

This commands copies a pipeline to create a new pipeline with the ID specified.

If the source doesn't exist, or the destination already exists, an error will be returned.

The copy will also copy the internal state of the pipeline, so things like penalties etc will be copied to the new pipeline.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "copy_pipeline",

    "data": {
        "source": "pipeline1_backup",
        "destination": "pipeline1",
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
