# Handmade Websocket client to test if axum is running well
# Code might change at anytime

# Requires websockets, of course
from typing import Any
from websockets.client import connect, WebSocketClientProtocol
from time import time
import asyncio
import json
from random import randint

queries = 100000
payload = {
    "echo_id": None,
    "command": "echo",
    "data": "The red is sus",
}

uri = "ws://127.0.0.1:5678/ws"


async def invoke_command(ws: WebSocketClientProtocol, command: str, payload: Any):
    echo_id = str(randint(0, 2**31))
    payload = {"echo_id": echo_id, "command": command, "data": payload}
    await ws.send(json.dump(payload))
    return json.loads(await ws.recv())


async def main():
    async with connect(uri) as ws:
        ...


if __name__ == "__main__":
    asyncio.run(main())
