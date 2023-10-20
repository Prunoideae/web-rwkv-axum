from web_rwkv_axum.builders.blocks import md_blocks
from web_rwkv_axum.typed.bnf import RuleSet

rules = RuleSet("bnf")

md_blocks.numbered_list(rules, count=0)
# md_blocks.heading(rules, level=0, pre="'sus'")
print(rules.declare())
