from typing import Any


class SamplerBuilder:
    def type_id(self) -> str:
        """
        The type_id that this builder will use
        """

    def payload(self) -> Any:
        """
        Create the payload used by the sampler
        """
