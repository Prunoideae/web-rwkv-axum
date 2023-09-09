#

## `echo`

`echo` command is mostly used for testing the server. What it does is simple: return the `data` field of the request as-is in the `result` field of the response.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "echo",

    // Can be any valid json value, including null,
    // number, string, array and object. If omitted,
    // `null` will be returned.
    "data": "Lorem ipsum dolor sit amet"
}
```

#### Response

```jsonc
{
    "echo_id": ...,
    "status": "success",
    "duration_ms": ...,

    "result": "Lorem ipsum dolor sit amet"
}
```
