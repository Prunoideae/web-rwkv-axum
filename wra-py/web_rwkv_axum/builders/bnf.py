from dataclasses import dataclass, is_dataclass
from typing import Callable, TypeVar
from ..typed.bnf import RuleSet, Rule

T = TypeVar("T")


class BNFFactory:
    def compile(self, rules: RuleSet, clazz: type) -> Rule:
        """
        Resolves and handles everything.

        Returns a rule that represents the class.
        """

    def deserialize(self, clazz: type[T], payload: str) -> T:
        """
        Construct the type from the payload.
        """


def bnf(factory: BNFFactory) -> Callable[[type[T]], type[T]]:
    def bnf_wrapper(clazz: type[T]):
        if not is_dataclass(clazz):
            clazz = dataclass(clazz)
        setattr(clazz, "__bnf_factory", factory)
        return clazz

    return bnf_wrapper


def compile(clazz: type[T]) -> tuple[str, str]:
    ruleset = RuleSet("__bnf")
    return get_bnf(clazz).compile(ruleset, clazz).rule_id, ruleset.declare()


def deserialize(clazz: type[T], payload: str) -> T:
    return get_bnf(clazz).deserialize(clazz, payload)


def get_bnf(clazz: type[T]) -> BNFFactory:
    return getattr(clazz, "__bnf_factory", None)


def is_bnf(clazz: type[T]) -> bool:
    return get_bnf(clazz) is not None
