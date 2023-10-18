"""
Function based schema definition of Markdown
"""

import hashlib
from typing import Literal
from ...typed.bnf import RuleSet, Rule


def hashstring(s: str) -> str:
    h = hashlib.new("sha256")
    h.update(s.encode())
    return h.hexdigest()[:16]


def line(rules: RuleSet) -> Rule:
    """
    A line. Note that it will match `- `, `1. `, etc.
    """
    if not rules.defined("__bml_decl"):
        rules = rules.with_prefix("__bml")
        newline = rules.define('"\\n"')
        non_newline = rules.except_(newline)
        line = rules.repeat(non_newline)
        return rules.define(f"{line}'\\n'", id="decl")
    else:
        return rules.get("__bml_decl")


def prefixed_line(rules: RuleSet, pre: str | Rule = None) -> Rule:
    """
    Constrained line.

    Defined as:
    ```text
    <pre><line>
    ```
    """

    if pre is None:
        return line(rules)

    prefix = hashstring(str(pre))
    if not rules.defined(f"__bmlp_{prefix}_decl"):
        rules = rules.with_prefix(f"__bmlp_{prefix}")
        pre = rules.define(pre)
        return rules.define(f"{pre}{line(rules)}", id="decl")
    else:
        return rules.get(f"__bmlp_{prefix}_decl")


def heading(rules: RuleSet, level: int = 0, pre: str | Rule = None):
    """
    A heading.

    Defined as:
    ```text
    "#"*(level+1)<line>
    ```
    """
    level = "#" * level + 1 + " "
    if pre is None:
        return prefixed_line(rules, rules.literal(level))
    else:
        return prefixed_line(rules, rules.join(rules.literal(level), pre))


def list_item(rules: RuleSet, bullet: Literal["*", "-"], pre: str | Rule = None):
    """
    A list item.

    Defined as:
    ```text
    "*  or - "<line>
    ```
    """
    bullet += " "
    if pre is None:
        return prefixed_line(rules, rules.literal(bullet))
    else:
        return prefixed_line(rules, rules.join(rules.literal(bullet), pre))


def unordered_list(rules: RuleSet, bullet: Literal["*", "-"], count: int = 0):
    """
    An unordered list.

    Defined as:
    ```text
    <line>
    <line>
    ...
    ```
    or
    ```text
    <line>
    ... for count time
    ```
    """
    if not rules.defined(f"__bmul_{count}_decl"):
        rules = rules.with_prefix(f"__bmul_{count}")
        if count == 0:
            return rules.define(rules.repeat(list_item(rules, bullet)), id="decl")
        else:
            return rules.define(str(list_item(rules, bullet)) * count, id="decl")
    else:
        return rules.get(f"__bmul_{count}_decl")


def numbered_list(rules: RuleSet, count: int = 0):
    ...


def itemized_list(rules: RuleSet, items: list[tuple[str | Rule, str | Rule | None]], bullet: Literal["*", "-", "1"]):
    ...
