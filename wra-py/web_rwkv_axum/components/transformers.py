from typing import TYPE_CHECKING, Any
from random import randint
from ..helper import get_random

if TYPE_CHECKING:
    from ..api import Session


class TransformerBuilder:
    def type_id(self) -> str:
        """
        The type_id that this builder will use
        """

    def payload(self) -> Any:
        """
        Create the payload used by the transformer
        """


class Transformer:
    transformer_id: str

    def __init__(self, transformer_id: str, transformers: "Transformers") -> None:
        self.transformer_id = transformer_id
        self._transformers = transformers

    @property
    def valid(self) -> bool:
        return self.transformer_id in self._transformers._transformers

    async def copy(self, dst_id: str=None) -> "Transformer":
        return await self._transformers.copy_transformer(self, dst_id)

    async def update(self, tokens: int | str | list[int | str]):
        if isinstance(tokens, int):
            tokens = [tokens]
        return await self._transformers.update_transformer(self, tokens)

    async def reset(self):
        return await self._transformers.reset_transformer(self)

    async def delete(self):
        return await self._transformers.delete_transformer(self)


class Transformers:
    def __init__(self, session: "Session") -> None:
        self._session = session
        self._transformers = set()

    async def create_transformer(self, builder: TransformerBuilder, transformer_id: str = None):
        if transformer_id is None:
            transformer_id = get_random(self._transformers)

        if (resp := await self._session.call("create_transformer", {"id": transformer_id, "data": {"type_id": builder.type_id(), "params": builder.payload()}})).success():
            self._transformers.add(transformer_id)
            return Transformer(transformer_id, self)
        else:
            raise RuntimeError(resp.result)

    async def delete_transformer(self, transformer: Transformer):
        if not transformer.valid:
            raise RuntimeError("Transformer id does not exist!")

        if (resp := await self._session.call("delete_transformer", transformer.transformer_id)).success():
            self._transformers.remove(transformer.transformer_id)
        else:
            raise RuntimeError(resp.result)

    async def copy_transformer(self, src: Transformer, dst_id: str = None) -> Transformer:
        if not src.valid:
            raise RuntimeError("Source transformer does not exist!")

        if dst_id is None:
            dst_id = get_random(self._transformers)

        if (resp := await self._session.call("copy_transformer", {"source": src.transformer_id, "destination": dst_id})).success():
            self._transformers.add(dst_id)
            return Transformer(dst_id, self)
        else:
            raise RuntimeError(resp.result)

    async def update_transformer(self, transformer: Transformer, tokens: str | list[int, str]):
        if not transformer.valid:
            raise RuntimeError("Transformer does not exist!")

        if not (response := await self._session.call("update_transformer", {"id": transformer.transformer_id, "tokens": tokens})).success():
            raise RuntimeError(response.result)

    async def reset_transformer(self, transformer: Transformer):
        if not transformer.valid:
            raise RuntimeError("Transformer does not exist!")

        await self._session.call("reset_transformer", transformer.transformer_id)
