from dataclasses import field
from typing import TypeVar

T = TypeVar("T")


def wrap_type(type: T) -> T:
    def wrapper(*args, **kwargs):
        obj = type(*args, **kwargs)
        return field(metadata={"bnf_override": obj})

    setattr(wrapper, "__wrapped", type)
    return wrapper


def unwrap(type: T) -> T:
    return getattr(type, "__wrapped")


@wrap_type
class Time:
    """
    Marker class for a time-like object matching.

    Usage:
    ```python
    field: str = Time("%%%%-%%-%% %%:%%:%%")
    ```

    Any `%` in the field will be treated as a number from 0 to 9.
    """

    def __init__(self, time: str) -> None:
        self.time = time
