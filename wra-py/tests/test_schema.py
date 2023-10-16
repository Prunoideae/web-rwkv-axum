from typing import Literal
from web_rwkv_axum.builders.bnf_schema import JsonFactory
from web_rwkv_axum.builders import bnf
from web_rwkv_axum.typed.bnf import RuleSet

from dataclasses import dataclass


@bnf.bnf(JsonFactory())
@dataclass
class Event:
    dates: list[str]
    place: Literal[1] | Literal[2] | Literal["42"]
    sequence: tuple[str, Literal[1] | Literal[2], int]


start, bnf_string = bnf.compile(Event)
# event = bnf.deserialize(Event, """{"date":"2013-10-20","place":"home"}""")

print(bnf_string)

print("Class entry point:", start)
