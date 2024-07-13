from typing import Sequence, runtime_checkable

from typing_extensions import Literal, Protocol


@runtime_checkable
class Indexable(Protocol):
    """Object that is indexable at 0 and 1."""

    def __getitem__(self, __v: Literal[0, 1]) -> float: ...


PointLike = Indexable

InputPointsLike = Sequence[PointLike]
"""Sequence of objects that are indexable at 0 and 1."""

Layer = int
