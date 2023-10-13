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
