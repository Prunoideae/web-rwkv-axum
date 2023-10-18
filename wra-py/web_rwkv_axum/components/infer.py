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

    def payload(self):
        return {"transformers": self.transformers, "sampler": self.sampler}


@dataclass
class InferResult:
    pipeline: "InferPipeline"
    ms_elapsed: int | None
    last_token: int
    result: str
    end_reason: str
    token_count: int

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
        self.token_count = resp.token_count
        return self

    async def copy(self) -> "InferResult":
        pipeline = await self.pipeline.copy()
        return InferResult(
            pipeline=pipeline,
            ms_elapsed=self.ms_elapsed,
            last_token=self.last_token,
            result=self.result,
            end_reason=self.end_reason,
            token_count=self.token_count,
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
        update_prompt: bool = True,
        reset_on_exhaustion: bool | ExhaustionReset = True,
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
                    "transformers": [[t.transformer_id for t in ts] for ts in self.transformers],
                    "sampler": self.sampler.sampler_id,
                    "terminal": self.terminal.terminal_id,
                    "reset_on_exhaustion": reset_on_exhaustion,
                    "update_prompt": update_prompt,
                },
            )
        ).success():
            return InferResult(
                self,
                ms_elapsed=resp.duration_ms,
                last_token=resp.result["last_token"],
                result=resp.result["value"],
                end_reason=resp.result["end_reason"],
                token_count=resp.result["inferred_tokens"],
            )
        else:
            raise RuntimeError(resp.result)

    async def copy(self) -> "InferPipeline":
        async def gather_list(ts: list[Any]) -> list[Any]:
            return asyncio.gather(*[t.copy() for t in ts])

        sampler, terminal, states, *transformers = await asyncio.gather(
            *([self.sampler.copy(), self.terminal.copy(), gather_list(self.states)] + [gather_list(x) for x in self.transformers]),
        )

        return InferPipeline(self._session, states, transformers, sampler, terminal)


class Infers:
    def __init__(self, session: "Session") -> None:
        self._session = session

    def pipeline(self, *args: tuple[State, list[Transformer]], sampler: Sampler, terminal: Terminal) -> InferPipeline:
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
                    raise RuntimeError(f"Transformer {t.transformer_id} does not exist!")
            states.append(arg[0])
            transformers.append(arg[1])

        return InferPipeline(
            self._session,
            states=states,
            transformers=transformers,
            sampler=sampler,
            terminal=terminal,
        )
