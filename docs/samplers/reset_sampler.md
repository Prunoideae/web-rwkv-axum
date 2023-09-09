#

## `reset_sampler`

This command resets an existing sampler with the ID.

If the sampler ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "reset_sampler",

    // Specify the ID of the sampler in a JSON string.
    "data": "sampler_1"
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
