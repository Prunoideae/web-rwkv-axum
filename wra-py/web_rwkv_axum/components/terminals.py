from typing import TYPE_CHECKING, Any
from ..helper import get_random

if TYPE_CHECKING:
    from ..api import Session


class TerminalBuilder:
    def type_id(self) -> str:
        """
        The type_id that this builder will use
        """

    def payload(self) -> Any:
        """
        Create the payload used by the terminal
        """


class Terminal:
    terminal_id: str

    def __init__(self, terminal_id: str, terminals: "Terminals") -> None:
        self.terminal_id = terminal_id
        self._terminals = terminals

    @property
    def valid(self) -> bool:
        return self.terminal_id in self._terminals._terminals

    async def copy(self, dst_id: str = None) -> "Terminal":
        return await self._terminals.copy_terminal(self, dst_id)

    async def update(self, tokens: int | str | list[int | str]):
        if isinstance(tokens, int):
            tokens = [tokens]
        return await self._terminals.update_terminal(self, tokens)

    async def reset(self):
        return await self._terminals.reset_terminal(self)

    async def delete(self):
        return await self._terminals.delete_terminal(self)


class Terminals:
    def __init__(self, session: "Session") -> None:
        self._session = session
        self._terminals = set()

    async def create_terminal(self, builder: TerminalBuilder, terminal_id: str = None):
        if terminal_id is None:
            terminal_id = get_random(self._terminals)

        if (resp := await self._session.call("create_terminal", {"id": terminal_id, "data": {"type_id": builder.type_id(), "params": builder.payload()}})).success():
            self._terminals.add(terminal_id)
            return Terminal(terminal_id, self)
        else:
            raise RuntimeError(resp.result)

    async def delete_terminal(self, terminal: Terminal):
        if not terminal.valid:
            raise RuntimeError("Terminal id does not exist!")

        if (resp := await self._session.call("delete_terminal", terminal.terminal_id)).success():
            self._terminals.remove(terminal.terminal_id)
        else:
            raise RuntimeError(resp.result)

    async def copy_terminal(self, src: Terminal, dst_id: str = None) -> Terminal:
        if not src.valid:
            raise RuntimeError("Source terminal does not exist!")

        if dst_id is None:
            dst_id = get_random(self._terminals)

        if (resp := await self._session.call("copy_terminal", {"source": src.terminal_id, "destination": dst_id})).success():
            self._terminals.add(dst_id)
            return Terminal(dst_id, self)
        else:
            raise RuntimeError(resp.result)

    async def update_terminal(self, terminal: Terminal, tokens: str | list[int, str]):
        if not terminal.valid:
            raise RuntimeError("Terminal does not exist!")

        if not (response := await self._session.call("update_terminal", {"id": terminal.terminal_id, "tokens": tokens})).success():
            raise RuntimeError(response.result)

    async def reset_terminal(self, terminal: Terminal):
        if not terminal.valid:
            raise RuntimeError("Terminal does not exist!")

        await self._session.call("reset_terminal", terminal.terminal_id)
