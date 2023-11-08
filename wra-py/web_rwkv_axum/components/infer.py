from .samplers import Sampler
from .transformers import Transformer
from .terminals import Terminal
from .states import State
import asyncio
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from ..api import Session


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
    pipeline: "InferPipeline"
    ms_elapsed: int | None
    last_token: int
    result: str
    end_reason: str
    inferred_token: int
    prompt_token: int

    async def continue_(
        self,
        tokens: None | str | list[list[int | str]] = None,
        update_prompt: bool = True,
        reset_on_exhaustion: bool | ExhaustionReset = True,
    ):
        if isinstance(tokens, list):
            if len(tokens) != len(self.pipeline.states):
                raise RuntimeError("Token list size mismatch!")
        if isinstance(tokens, str):
            tokens = [[self.last_token, tokens] for _ in self.pipeline.states]

        if tokens is None:
            tokens = [[self.last_token] for _ in self.pipeline.states]
        resp = await self.pipeline.infer(
            tokens=tokens,
            update_prompt=update_prompt,
            reset_on_exhaustion=reset_on_exhaustion,
        )
        self.ms_elapsed = resp.ms_elapsed
        self.last_token = resp.last_token
        self.end_reason = resp.end_reason
        self.result = resp.result
        self.inferred_token = resp.inferred_token
        return self

    async def copy(self) -> "InferResult":
        pipeline = await self.pipeline.copy()
        return InferResult(
            pipeline=pipeline,
            ms_elapsed=self.ms_elapsed,
            last_token=self.last_token,
            result=self.result,
            end_reason=self.end_reason,
            inferred_token=self.inferred_token,
        )


@dataclass
class InferPipeline:
    _session: "Session"
    states: list[State]
    transformers: list[list[Transformer]]
    sampler: Sampler
    terminal: Terminal

    async def infer(
        self,
        tokens: str | list[list[int | str]],
        *,
        update_prompt: bool = True,
        update_states: bool | list[bool] = True,
        reset_on_exhaustion: bool | ExhaustionReset = True,
        timeout: float = 20,
    ):
        if isinstance(tokens, str):
            tokens = [[tokens] for _ in self.states]

        if isinstance(reset_on_exhaustion, ExhaustionReset):
            reset_on_exhaustion = reset_on_exhaustion.payload()

        if (
            resp := await self._session.call(
                "infer",
                {
                    "tokens": tokens,
                    "states": [s.state_id for s in self.states],
                    "transformers": [
                        [t.transformer_id for t in ts] for ts in self.transformers
                    ],
                    "sampler": self.sampler.sampler_id,
                    "terminal": self.terminal.terminal_id,
                    "reset_on_exhaustion": reset_on_exhaustion,
                    "update_states": update_states,
                    "update_prompt": update_prompt,
                    "timeout": int(timeout * 1000),
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
            )
        else:
            raise RuntimeError(resp.result)

    async def copy(self, shallow=False) -> "InferPipeline":
        async def gather_list(ts: list[Any], *args, **kwargs) -> list[Any]:
            return await asyncio.gather(*[t.copy(*args, **kwargs) for t in ts])

        tasks = [
            self.sampler.copy(),
            self.terminal.copy(),
            gather_list(self.states, shallow=shallow),
        ] + [gather_list(x) for x in self.transformers]

        sampler, terminal, states, *transformers = await asyncio.gather(*tasks)
        return InferPipeline(self._session, states, transformers, sampler, terminal)

    async def close(self):
        await asyncio.gather(
            self.sampler.delete(),
            self.terminal.delete(),
            *(state.delete() for state in self.states),
            *(transformer.delete() for ts in self.transformers for transformer in ts),
        )

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()


class Infers:
    def __init__(self, session: "Session") -> None:
        self._session = session

    def pipeline(
        self,
        *args: tuple[State, list[Transformer]],
        sampler: Sampler,
        terminal: Terminal,
    ) -> InferPipeline:
        states: list[State] = []
        transformers: list[list[Transformer]] = []
        if not sampler.valid:
            raise RuntimeError(f"Sampler {sampler.sampler_id} does not exist!")
        if not terminal.valid:
            raise RuntimeError(f"Terminal {terminal.terminal_id} does not exist!")
        for arg in args:
            if not arg[0].valid:
                raise RuntimeError(f"State {arg[0].state_id} does not exist!")
            for t in arg[1]:
                if not t.valid:
                    raise RuntimeError(
                        f"Transformer {t.transformer_id} does not exist!"
                    )
            states.append(arg[0])
            transformers.append(arg[1])

        return InferPipeline(
            self._session,
            states=states,
            transformers=transformers,
            sampler=sampler,
            terminal=terminal,
        )
