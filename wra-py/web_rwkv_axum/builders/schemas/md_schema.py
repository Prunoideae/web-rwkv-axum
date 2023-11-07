import hashlib

from mistune import create_markdown
from typing import TypeVar
from ...typed.bnf import RuleSet, Rule
from ..bnf import BNFFactory

T = TypeVar("T")

markdown = create_markdown(renderer=None)


def hashstring(s: str) -> str:
    h = hashlib.new("sha256")
    h.update(s.encode())
    return h.hexdigest()[:16]


class MarkdownFactory(BNFFactory):
    @staticmethod
    def line(rules: RuleSet) -> Rule:
        """
        Unconstrained markdown line.
        """

    @staticmethod
    def line_pre(rules: RuleSet, pre: str | Rule = None) -> Rule:
        """
        Constrained line.

        Defined as:
        ```text
        "pre"<line>
        ```
        """
        line = MarkdownFactory.line(rules)
        return rules.define(f"{pre}{line}")

    @staticmethod
    def heading(rules: RuleSet, level: int = 0, pre: str | Rule = None):
        line = MarkdownFactory.line(rules) if pre is None else MarkdownFactory.line_pre(rules, pre)

    def compile(self, rules: RuleSet, clazz: type) -> Rule:
        return super().compile(rules, clazz)

    def deserialize(self, clazz: type[T], payload: str) -> T:
        raise NotImplementedError("No, I don't wanna.")
