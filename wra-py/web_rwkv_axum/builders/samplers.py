from typing import Any
from ..components.samplers import SamplerBuilder


class Nucleus(SamplerBuilder):
    def __init__(self, top_p: float = 0.5, temp: float = 1.5) -> None:
        self.top_p = top_p
        self.temp = temp

    def type_id(self) -> str:
        return "nucleus"

    def payload(self) -> Any:
        return {"top_p": self.top_p, "temp": self.temp}
