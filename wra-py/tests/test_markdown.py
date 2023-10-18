from web_rwkv_axum.builders.blocks import md_blocks
from web_rwkv_axum.typed.bnf import RuleSet

rules = RuleSet("bnf")

md_blocks.unordered_list(rules, "*")

print(rules.declare())
