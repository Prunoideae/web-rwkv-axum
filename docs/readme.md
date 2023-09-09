#

## General Specification

`web-rwkv-axum` uses `Websocket` as the protocol, and it is fully asynced, which means that a client and send out any amount of requests at anytime without blocking, and the server will respond to all requests *without blocking* and *without guarantee of order*.

To ensure that the server responses can match the client requests, all WS API implemented in `web-rwkv-axum` will follow the structure below:

#### Request

```jsonc
{
    // The unique identifier of the invocation payload.
    // The server will attach specified `echo_id` when
    // respond to this command.
    // There must not be two requests with same `echo_id`
    // at a time in a same connection.
    "echo_id": "ID",

    // The id of this command, refer to actual doc of the
    // commands for more information.
    "command": "CommandID",

    // The data that will be used by the command, refer
    // to actual doc of the commands for more information.
    "data": ...
}
```

#### Response

```jsonc
// In case of success:
{
    // The `echo_id` of the command invocation. Client
    // can use it to identify it to be a response to
    // which command
    "echo_id": "ID",

    // The status identifier marking this command is
    // invoked without error, and has a successful
    // response.
    "status": "success",

    // The result of the command, refer to actual docs
    // of the commands for more information.
    "result": ...,

    // The time on `axum` side to process the command,
    // in milliseconds.
    "duration_ms": 114514
}
```

```jsonc
// In case of error:
{
    // The `echo_id` of the invocation, if the request
    // has no `echo_id`, this field will be skipped.
    "echo_id": "ID",
    // The status identifier marking this command has 
    // error during invocation.
    "status": "error",
    // The actual error occurred, usually a string describing
    // what the error is.
    "error": "You didn't install Genshin on the server!"
}
```
