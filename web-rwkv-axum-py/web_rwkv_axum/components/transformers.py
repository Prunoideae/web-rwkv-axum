from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..api import Session


class Transformers:
    def __init__(self, session: "Session") -> None:
        self._session = session

    