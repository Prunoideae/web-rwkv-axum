import ujson
import asyncio
from types import TracebackType
from typing import Any, Optional, Type, TypeVar
from .model import Response
from .components.states import States
from .components.pipeline import Pipelines
from websockets.client import connect, WebSocketClientProtocol
from random import randint

T = TypeVar("T")


class Session:
    """
    A session for calling the web-rwkv-axum api.
    """

    _ws: WebSocketClientProtocol
    _echoes: dict[str, asyncio.Event]

    def __init__(self, uri: str) -> None:
        self._ws = None
        self._echoes = {}
        self._task = None
        self.uri = uri

        # APIs
        self.states = States(self)
        self.pipeline = Pipelines(self)

    async def connect(self) -> "Session":
        if self._ws is not None:
            raise RuntimeError("Already connected to the server!")

        self._ws = await connect(self.uri)
        self._task = asyncio.create_task(self._listen())
        return self

    async def close(self):
        await asyncio.gather(
            self.states.close(),
            self.pipeline.close(),
        )
        await self._ws.close()
        self._ws = None

    async def call(self, command: str, payload: Any) -> Response[T]:
        if self._ws is None:
            raise RuntimeError("Not connected to server yet!")

        echo_id = str(randint(0, 2**31))
        while echo_id in self._echoes:
            echo_id = str(randint(0, 2**31))
        event = asyncio.Event()
        self._echoes[echo_id] = event
        await self._ws.send(
            ujson.dumps({"echo_id": echo_id, "command": command, "data": payload})
        )
        await event.wait()
        self._echoes.pop(echo_id)
        return getattr(event, "__response")

    async def __aenter__(self) -> "Session":
        return await self.connect()

    async def __aexit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_value: Optional[BaseException],
        traceback: Optional[TracebackType],
    ) -> None:
        await self.close()

    async def _listen(self):
        try:
            async for message in self._ws:
                response = Response.from_json(message)
                if (event := self._echoes.get(response.echo_id)) != None:
                    setattr(event, "__response", response)
                    event.set()
        except:
            ...
