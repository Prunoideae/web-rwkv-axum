from functools import cache
import ujson
from typing import Any, Generic, TypeVar
from dataclasses import dataclass

T = TypeVar("T")


@dataclass
class Response(Generic[T]):
    echo_id: str
    status: str
    duration_ms: int
    result: T

    def from_json(json: str) -> "Response[T | str]":
        json = ujson.loads(json)

        return Response(
            echo_id=json["echo_id"],
            status=json["status"],
            duration_ms=json["duration_ms"] if json["status"] == "success" else -1,
            result=json["result"] if json["status"] == "success" else json["error"],
        )

    def success(self) -> bool:
        return self.status == "success"
