"""
Function based schema definition of JSON
"""

import hashlib
import json
from ...typed.bnf import RuleSet, Rule


def hashstring(s: str) -> str:
    h = hashlib.new("sha256")
    h.update(s.encode())
    return h.hexdigest()[:16]


def null(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bjn_decl"):
        return type_decl.with_prefix("__bjn").define(type_decl.literal("null"), id="decl")
    else:
        return type_decl.get("__bjn_decl")


def number(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bjnu_decl"):
        type_decl = type_decl.with_prefix("__bjnu")

        exp_indicator = type_decl.define('"e"|"E"')
        onenine = type_decl.define(type_decl.union(*[type_decl.literal(x) for x in range(1, 10)]))
        sign = type_decl.define('"+"|"-"')

        digit = type_decl.define(f'"0"|{onenine}')
        digits = type_decl.repeat(digit)
        integer = type_decl.define(f"{digit}|{onenine}{digits}")
        signed = type_decl.define(f"{integer}|{sign}{integer}")
        exp = type_decl.define(f"{exp_indicator}{signed}")
        fraction = type_decl.define(f'"."{digits}')

        return type_decl.define(
            type_decl.union(
                f"{signed}",
                f"{signed}{exp}",
                f"{signed}{fraction}",
                f"{signed}{fraction}{exp}",
            ),
            id="decl",
        )
    else:
        return type_decl.get("__bjnu_decl")


def number(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bjnu_decl"):
        type_decl = type_decl.with_prefix("__bjnu")

        exp_indicator = type_decl.define('"e"|"E"')
        onenine = type_decl.define(type_decl.union(*[type_decl.literal(x) for x in range(1, 10)]))
        sign = type_decl.define('"+"|"-"')

        digit = type_decl.define(f'"0"|{onenine}')
        digits = type_decl.repeat(digit)
        integer = type_decl.define(f"{digit}|{onenine}{digits}")
        signed = type_decl.define(f"{integer}|{sign}{integer}")
        exp = type_decl.define(f"{exp_indicator}{signed}")
        fraction = type_decl.define(f'"."{digits}')

        return type_decl.define(
            type_decl.union(
                f"{signed}",
                f"{signed}{exp}",
                f"{signed}{fraction}",
                f"{signed}{fraction}{exp}",
            ),
            id="decl",
        )
    else:
        return type_decl.get("__bjnu_decl")


def string(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bjs_decl"):
        type_decl = type_decl.with_prefix("__bjs")
        hex_digit = type_decl.define(type_decl.union(*[type_decl.literal(x) for x in "0123456789ABCDEFabcdef"]))
        hex4digits = type_decl.define(type_decl.join(hex_digit, hex_digit, hex_digit, hex_digit))
        escapes = type_decl.define(
            type_decl.union(*([type_decl.literal(x) for x in "/bfnrt"] + ['"\\\\"', "'\"'"] + [type_decl.join(type_decl.literal("u"), hex4digits)])),
        )
        escaped = type_decl.define(f'"\\\\"{escapes}')

        not_allowed = type_decl.define(
            type_decl.union(
                type_decl.literal("\\n"),
                type_decl.literal("\\t"),
                type_decl.literal("\\r"),
                "'\"'",
                '"\\\\"',
            )
        )
        unescaped = type_decl.define(type_decl.except_(not_allowed))
        character = type_decl.define(type_decl.union(escaped, unescaped))
        characters = type_decl.repeat(character)
        return type_decl.define(type_decl.union(f"'\"'{characters}'\"'", "'\"\"'"), id="decl")
    else:
        return type_decl.get("__bjs_decl")


def boolean(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bjb_decl"):
        type_decl = type_decl.with_prefix("__bjb")
        return type_decl.define('"true"|"false"', id="decl")
    else:
        return type_decl.get("__bjb_decl")


def array(type_decl: RuleSet, inner: Rule) -> Rule:
    obj_hash = hashstring(inner.rule)
    prefix = f"__bja_{obj_hash}"
    if not type_decl.defined(prefix + "_decl"):
        type_decl = type_decl.with_prefix(prefix)
        trail = type_decl.repeat(f'", "{inner}')
        return type_decl.define(
            type_decl.union(
                f'"[]"',
                f'"["{inner}"]"',
                f'"["{inner}{trail}"]"',
            ),
            id="decl",
        )
    else:
        return type_decl.get(prefix + "_decl")


def items(type_decl: RuleSet, *rules: Rule) -> Rule:
    obj_hash = hashstring("[".join(map(lambda rule: rule.rule, rules)))
    if not type_decl.defined(f"__bji_{obj_hash}_decl"):
        type_decl = type_decl.with_prefix(f"__bji_{obj_hash}")
        rules: list[Rule] = list(rules)
        sequence: list[Rule | str] = ['"["', rules.pop(0)]
        for rule in rules:
            sequence.append('", "')
            sequence.append(rule)
        sequence.append('"]"')
        return type_decl.define(type_decl.join(*sequence), id="decl")
    else:
        return type_decl.get(f"__bji_{obj_hash}_decl")


def enum(type_decl: RuleSet, *items: Rule | str) -> Rule:
    obj_hash = hashstring("|".join(map(str, items)))
    if not type_decl.defined(f"__bje_{obj_hash}_decl"):
        type_decl = type_decl.with_prefix(f"__bje_{obj_hash}")
        return type_decl.define(type_decl.union(*items), id="decl")
    else:
        return type_decl.get(f"__bje_{obj_hash}_decl")


def obj(type_decl: RuleSet, items: dict[str, Rule | str]) -> Rule:
    obj_hash = hashstring(",".join(f"{k}{v}" for k, v in items.items()))
    if not type_decl.defined(f"__bjo_{obj_hash}_decl"):
        type_decl = type_decl.with_prefix(f"__bjo_{obj_hash}")
        sequence = ['"{"']
        for k, v in items.items():
            k = json.dumps(k).replace("'", "\\'")
            if len(sequence) > 1:
                k = ", " + k
            sequence.append(f"'{k}: '")
            sequence.append(v)
        sequence.append('"}"')
        return type_decl.define(type_decl.join(*sequence), id="decl")
    else:
        return type_decl.get(f"__bjo_{obj_hash}_decl")
