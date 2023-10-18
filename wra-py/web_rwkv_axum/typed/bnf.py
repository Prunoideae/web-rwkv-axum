class Rule:
    def __init__(self, rule_id: str, rule: str) -> None:
        self.rule_id = rule_id
        self.rule = rule

    def __str__(self) -> str:
        return f"<{self.rule_id}>"


class RuleSet:
    def __init__(self, rule_prefix: str) -> None:
        self.prefix = rule_prefix
        self.rules: dict[str, Rule] = {}
        self.rule_counter = 0

    def with_prefix(self, prefix: str) -> "RuleSet":
        sub = RuleSet(prefix)
        sub.rules = self.rules
        return sub

    def defined(self, id: str) -> bool:
        return id in self.rules

    def get(self, id: str) -> Rule:
        return self.rules[id]

    def define(self, rule: str, id: str = None) -> Rule:
        if id is None:
            id = str(self.rule_counter)
            self.rule_counter += 1
        key = f"{self.prefix}_{id}"
        rule_holder = Rule(key, rule)
        self.rules[key] = rule_holder
        return rule_holder

    def literal(self, literal) -> str:
        sanitized = str(literal)
        sanitized = sanitized.replace("\n", "\\n").replace("\t", "\\t").replace("\\", "\\\\").replace("\r", "\\r")
        return f'"{sanitized}"' if '"' not in sanitized else f"'{sanitized}'"

    def union(self, *elements: str | Rule) -> str:
        return "|".join(map(str, elements))

    def join(self, *elements: str | Rule) -> str:
        return "".join(map(str, elements))

    def optional(self, base: str | Rule, optional: str | Rule):
        """
        Define an optional grammar, where the optional can be or not be present
        right after the base.

        An optional rule is:
        ```text
        <rule>::=<base>|<base><optional>
        ```
        """
        return self.union(base, self.join(base, optional))

    def optional_rev(self, optional: str | Rule, base: Rule | str):
        """
        Define an optional grammar, where the optional can be or not be present
        right before the base.

        An optional rule is:
        ```text
        <rule>::=<base>|<optional><base>
        ```
        """
        return self.union(self.join(optional, base), base)

    def except_(self, rule: str | Rule):
        return f"<except!([{rule.rule_id}])>" if isinstance(rule, Rule) else f"<except!({rule})>"

    def any(self):
        return "<any!>"

    def repeat(self, element: str | Rule) -> Rule:
        """
        Define a recursive grammar.

        A recursive rule is:
        ```text
        <this>::=<element>|<element><this>
        ```
        """
        this = self.define(None)
        this.rule = self.optional(element, this)
        return this

    def declare(self) -> str:
        rules = []
        for k, v in self.rules.items():
            rules.append(f"<{k}>::={v.rule}")
        return "\n".join(rules)
