from ..bnf import BNFFactory, get_bnf
from typing import Any, Literal, TypeVar, get_args, Union, get_origin
from types import UnionType
from dataclasses import is_dataclass
from ...typed.bnf import RuleSet, Rule
from ...typed.typetools import DeserdeReader, RuleReader
from ...typed.json import Time, unwrap
from ..blocks import json_blocks
import json
import hashlib

T = TypeVar("T")


def hashstring(s: str) -> str:
    h = hashlib.new("sha256")
    h.update(s.encode())
    return h.hexdigest()[:16]


class JsonFactory(BNFFactory):
    @staticmethod
    def null(type_decl: RuleSet) -> Rule:
        if not type_decl.defined("__bjn_decl"):
            return type_decl.with_prefix("__bjn").define(type_decl.literal("null"), id="decl")
        else:
            return type_decl.get("__bjn_decl")

    def compile(self, rules: RuleSet, clazz: type) -> Rule:
        make = lambda f: lambda r, _: f(r.rules)

        def handle_default(reader: RuleReader, type_: type) -> Rule:
            if not isinstance(type_, type):
                raise TypeError("Must be a class to determine annotation types.")
            # default will also handle the clazz itself, so need extra checking
            # to prevent infinite recursion
            if (factory := get_bnf(type_)) is not None and type_ is not clazz:
                return factory.compile(reader.rules, type_)
            return json_blocks.obj(reader.rules, reader.read_class(type_))

        def handle_list(reader: RuleReader, type: type) -> Rule:
            if len(args := get_args(type)) != 1:
                raise TypeError("Unable to determine list type.")
            list_type = args[0]
            return json_blocks.array(reader.rules, reader.handle(list_type))

        def handle_tuple(reader: RuleReader, type: type) -> Rule:
            if not (args := get_args(type)):
                raise TypeError("Unable to determine tuple types.")
            if any(x is ... for x in args):
                raise TypeError("No ... is allowed in a tuple type, use list[type] instead.")
            return json_blocks.items(reader.rules, *map(lambda t: reader.handle(t), args))

        def handle_enum(reader: RuleReader, type: type) -> Rule:
            if not (args := get_args(type)):
                raise TypeError("Unable to determine enum types.")

            if any(get_origin(x) is not Literal for x in args):
                raise TypeError("All items must be only literals.")
            args = [literal for arg in args for literal in get_args(arg)]

            return json_blocks.enum(reader.rules, *[reader.rules.literal(x if not isinstance(x, str) else json.dumps(x)) for x in args])

        def handle_time(reader: RuleReader, obj: Time) -> Rule:
            return json_blocks.time(reader.rules, obj.time)

        reader = RuleReader(
            rules=rules,
            handlers={
                int: make(json_blocks.number),
                float: make(json_blocks.number),
                str: make(json_blocks.string),
                bool: make(json_blocks.boolean),
                list: handle_list,
                tuple: handle_tuple,
                Union: handle_enum,
                UnionType: handle_enum,
            },
            value_handlers={
                unwrap(Time): handle_time,
            },
            default=handle_default,
        )
        return reader.handle(clazz)

    def deserialize(self, clazz: type[T], payload: str) -> T:
        payload: dict[str, Any] = json.loads(payload)

        make = lambda f: lambda _, p, t: f(p)

        def handle_default(reader: DeserdeReader, value: Any, type_: type[T]) -> T:
            if not is_dataclass(type_):
                raise TypeError(f"{type_} must be a dataclass to perform deserialization.")
            if (factory := get_bnf(type_)) is not None and type_ is not clazz:
                return factory.deserialize(type_, value)
            return type_(**reader.read_class(value, type_))

        def handle_list(reader: DeserdeReader, value: Any, type_: type) -> list:
            if not isinstance(value, list):
                raise TypeError(f"Error converting {value} to {type_}. Payload is not a list.")
            if len(args := get_args(type_)) != 1:
                raise TypeError("Unable to determine list type.")
            list_type = args[0]
            return [reader.handle(x, list_type) for x in value]

        def handle_tuple(reader: DeserdeReader, value: Any, type_: type) -> list:
            if not isinstance(value, list):
                raise TypeError(f"Error converting {value} to {type_}. Payload is not a list.")
            if not (args := get_args(type_)):
                raise TypeError("Unable to determine tuple types.")
            if any(x is ... for x in args):
                raise TypeError("No ... is allowed in a tuple type, use list[type] instead.")
            return tuple(reader.handle(v, t) for t, v in zip(args, value))

        def handle_enum(reader: DeserdeReader, value: Any, type_: type):
            if not (args := get_args(type_)):
                raise TypeError("Unable to determine enum types.")
            if any(get_origin(x) is not Literal for x in args):
                raise TypeError("All items must be only literals.")
            values = [literal for arg in args for literal in get_args(arg)]
            if value not in values:
                raise TypeError(f"{value} is not one of the {values}.")
            return value

        reader = DeserdeReader(
            handlers={
                int: make(int),
                str: make(str),
                float: make(float),
                bool: make(bool),
                list: handle_list,
                tuple: handle_tuple,
                Union: handle_enum,
                UnionType: handle_enum,
            },
            default=handle_default,
        )

        return reader.handle(payload, clazz)
