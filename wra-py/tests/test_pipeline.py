from web_rwkv_axum.api import Session
from web_rwkv_axum.builders.blocks.special import email
from web_rwkv_axum.typed.bnf import RuleSet
from web_rwkv_axum.builders.transformers import GlobalPenalty, BNFTransformer, SchemaBNF
from web_rwkv_axum.builders.samplers import Nucleus
from web_rwkv_axum.builders.terminals import Lengthed
from time import time

uri = "ws://127.0.0.1:5678/ws"


async def main():
    async with Session(uri) as session:
        penalty = await session.transformers.create_transformer(GlobalPenalty())

        ruleset = RuleSet("email")
        match_email = email(ruleset)
        match_all = ruleset.define(f"{match_email}'\n'")
        bnf = await session.transformers.create_transformer(BNFTransformer(start=match_all, rules=ruleset))

        sampler = await session.samplers.create_sampler(Nucleus())
        terminal = await session.terminals.create_terminal(Lengthed(64))
        pipeline = session.infer.pipeline(
            (await session.states.create_state(), [penalty, bnf]),
            sampler=sampler,
            terminal=terminal,
        )

        result = await pipeline.infer("Name: Tom\nTom's Email: ", update_prompt=False)
        print(result.result, end="")

        # ms = result.ms_elapsed
        # count = result.token_count
        # start = time()

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
