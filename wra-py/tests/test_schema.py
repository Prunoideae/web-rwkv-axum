from web_rwkv_axum.builders.schemas.json_schema import JsonFactory
from web_rwkv_axum.builders import bnf
from web_rwkv_axum.typed.json import Time
from web_rwkv_axum.typed.bnf import RuleSet
from web_rwkv_axum.builders.blocks.special import email

from dataclasses import field, dataclass, fields


@bnf.bnf(JsonFactory())
class Event:
    @dataclass
    class Author:
        name: str
        age: str

    first_author: Author
    authors: list[Author] = field(default_factory=list, init=False)
    time_epoch: int
    time: str = Time("%%%%-%%-%%")


payload = {
    "first_author": {
        "name": "Sussy",
        "age": "guess",
    },
    "time_epoch": 114514.1919810,
    "time": "2023-22-22",
}

import json

event = bnf.deserialize(Event, json.dumps(payload))
entry, bnf_string = bnf.compile(Event)
print(event.first_author)
print(event.time_epoch, event.time)
print("Class Entrypoint:", entry)
print(bnf_string)

ruleset = RuleSet("email")
match_email = email(ruleset)
match_all = ruleset.define(f"' '{match_email}'\\n\\n'")
