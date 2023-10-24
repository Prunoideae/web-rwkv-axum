from web_rwkv_axum.api import Session
from web_rwkv_axum.builders.blocks.md_blocks import numbered_list, heading
from web_rwkv_axum.typed.bnf import RuleSet
from web_rwkv_axum.builders.transformers import GlobalPenalty, BNFTransformer, SchemaBNF, DisableToken
from web_rwkv_axum.builders.samplers import Typical, Nucleus
from web_rwkv_axum.builders.terminals import Lengthed
from time import time

uri = "ws://127.0.0.1:5678/ws"


async def main():
    async with Session(uri) as session:
        penalty = await session.transformers.create_transformer(GlobalPenalty())

        ruleset = RuleSet("steps")
        title = heading(ruleset, 0, pre=ruleset.literal("Funding Proposal"))
        match_list = numbered_list(ruleset)
        disable_0 = await session.transformers.create_transformer(DisableToken([0]))
        bnf = await session.transformers.create_transformer(BNFTransformer(start=ruleset.define(ruleset.join(title, match_list)), rules=ruleset))

        sampler = await session.samplers.create_sampler(Typical())
        terminal = await session.terminals.create_terminal(Lengthed(256))
        pipeline = session.infer.pipeline(
            (await session.states.create_state(), [disable_0, bnf]),
            sampler=sampler,
            terminal=terminal,
        )

        prompt = """Instruction: Write a proposal for funding.

Response:
```markdown
"""

        result = await pipeline.infer(prompt, update_prompt=False)
        print(result.result)
        print(result.ms_elapsed, result.token_count)

        # for _ in range(15):
        #     await result.continue_()
        #     print(result.result, end="")
        #     ms += result.ms_elapsed
        #     count += result.token_count
        # print(f"\nTPS: {count*1000/ms:.2f}")
        # print(f"TPS(Real): {count/(time()-start):.2f}")


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
