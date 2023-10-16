from typing import Callable, get_origin
from ..typed.bnf import Rule, RuleSet


class AnnotationReader:
    def __init__(self, rules: dict[type, Callable[["AnnotationReader", RuleSet, type], Rule]], default: Callable[["AnnotationReader", RuleSet, type], Rule]) -> None:
        self.handled = rules
        self.default = default

    def handle(self, rules: RuleSet, type: type) -> Rule:
        if type in self.handled:
            return self.handled[type](self, rules, type)
        elif (origin := get_origin(type)) in self.handled:
            return self.handled[origin](self, rules, type)
        else:
            return self.default(self, rules, type)
