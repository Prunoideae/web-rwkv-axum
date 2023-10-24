from web_rwkv_axum.api import Session

uri = "ws://127.0.0.1:5678/ws"


async def main():
    async with Session(uri) as session:
        state = await session.states.create_state()
        state2 = await state.copy()

        print(state.state_id, state2.state_id)


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
