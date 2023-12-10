#

## `create_pipeline`

This command creates a pipeline with an ID and a set of components specified.

The ID must be unique, and it will be the identifier of any subsequent commands related to the pipeline.

If an ID already exists, an error will be returned.

For how to specify each component, check out the corresponding sub-directory. The type id is specified as the document title.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "create_pipeline",
    
    "data":{
        // Specify the ID of the pipeline.
        "id": "pipeline_1",

        // Specifies the transformer used by the pipeline.
        // Each array in it specifies a transformer chain for a
        // state at given index.
        // So, a transformers specified like:
        // [[tf1, tf2], [tf3, tf4]]
        // Will handle the states [s1, s2] like:
        // s1 -> tf1 -> tf2 -> s1 output
        // s2 -> tf3 -> tf4 -> s2 output
        // If the infer state mismatches, an error will be returned.
        "transformers": [
            [
                {
                    // The type id of the component.
                    "type_id": ...,
                    // The payload specifying params of it.
                    "params": ...
                }
            ]
        ],

        // Specifies the sampler used by the pipeline.
        "sampler": {
            // The type id of the component.
            "type_id": ...,
            // The payload specifying params of it.
            "params": ...
        },

        // Specifies the normalizer used by the pipeline.
        // It can be omitted now, and softmax will be used.
        "normalizer": {
            // The type id of the component.
            "type_id": ...,
            // The payload specifying params of it.
            "params": ...
        },

        // Specifies the terminal used by the pipeline.
        "terminal": {
            // The type id of the component.
            "type_id": ...,
            // The payload specifying params of it.
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
