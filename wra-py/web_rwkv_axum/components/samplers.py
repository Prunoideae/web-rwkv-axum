import asyncio
from typing import TYPE_CHECKING, Any
from ..helper import get_random

if TYPE_CHECKING:
    from ..api import Session


class SamplerBuilder:
    def type_id(self) -> str:
        """
        The type_id that this builder will use
        """

    def payload(self) -> Any:
        """
        Create the payload used by the sampler
        """


class Sampler:
    sampler_id: str

    def __init__(self, sampler_id: str, samplers: "Samplers") -> None:
        self.sampler_id = sampler_id
        self._samplers = samplers

    @property
    def valid(self) -> bool:
        return self.sampler_id in self._samplers._samplers

    async def copy(self, dst_id: str = None) -> "Sampler":
        return await self._samplers.copy_sampler(self, dst_id)

    async def update(self, tokens: int | str | list[int | str]):
        if isinstance(tokens, int):
            tokens = [tokens]
        return await self._samplers.update_sampler(self, tokens)

    async def reset(self):
        return await self._samplers.reset_sampler(self)

    async def delete(self):
        return await self._samplers.delete_sampler(self)


class Samplers:
    def __init__(self, session: "Session") -> None:
        self._session = session
        self._samplers = set()

    async def create_sampler(self, builder: SamplerBuilder, sampler_id: str = None):
        if sampler_id is None:
            sampler_id = get_random(self._samplers)

        if (
            resp := await self._session.call(
                "create_sampler",
                {
                    "id": sampler_id,
                    "data": {"type_id": builder.type_id(), "params": builder.payload()},
                },
            )
        ).success():
            self._samplers.add(sampler_id)
            return Sampler(sampler_id, self)
        else:
            raise RuntimeError(resp.result)

    async def delete_sampler(self, sampler: Sampler):
        if not sampler.valid:
            raise RuntimeError("Sampler id does not exist!")

        await self._session.call("delete_sampler", sampler.sampler_id)
        self._samplers.remove(sampler.sampler_id)

    async def copy_sampler(self, src: Sampler, dst_id: str = None) -> Sampler:
        if not src.valid:
            raise RuntimeError("Source sampler does not exist!")

        if dst_id is None:
            dst_id = get_random(self._samplers)

        if (
            resp := await self._session.call(
                "copy_sampler", {"source": src.sampler_id, "destination": dst_id}
            )
        ).success():
            self._samplers.add(dst_id)
            return Sampler(dst_id, self)
        else:
            raise RuntimeError(resp.result)

    async def update_sampler(self, sampler: Sampler, tokens: str | list[int, str]):
        if not sampler.valid:
            raise RuntimeError("Sampler does not exist!")

        if not (
            response := await self._session.call(
                "update_sampler", {"id": sampler.sampler_id, "tokens": tokens}
            )
        ).success():
            raise RuntimeError(response.result)

    async def reset_sampler(self, sampler: Sampler):
        if not sampler.valid:
            raise RuntimeError("Sampler does not exist!")

        await self._session.call("reset_sampler", sampler.sampler_id)

    async def close(self):
        await asyncio.gather(
            *(
                self._session.call("delete_sampler", sampler)
                for sampler in self._samplers
            )
        )
