# Handmade Websocket client to test if axum is running well
# Code might change at anytime

# Requires websockets, of course
from typing import Any
from websockets.client import connect, WebSocketClientProtocol
from time import time
import asyncio
import json
from random import randint
import sys

prompt = " ".join(sys.argv[1:])
uri = "ws://127.0.0.1:5678/ws"
state_name = str(randint(0, 2**31))
sampler_name = str(randint(0, 2**31))


async def invoke_command(ws: WebSocketClientProtocol, command: str, payload: Any):
    echo_id = str(randint(0, 2**31))
    payload = {"echo_id": echo_id, "command": command, "data": payload}
    await ws.send(json.dumps(payload))
    result = json.loads(await ws.recv())
    return result


commands = [
    ["echo", "sus"],
    ["create_state", state_name],
    [
        "create_sampler",
        {
            "id": sampler_name,
            "data": {
                "type_id": "typical",
                "params": {
                    "temp": 0.6,
                    "top_p": 0.6,
                },
            },
        },
    ],
    [
        "infer",
        {
            "tokens": [prompt],
            "states": [state_name],
            "transformers": [[]],
            "sampler": sampler_name,
            "update_prompt": True,
        },
    ],
]

payload = {}

repeats = 1000


async def main():
    async with connect(uri) as ws:
        for command, payload in commands:
            result = await invoke_command(ws, command, payload)
            if "error" not in result:
                result = result["result"]
                print(result, flush=True)
            else:
                print(result)

        elapsed = 0
        result = result['last_token']
        for i in range(repeats):
            data = {
                "tokens": None,
                "states": [state_name],
                "transformers": [[]],
                "sampler": sampler_name,
                "update_prompt": True,
            }
            data["tokens"] = [[result]]
            result = await invoke_command(ws, "infer", data)
            elapsed += result["duration_ms"]
            output = result["result"]['value']
            result = result["result"]['last_token']
            print(output, flush=True, end="")

        print(f"\nEnded in {(elapsed/1000):.2f}s, tps: {(repeats/(elapsed/1000)):.2f}")

        await invoke_command(ws, "delete_state", state_name)
        await invoke_command(ws, "delete_sampler", sampler_name)


if __name__ == "__main__":
    asyncio.run(main())
