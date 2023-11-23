from web_rwkv_axum.api import Session
from web_rwkv_axum.builders.blocks.md_blocks import numbered_list, heading
from web_rwkv_axum.typed.bnf import RuleSet
from web_rwkv_axum.builders.transformers import (
    GlobalPenalty,
    BNFTransformer,
    SchemaBNF,
    DisableToken,
)
from web_rwkv_axum.builders.samplers import Typical, Nucleus
from web_rwkv_axum.builders.terminals import Lengthed
from time import time

uri = "ws://127.0.0.1:5678/ws"


async def main():
    async with Session(uri) as session:
        ruleset = RuleSet("steps")
        title = heading(ruleset, 0, pre=ruleset.literal("Funding Proposal"))
        match_list = numbered_list(ruleset)
        prompt = """Instruction: Select the most appropriate expression from the list of expressions and explain why.

Input:
Detrius: You sucks!
Lupis: What?
Available expressions: happy, sad, angry, confused, disgusted, disappointed, concerned, normal

Response: The most appropriate expression for Lupis is angry, as Lupis feels insulted by Detrius.

"""
        mail = """Input:
Detrius: Hey, do you know the guy akqwjrbui?
Lupis: What?
Available expressions: happy, sad, angry, confused, disgusted, disappointed, concerned, normal

Response: The most appropriate expression for Lupis is"""

        state = await session.states.create_state(initial_prompt=prompt)

        pipeline = await session.pipeline.create_pipeline(
            transformers=[[]],
            sampler=Nucleus(),
            terminal=Lengthed(32),
        )

        result = await pipeline.infer(
            states=[state],
            tokens=mail,
            update_prompt=False,
        )
        print(result.result)
        print(result.ms_elapsed, result.inferred_token)
        print(result.end_reason)

        await result.continue_([["\n\nInput:"]])

        print(result.result)
        print(result.ms_elapsed, result.inferred_token)
        print(result.end_reason)

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
