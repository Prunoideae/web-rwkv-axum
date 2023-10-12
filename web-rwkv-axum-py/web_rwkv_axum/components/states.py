from random import randint
from typing import TYPE_CHECKING
from dataclasses import dataclass

if TYPE_CHECKING:
    from ..api import Session


class State:
    state_id: str

    def __init__(self, state_id: str, states: "States") -> None:
        self.state_id = state_id
        self._states = states

    @property
    def valid(self) -> bool:
        return self.state_id in self._states._states

    async def delete(self):
        return await self._states.delete_state(self)

    async def copy(self, dst_id: str = None) -> "State":
        return await self._states.copy_state(self, dst_id)

    async def update(self, tokens: str | int | list[str | int]):
        if isinstance(tokens, int):
            tokens = [tokens]
        self._states.update_state([self], [tokens])


class States:
    def __init__(self, session: "Session") -> None:
        self._session = session
        self._states = set()

    def get_random(self) -> str:
        rand = str(randint(0, 2**31))
        while rand in self._states:
            rand = str(randint(0, 2**31))
        return rand

    async def create_state(self, state_id: str = None) -> State:
        if state_id is None:
            state_id = self.get_random()
        if (await self._session.call("create_state", state_id)).success():
            self._states.add(state_id)
            return State(state_id, self)
        else:
            raise RuntimeError("Duplicated state id!")

    async def delete_state(self, state: State):
        if not (await self._session.call("delete_state", state.state_id)).success():
            raise RuntimeError("State id does not exist!")
        else:
            self._states.remove(state.state_id)

    async def copy_state(self, src: State, dst_id: str = None) -> State:
        if src.state_id not in self._states:
            raise RuntimeError("Source state id does not exist!")

        if dst_id is None:
            dst_id = self.get_random()

        if (await self._session.call("copy_state", {"source": src.state_id, "destination": dst_id})).success():
            return State(dst_id, self)

    async def update_state(self, states: list[State], tokens: list[str | list[int | str]]):
        state_ids = []
        for state in states:
            if state.state_id not in self._states:
                raise RuntimeError(f"State {state.state_id} does not exist!")
            state_ids.append(state.state_id)
        if not (response := await self._session.call("update_state", {"states": state_ids, "tokens": tokens})).success():
            raise RuntimeError(response.result)
