from typing import Any, TYPE_CHECKING
from ..components.terminals import TerminalBuilder

if TYPE_CHECKING:
    from ..api import Session


class Lengthed(TerminalBuilder):
    def __init__(self, length: int) -> None:
        self.length = length

    def type_id(self) -> str:
        return "lengthed"

    def payload(self) -> Any:
        return {"length": self.length}


class Until(TerminalBuilder):
    def __init__(self, until: str, cap: int = None) -> None:
        self.until = until
        self.cap = cap

    def type_id(self) -> str:
        return "until"

    def payload(self) -> Any:
        return {"until": self.until, "cap": self.cap}
