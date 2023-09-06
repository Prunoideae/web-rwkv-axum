# Handmade Websocket client to test if axum is running well
# Code might change at anytime

# Requires websockets, of course
from typing import Any
from websockets.client import connect, WebSocketClientProtocol
from time import time
import asyncio
import json
from random import randint


uri = "ws://127.0.0.1:5678/ws"


async def invoke_command(ws: WebSocketClientProtocol, command: str, payload: Any):
    echo_id = str(randint(0, 2**31))
    payload = {"echo_id": echo_id, "command": command, "data": payload}
    await ws.send(json.dumps(payload))
    start = time()
    result = json.loads(await ws.recv())
    elapsed = time() - start
    print(f"\n{elapsed*1000:.1f}ms, tps: {1/elapsed}")
    return result


commands = [
    ["echo", "sus"],
    ["create_state", "sussy_baka"],
    [
        "create_sampler",
        {
            "id": "sampler_test",
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
            "tokens": ["lorem ipsum dolor sit amet, con"],
            "states": ["sussy_baka"],
            "transformers": [[]],
            "sampler": "sampler_test",
            "update_prompt": True,
        },
    ],
]

payload = {}

repeats = 500


async def main():
    async with connect(uri) as ws:
        for command, payload in commands:
            result = await invoke_command(ws, command, payload)
            if "error" not in result:
                result = result["result"]
                print(result, flush=True, end="")
            else:
                print(result)

        for i in range(repeats):
            data = {
                "tokens": None,
                "states": ["sussy_baka"],
                "transformers": [[]],
                "sampler": "sampler_test",
                "update_prompt": True,
            }
            data["tokens"] = [result]
            result = (await invoke_command(ws, "infer", data))["result"]
            print(result, flush=True, end="")

        await invoke_command(ws, "delete_state", "sussy_baka")
        await invoke_command(ws, "delete_sampler", "sampler_test")


if __name__ == "__main__":
    asyncio.run(main())
