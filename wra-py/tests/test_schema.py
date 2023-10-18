from web_rwkv_axum.builders.schemas.json_schema import JsonFactory
from web_rwkv_axum.builders import bnf

from dataclasses import dataclass


@bnf.bnf(JsonFactory())
@dataclass
class Event:
    @dataclass
    class Author:
        name: str
        age: str

    authors: list[Author]
    time_epoch: int


payload = {
    "authors": [
        {
            "name": "Red",
            "age": "sus",
        }
    ],
    "time_epoch": 114514.1919810,
}

import json

event = bnf.deserialize(Event, json.dumps(payload))

print(event.authors)
print(event.time_epoch)
