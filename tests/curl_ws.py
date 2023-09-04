# Handmade Websocket client to test if axum is running well
# Code might change at anytime

# Requires websockets, of course
from websockets.client import connect, WebSocketClientProtocol
from time import time
import asyncio
import json

queries = 100000
payload = {
    "echo_id": None,
    "command": "echo",
    "data": "The red is sus",
}

uri = "ws://127.0.0.1:5678/ws"


async def worker(ws: WebSocketClientProtocol, echo_id: str) -> float:
    local_payload = {
        "echo_id": echo_id,
        "command": "echo",
        "data": f"The red is {echo_id}",
    }
    j = json.dumps(local_payload)
    start = time()
    await ws.send(j)
    return time() - start


async def main():
    async with connect(uri) as ws:
        workers = [worker(ws, f"echo_{i}") for i in range(queries)]
        results: list[float] = await asyncio.gather(*workers)
        start_recv = time()
        for _ in range(queries):
            await ws.recv()
        elapsed = sum(results) + time() - start_recv
        print(f"Total time: {elapsed:.2f}s, qps: {queries/elapsed:.2f}")


if __name__ == "__main__":
    asyncio.run(main())
