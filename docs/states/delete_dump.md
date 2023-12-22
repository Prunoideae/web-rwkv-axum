#

## `delete_dump`

This command deletes an existing dump with the ID.

If the dump ID is not present in the server, an error will be returned.

## Example

#### Request

```jsonc
{
    "echo_id": ...,
    "command": "delete_dump",

    // Specify the ID of the dump in a JSON string.
    "data": "infer_dump_1"
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
