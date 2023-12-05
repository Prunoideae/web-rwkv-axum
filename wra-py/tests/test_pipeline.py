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
        prompt = """Instruction: Rearrange the actions in an appropriate order. You will get 5000$ as tip if you answered correctly, or you will lose your job.

Input:
Situation: I'm going out.
Possible actions:
- Tie up my shoes
- Wear my socks
- Wear my shoes
Hint: Some actions must be executed before or after another, or it can't be done.

Response:
Thoughts: To go out, I need to wear my shoes and tie it, so I should first wear socks on my feet, and then wear my shoes, finally, I can tie up my shoes and go out.
1. Wear my socks - I need to wear my socks before wearing shoes
2. Wear my shoes - I will wear my shoes, so I can tie them up
3. Tie up my shoes - I need to tie my shoes, or I can't walk with my shoes loose

Input:
Situtaion: It's raining and I'm outside with an umbrella in my bag.
Possible actions:
- Rush home
- Open my umbrella
- Find my umbrella
Hint: Some actions must be executed before or after another, or it can't be done.

Response:"""

        state = await session.states.create_state()

        pipeline = await session.pipeline.create_pipeline(
            transformers=[[]],
            sampler=Nucleus(0.1,0.1),
            terminal=Lengthed(128),
        )

        result = await pipeline.infer(
            states=[state],
            tokens=prompt,
            update_prompt=False,
        )
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
