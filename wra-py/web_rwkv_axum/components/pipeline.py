import asyncio
from dataclasses import dataclass
from ..helper import get_random
from typing import TYPE_CHECKING, Any, Union

if TYPE_CHECKING:
    from ..api import Session
    from ..components.states import State
    from ..builders.transformers import TransformerBuilder
    from ..builders.samplers import SamplerBuilder
    from ..builders.terminals import TerminalBuilder


@dataclass
class ExhaustionReset:
    transformers: list[bool]
    sampler: bool
    normalizer: bool

    def payload(self):
        return {
            "transformers": self.transformers,
            "sampler": self.sampler,
            "normalizer": self.normalizer,
        }


@dataclass
class InferResult:
    pipeline: "Pipeline"
    ms_elapsed: int | None
    last_token: int
    result: str
    end_reason: str
    inferred_token: int
    prompt_token: int
    states: list["State"]

    async def continue_(
        self,
        tokens: None | str | list[list[int | str]] = None,
        update_prompt: bool = True,
        reset_on_exhaustion: bool = True,
        timeout: int = 20 * 1000,
        inplace: bool = True,
        use_last_token: bool = True,
    ) -> "InferResult":
        last_token_list = (
            [self.last_token] if self.last_token != 0 and use_last_token else []
        )
        if isinstance(tokens, list):
            tokens = [last_token_list + token for token in tokens]
        if isinstance(tokens, str):
            tokens = [last_token_list + [tokens] for _ in self.states]
        if tokens is None:
            if last_token_list is None:
                raise RuntimeError(
                    "The last token is either 0 or use_last_token is set to False with no token input!"
                )
            tokens = [last_token_list for _ in self.states]

        resp = await self.pipeline.infer(
            self.states,
            tokens,
            update_prompt=update_prompt,
            reset_on_exhaustion=reset_on_exhaustion,
            timeout=timeout,
        )
        if inplace:
            self.ms_elapsed = resp.ms_elapsed
            self.last_token = resp.last_token
            self.end_reason = resp.end_reason
            self.result = resp.result
            self.inferred_token = resp.inferred_token
            return self
        else:
            return resp

    async def close(self, close_states: bool = True):
        if close_states:
            await asyncio.gather(*(state.delete() for state in self.states))
        await self.pipeline.delete()


class Pipeline:
    pipeline_id: str

    def __init__(
        self,
        pipeline_id: str,
        pipelines: "Pipelines",
        state_size: int,
    ) -> None:
        self.pipeline_id = pipeline_id
        self._pipelines = pipelines
        self._state_size = state_size

    @property
    def valid(self) -> bool:
        return self.pipeline_id in self._pipelines._pipelines

    async def copy(self, dst: str = None):
        return await self._pipelines.copy_pipeline(self, dst)

    async def reset(self):
        await self._pipelines.reset_pipeline(self)

    async def delete(self):
        await self._pipelines.delete_pipeline(self)

    async def infer(
        self,
        states: list["State"],
        tokens: str | list[list[int | str]],
        *,
        update_prompt: bool = True,
        reset_on_exhaustion: bool | ExhaustionReset = True,
        timeout: int = 20 * 1000,
    ):
        if isinstance(tokens, str):
            tokens = [[tokens] for _ in range(self._state_size)]
        if isinstance(reset_on_exhaustion, ExhaustionReset):
            reset_on_exhaustion = reset_on_exhaustion.payload()

        if (
            resp := await self._pipelines._session.call(
                "infer",
                {
                    "tokens": tokens,
                    "states": [s.state_id for s in states],
                    "pipeline": self.pipeline_id,
                    "update_prompt": update_prompt,
                    "reset_on_exhaustion": reset_on_exhaustion,
                    "timeout": timeout,
                },
            )
        ).success():
            return InferResult(
                self,
                ms_elapsed=resp.duration_ms,
                last_token=resp.result["last_token"],
                result=resp.result["result"],
                end_reason=resp.result["end_reason"],
                inferred_token=resp.result["inferred_tokens"],
                prompt_token=resp.result["prompt_tokens"],
                states=states,
            )
        else:
            raise RuntimeError(resp.result)


class Pipelines:
    def __init__(self, session: "Session") -> None:
        self._session = session
        self._pipelines = set()

    async def create_pipeline(
        self,
        *,
        pipeline_id: str = None,
        transformers: list[list["TransformerBuilder"]],
        sampler: "SamplerBuilder",
        terminal: "TerminalBuilder",
    ) -> Pipeline:
        def create_payload(
            item: Union[
                "TransformerBuilder",
                "SamplerBuilder",
                "TerminalBuilder",
            ]
        ) -> Any:
            return {
                "type_id": item.type_id(),
                "params": item.payload(),
            }

        transformers = [[create_payload(tf) for tf in tfs] for tfs in transformers]
        sampler = create_payload(sampler)
        terminal = create_payload(terminal)

        if pipeline_id is None:
            pipeline_id = get_random(self._pipelines)

        if (
            resp := await self._session.call(
                "create_pipeline",
                {
                    "id": pipeline_id,
                    "transformers": transformers,
                    "sampler": sampler,
                    "terminal": terminal,
                    "noramlizer": None,
                },
            )
        ).success():
            self._pipelines.add(pipeline_id)
            return Pipeline(pipeline_id, self, len(transformers))
        else:
            raise RuntimeError(resp.result)

    async def delete_pipeline(self, pipeline: Pipeline):
        if not pipeline.valid:
            raise RuntimeError("Pipeline id does not exist!")
        await self._session.call("delete_pipeline", pipeline.pipeline_id)
        self._pipelines.remove(pipeline.pipeline_id)

    async def copy_pipeline(self, src: Pipeline, dst: str = None) -> Pipeline:
        if not src.valid:
            raise RuntimeError("Source pipeline does not exist!")

        if dst is None:
            dst = get_random(self._pipelines)

        if (
            resp := await self._session.call(
                "copy_pipeline", {"source": src.pipeline_id, "destination": dst}
            )
        ).success():
            self._pipelines.add(dst)
            return Pipeline(dst, self, src._state_size)
        else:
            raise RuntimeError(resp.result)

    async def reset_pipeline(self, pipeline: Pipeline):
        if not pipeline.valid:
            raise RuntimeError("Pipeline does not exist!")
        await self._session.call("reset_pipeline", pipeline.pipeline_id)

    async def close(self):
        await asyncio.gather(
            *(
                self._session.call("delete_pipeline", pipeline)
                for pipeline in self._pipelines
            )
        )
