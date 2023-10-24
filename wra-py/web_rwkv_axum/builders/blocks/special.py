import hashlib
from ...typed.bnf import Rule, RuleSet


def hashstring(s: str) -> str:
    h = hashlib.new("sha256")
    h.update(s.encode())
    return h.hexdigest()[:16]


def time(type_decl: RuleSet, time_string: str) -> Rule:
    prefix = hashstring(time_string)
    if not type_decl.defined(f"__bst_{prefix}_decl"):
        if not type_decl.defined("__bst_d"):
            digit = type_decl.with_prefix(f"__bst").define(type_decl.union(*(type_decl.literal(x) for x in range(10))), id="d")
        else:
            digit = type_decl.get("__bst_d")
        type_decl = type_decl.with_prefix(f"__bst_{prefix}")
        return type_decl.define(str(digit).join(type_decl.literal(x) if x else "" for x in time_string.split("%")), id="decl")
    else:
        return type_decl.get(f"__bst_{prefix}_decl")


def email(type_decl: RuleSet) -> Rule:
    if not type_decl.defined("__bse_decl"):
        type_decl = type_decl.with_prefix("__bse")
        alphas = "abcdefghijklmnopqrstuvwxyz"
        alphas += alphas.upper()
        alphas = type_decl.define(type_decl.union(*(type_decl.literal(x) for x in alphas)))
        nums = type_decl.define(type_decl.union(*(type_decl.literal(x) for x in range(10))))

        alphanum = type_decl.define(type_decl.union(alphas, nums))

        subdomain_rest = type_decl.define(None)
        subdomain_rest.rule = f"{alphanum}{subdomain_rest}|'-'{subdomain_rest}|{alphanum}|'-'"
        subdomain = type_decl.define(type_decl.optional(alphanum, subdomain_rest))

        domain_rest = type_decl.repeat(f"'.'{subdomain}")
        domain = type_decl.define(type_decl.optional(subdomain, domain_rest))

        specials = "!#$%&*+-/=?^_~"
        specials = type_decl.define(type_decl.union(*(type_decl.literal(x) for x in specials)))
        allowed = type_decl.define(type_decl.union(specials, alphanum))

        dot_seq = f"'.'{allowed}"
        local_part_rest = type_decl.define(None)
        local_part_rest.rule = f"{allowed}{local_part_rest}|{dot_seq}{local_part_rest}|{allowed}|{dot_seq}"
        local_part = type_decl.define(type_decl.optional(allowed, local_part_rest))
        return type_decl.define(f"{local_part}'@'{domain}", id="decl")
    else:
        return type_decl.get("__bse_decl")


def url(type_decl: RuleSet) -> Rule:
    ...
