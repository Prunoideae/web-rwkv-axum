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
        prompt = """Instruction: Translate input Romaji to Japanese.

Input:"""
        mail = """ Nande haruhikage yatta no!?
Response:"""
        sampler = await session.samplers.create_sampler(Nucleus(temp=0.6, top_p=2.5))
        terminal = await session.terminals.create_terminal(Lengthed(48))
        state = await session.states.create_state(initial_prompt=prompt)

        pipeline = session.infer.pipeline(
            (state, []),
            sampler=sampler,
            terminal=terminal,
        )

        result = await pipeline.infer(mail, update_prompt=False)
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
