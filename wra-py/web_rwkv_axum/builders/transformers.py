from typing import Any
from ..components.transformers import TransformerBuilder
from ..typed.bnf import RuleSet, Rule
from ..builders.bnf import is_bnf, compile, deserialize


class BNFTransformer(TransformerBuilder):
    def __init__(self, start: str | Rule, rules: str | RuleSet) -> None:
        if isinstance(start, Rule):
            start = start.rule_id
        if isinstance(rules, RuleSet):
            rules = rules.declare()
        self.start = start
        self.rules = rules
        self.stack_arena_capacity = 1024
        self.grammar_stack_areana_capacity = 1048576
        self.bytes_cache = True

    def arena_capacity(self, cap: int):
        self.stack_arena_capacity = cap

    def gramma_arena_capacity(self, cap: int):
        self.grammar_stack_areana_capacity = cap

    def enable_bytes_cache(self, flag: bool = True):
        self.bytes_cache = flag

    def type_id(self) -> str:
        return "bnf_grammar"

    def payload(self) -> Any:
        return {
            "grammar": self.rules,
            "start_nonterminal": self.start,
            "grammar_stack_arena_capacity": self.grammar_stack_areana_capacity,
            "stack_arena_capacity": self.stack_arena_capacity,
            "stack_to_bytes_cache_enabled": self.bytes_cache,
        }


class SchemaBNF(BNFTransformer):
    def __init__(self, clazz: type) -> None:
        if not is_bnf(clazz):
            raise TypeError("Class is not a BNF schema class!")
        start, rules = compile(clazz)
        super().__init__(start, rules)
        self.clazz = clazz

    def deserialize(self, payload: str) -> Any:
        return deserialize(self.clazz, payload)


class GlobalPenalty(TransformerBuilder):
    def __init__(self, alpha_occurrence: float = 0.3, alpha_presence: float = 0.3) -> None:
        self.alpha_occurrence = alpha_occurrence
        self.alpha_presence = alpha_presence

    def type_id(self) -> str:
        return "global_penalty"

    def payload(self) -> Any:
        return {
            "alpha_occurrence": self.alpha_occurrence,
            "alpha_presence": self.alpha_presence,
        }


class SlidingPenalty(TransformerBuilder):
    def __init__(
        self,
        alpha_occurrence: float = 0.3,
        alpha_presence: float = 0.3,
        window_size: int = 128,
    ) -> None:
        self.alpha_occurrence = alpha_occurrence
        self.alpha_presence = alpha_presence
        self.window_size = window_size

    def type_id(self) -> str:
        return "sliding_penalty"

    def payload(self) -> Any:
        return {
            "alpha_occurrence": self.alpha_occurrence,
            "alpha_presence": self.alpha_presence,
            "window_size": self.window_size,
        }


class DisableToken(TransformerBuilder):
    def __init__(self, tokens: list[int]) -> None:
        self.tokens = tokens

    def type_id(self) -> str:
        return "disable_token"

    def payload(self) -> Any:
        return {"tokens": self.tokens}
