from web_rwkv_axum.api import Session
from web_rwkv_axum.builders.transformers import GlobalPenalty
from web_rwkv_axum.builders.samplers import Nucleus
from web_rwkv_axum.builders.terminals import Lengthed
from time import time

uri = "ws://127.0.0.1:5678/ws"


async def main():
    async with Session(uri) as session:
        pipeline = session.infer.pipeline(
            (
                await session.states.create_state(),
                [await session.transformers.create_transformer(GlobalPenalty())],
            ),
            sampler=await session.samplers.create_sampler(Nucleus()),
            terminal=await session.terminals.create_terminal(Lengthed(32)),
        )

        result = await pipeline.infer("Breaking news: A man in Florida")
        print(result.result, end="")

        ms = result.ms_elapsed
        count = result.token_count
        start = time()

        for _ in range(15):
            await result.continue_()
            print(result.result, end="")
            ms += result.ms_elapsed
            count += result.token_count
        print(f"\nTPS: {count*1000/ms:.2f}")
        print(f"TPS(Real): {count/(time()-start):.2f}")


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
