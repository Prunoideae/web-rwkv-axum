from web_rwkv_axum.builders.blocks import md_blocks, special
from web_rwkv_axum.typed.bnf import RuleSet

rules = RuleSet("bnf")

special.email(rules)
# md_blocks.heading(rules, level=0, pre="'sus'")
print(rules.declare())
