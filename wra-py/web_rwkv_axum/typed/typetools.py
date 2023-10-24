from typing import Any, Callable, get_origin, TypeVar
from ..typed.bnf import Rule, RuleSet
from dataclasses import Field, is_dataclass, fields
from inspect import getmembers, isroutine

T = TypeVar("T")


def is_shown(fields: dict[str, Field], name: str):
    return name not in fields or fields[name].init


class RuleReader:
    def __init__(
        self,
        rules: RuleSet,
        handlers: dict[type, Callable[["RuleReader", type], Rule]],
        value_handlers: dict[type[T], Callable[["RuleReader", T], Rule]],
        default: Callable[["RuleReader", type], Rule],
    ) -> None:
        self.rules = rules
        self.handled = handlers
        self.handled_override = value_handlers
        self.default = default

    def handle(self, type: type) -> Rule:
        if type in self.handled:
            return self.handled[type](self, type)
        elif (origin := get_origin(type)) in self.handled:
            return self.handled[origin](self, type)
        else:
            return self.default(self, type)

    def read_class(self, clazz: type) -> dict[str, Rule]:
        """
        Reads annotations and processes them into rules.

        Since we don't know how to handle the class itself, we return
        the mapping so you can handle it.

        Fields having `field(init=False)` from `dataclasses` are ignored.
        """
        class_fields = {x.name: x for x in fields(clazz)} if is_dataclass(clazz) else {}
        rules = {}
        for k, v in clazz.__annotations__.items():
            if is_shown(class_fields, k):
                if (field := class_fields[k]) and "bnf_override" in field.metadata and (override_type := type(override := field.metadata["bnf_override"])) in self.handled_override:
                    rules[k] = self.handled_override[override_type](self, override)
                else:
                    rules[k] = self.handle(v)

        return rules


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
        class_fields = {x.name: x for x in fields(type)} if is_dataclass(type) else {}
        return {k: self.handle(payload[k], v) for k, v in type.__annotations__.items() if is_shown(class_fields, k)}
