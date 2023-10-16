from typing import Any
from ..components.transformers import TransformerBuilder


class GlobalPenalty(TransformerBuilder):
    def __init__(self, alpha_occurrence: float = 0.3, alpha_presence: float = 0.3) -> None:
        self.alpha_occurrence = alpha_occurrence
        self.alpha_presence = alpha_presence

    def type_id(self) -> str:
        return "global_penalty"

    def payload(self) -> Any:
        return {
            "alpha_occurrence": self.alpha_occurrence,
            "alpha_presence": self.alpha_presence,
        }


class SlidingPenalty(TransformerBuilder):
    def __init__(
        self,
        alpha_occurrence: float = 0.3,
        alpha_presence: float = 0.3,
        window_size: int = 128,
    ) -> None:
        self.alpha_occurrence = alpha_occurrence
        self.alpha_presence = alpha_presence
        self.window_size = window_size

    def type_id(self) -> str:
        return "sliding_penalty"

    def payload(self) -> Any:
        return {
            "alpha_occurrence": self.alpha_occurrence,
            "alpha_presence": self.alpha_presence,
            "window_size": self.window_size,
        }


class DisableToken(TransformerBuilder):
    def __init__(self, tokens: list[int]) -> None:
        self.tokens = tokens

    def type_id(self) -> str:
        return "disable_token"

    def payload(self) -> Any:
        return {"tokens": self.tokens}


class BNF(TransformerBuilder):
    def __init__(self) -> None:
        raise NotImplementedError()
