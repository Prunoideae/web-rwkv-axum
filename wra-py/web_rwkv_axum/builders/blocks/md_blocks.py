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
    level_prefix = "#" * (level + 1) + " "
    if pre is None:
        if not rules.defined(f"__bmh_{level}_decl"):
            rules = rules.with_prefix(f"__bmh_{level}")
            return rules.define(f"{prefixed_line(rules, rules.literal(level_prefix))}'\\n'", id="decl")
        else:
            return rules.get(f"__bmh_{level}_decl")
    else:
        prefix = hashstring(str(pre))
        if not rules.defined(f"__bmh_{level}_{prefix}_decl"):
            rules = rules.with_prefix(f"__bmh_{level}_{prefix}")
            return rules.define(f"{prefixed_line(rules, rules.join(rules.literal(level_prefix), pre))}'\\n'", id="decl")
        else:
            return rules.get(f"__bmh_{level}_{prefix}_decl")


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


def unordered_list(rules: RuleSet, bullet: Literal["*", "-"], count: int = 0, pre: str | Rule = None):
    """
    An unordered list.

    Defined as:
    ```text
    <list_item>
    <list_item>
    ...
    ```
    or
    ```text
    <list_item>
    ... for count time
    ```
    """

    bullet_id = 0 if bullet == "*" else 1

    if pre is None:
        prefix = f"__bmul_{count}_{bullet_id}"
    else:
        prefix = f"__bmul_{count}_{bullet_id}_{hashstring(str(pre))}"

    if not rules.defined(f"{prefix}_decl"):
        rules = rules.with_prefix(prefix)
        if count == 0:
            return rules.define(rules.join(rules.repeat(list_item(rules, bullet, pre)), rules.literal("\n")), id="decl")
        else:
            return rules.define(f"{str(list_item(rules, bullet)) * count}'\\n'", id="decl")
    else:
        return rules.get(f"{prefix}_decl")


def numbered_list(rules: RuleSet, count: int | tuple[int, int] = 0, pre: str | Rule = None):
    """
    A numbered list. Only finite list can have a guarantee on number order.

    Defined as:
    ```text
    <digits>. <line>
    <digits>. <line>
    ...
    ```
    or
    ```text
    1. <line>
    ... for count times.
    ```
    """

    def prefix():
        pre = hashstring(pre)
        if isinstance(count, int):
            if count == 0:
                return f"0_{pre}"
            else:
                return f"0_{count}_{pre}"
        else:
            return f"{count[0]}_{count[1]}_{pre}"

    prefix: str = prefix()

    if not rules.defined(f"__bmnl_{prefix}_decl"):
        rules = rules.with_prefix(f"__bmnl_{prefix}")
        if count == 0:
            digit = rules.define(rules.union(*(rules.literal(x) for x in range(10))))
            digits = rules.repeat(digit)
            if pre is None:
                list_item = prefixed_line(rules, f"{digits}'. '")
            else:
                list_item = rules.define(f"{digits}'. '{pre}")
            return rules.define(f"{rules.repeat(list_item)}'\\n'", id="decl")
        else:
            min, max = (0, count) if isinstance(count, int) else count
            num_list = ""
            for n in range(min, max):
                num_list += f"'{n}. '"
                if pre is None:
                    num_list += str(line(rules))
                else:
                    num_list += str(pre)
            num_list += "'\\n'"
            return rules.define(num_list, id="decl")
    else:
        return rules.get(f"__bmnl_{prefix}_decl")


def itemized_list(rules: RuleSet, items: list[tuple[str | Rule, str | Rule | None]], bullet: Literal["*", "-", "1"]):
    """
    An itemized list. An itemized list is a list that each line has a prefix, and matching a line or other rule.

    If the matcher is not a line, it must be a line ending with `\\n`.

    Defined as:
    ```text
    <bullet> item[0][0]: item[0][1] or <line>
    <bullet> item[1][0]: item[1][1] or <line>
    ...
    """

    def item(pre: str | Rule, end: str | Rule | None):
        if end is None:
            return str(pre)
        else:
            return f"{pre}|{end}"

    prefix = "&".join(item(p, e) for p, e in items)
    if not rules.defined(f"__bmil_{prefix}_decl"):
        rules = rules.with_prefix(f"__bmil_{prefix=}")

        def itemize(pre: str | Rule, end: str | Rule | None, idx: int):
            nonlocal bullet
            if bullet == "1":
                bullet = f"{idx+1}."
            bullet = rules.literal(bullet + " ")
            return rules.join(bullet, pre, end)

        return rules.define(rules.join(*(itemize(p, e, i) for i, (p, e) in enumerate(items))) + "'\\n'", id="decl")

    else:
        return rules.get(f"__bmil_{prefix}_decl")


def paragraph(rules: RuleSet, pre: str | Rule = None):
    """
    A paragraph consists of one or more lines. A prefix can be specified.

    The paragraph ends with double '\\n'.

    Defined as:
    ```text
    <lines>...'\\n'
    ```
    or
    ```text
    <pre><lines>...'\\n'|<pre>'\\n'
    ```
    """
    prefix = hashstring(str(pre))
    if not rules.defined(f"__bmp_{prefix}_decl"):
        rules = rules.with_prefix(f"__{prefix}")
        lines = rules.repeat(line(rules))
        if pre is None:
            return rules.define(f"{lines}'\\n'", id="decl")
        else:
            return rules.define(f"{pre}'\\n'|{pre}{lines}'\\n", id="decl")
    else:
        return rules.get(f"__bmp_{prefix}_decl")
