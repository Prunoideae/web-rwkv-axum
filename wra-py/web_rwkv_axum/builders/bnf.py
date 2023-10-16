from typing import Callable, TypeVar, final
import inspect
from ..typed.bnf import RuleSet

T = TypeVar("T")


class BNFFactory:
    def compile(self, rules: RuleSet, clazz: type) -> tuple[str, dict[str, str]]:
        """
        Resolves and handles everything.

        Returns a ruleset that represents a certain grammar.
        """

    def deserialize(self, clazz: type[T], payload: str) -> T:
        """
        Construct the type from the payload.
        """


def bnf(factory: BNFFactory) -> Callable[[type[T]], type[T]]:
    def bnf_wrapper(clazz: type[T]):
        setattr(clazz, "__bnf_factory", factory)
        return clazz

    return bnf_wrapper


def compile(clazz: type[T]) -> tuple[str, str]:
    return getattr(clazz, "__bnf_factory").compile(RuleSet("__bnf"), clazz)


def is_bnf(clazz: type[T]) -> bool:
    return getattr(clazz, "__bnf_factory", None) is not None


def deserialize(clazz: type[T], payload: str) -> T:
    if not hasattr(clazz, "__bnf_parse"):
        raise TypeError("Not a decorated type!")
    return getattr(clazz, "__bnf_parse")(payload)
