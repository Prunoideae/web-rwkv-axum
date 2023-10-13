from typing import Any
from ..components.terminals import TerminalBuilder


class Lengthed(TerminalBuilder):
    def __init__(self, length: int) -> None:
        self.length = length

    def type_id(self) -> str:
        return "lengthed"

    def payload(self) -> Any:
        return {"length": self.length}
