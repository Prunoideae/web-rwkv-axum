from typing import Any, Callable, get_origin, TypeVar
from ..typed.bnf import Rule, RuleSet
from dataclasses import Field, is_dataclass, fields
from inspect import getmembers, isroutine

T = TypeVar("T")


def is_shown(type: type, name: str):
    if not is_dataclass(type):
        return True
    return (field := next(filter(lambda x: x.name == name, fields(type)), None)) is None or field.init


class RuleReader:
    def __init__(
        self,
        rules: RuleSet,
        handlers: dict[type, Callable[["RuleReader", type], Rule]],
        default: Callable[["RuleReader", type], Rule],
    ) -> None:
        self.rules = rules
        self.handled = handlers
        self.default = default

    def handle(self, type: type) -> Rule:
        if type in self.handled:
            return self.handled[type](self, type)
        elif (origin := get_origin(type)) in self.handled:
            return self.handled[origin](self, type)
        else:
            return self.default(self, type)

    def read_class(self, type: type) -> dict[str, Rule]:
        """
        Reads annotations and processes them into rules.

        Since we don't know how to handle the class itself, we return
        the mapping so you can handle it.

        Fields having `field(init=False)` from `dataclasses` are ignored.
        """
        return {k: self.handle(v) for k, v in type.__annotations__.items() if is_shown(type, k)}


class DeserdeReader:
    def __init__(self, handlers: dict[type[T], Callable[["DeserdeReader", Any, type[T]], T]], default: Callable[["DeserdeReader", Any, type[T]], T]) -> None:
        self.handled = handlers
        self.default = default

    def handle(self, payload: Any, type: type[T]) -> T:
        if type in self.handled:
            return self.handled[type](self, payload, type)
        elif (origin := get_origin(type)) in self.handled:
            return self.handled[origin](self, payload, type)
        else:
            return self.default(self, payload, type)

    def read_class(self, payload: dict[str, Any], type: type[T]) -> dict[str, Any]:
        """
        Reads annotations and handle process the payload by each handled type.

        The payload must be a str->Any mapping since class annotations are named.

        Fields having `field(init=False)` from `dataclasses` are ignored.
        """
        return {k: self.handle(payload[k], v) for k, v in type.__annotations__.items() if is_shown(type, k)}
