#

## `modify_pipeline`

This command modifies a pipeline with an ID and a set of actions specified.

If the ID does not exist, an error will be returned.

You need to specify a list of `Action`s along with the ID.

## Actions

An action specify a modification that will be made to the pipeline. Action types are specified via `modification` tag.

Available actions are:

### `replace_transformer`

Replace a transformer at a given position with a newly created transformer from given params.

```jsonc
{
    "modification": "replace_transformer",
    // Type ID of the new transformer you want to create
    "type_id": ...,
    // Params of the new transformer you want to create
    "params": ...,
    // State index of the transformer to be placed into
    "state_index": ...,
    // Transformer index of the transformer to be placed into,
    // if index is larger than the length, transformer will be
    // appended to the chain.
    "transformer_index": ...
}
```

### `replace_sampler`

Replace the sampler with a newly created sampler from given params.

```jsonc
{
    "modification": "replace_sampler",
    // Type ID of the new sampler you want to create
    "type_id": ...,
    // Params of the new sampler you want to create
    "params": ...,
}
```

### `replace_terminal`

Replace the terminal with a newly created terminal from given params.

```jsonc
{
    "modification": "replace_terminal",
    // Type ID of the new terminal you want to create
    "type_id": ...,
    // Params of the new terminal you want to create
    "params": ...,
}
```

### `delete_transformer`

Remove the transformer at a given position.

```jsonc
{
    "modification": "delete_transformer",
    // State index of the transformer to be deleted
    "state_index": ...,
    // Transformer index of the transformer to be deleted,
    // if index is larger than the length, the last transformer 
    // will be deleted.
    "transformer_index": ...
}
```

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "modify_pipeline",
    
    "data": {
        // The ID of the pipeline you want to modify.
        "id": ...,
        // A list of modifications you want to apply to
        // the pipeline.
        "modifications": [
            ...,
            ...,
            ...
        ]
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
