#

## `update_sampler`

This command updates a sampler with a list of tokens.

If the sampler ID does not exist, or any error occurred in tokenization/update, an error will be returned.

Tokens can either be a string or a list of integers.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "update_sampler",

    "data": {
        "sampler": "sampler1",
        "tokens": "lorem ipsum dolor sit amet"
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
