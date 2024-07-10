import sys
from typing import Sequence, Tuple, List

if sys.version_info >= (3, 8):
    from typing import Protocol, Literal
else:
    from typing_extensions import Protocol, Literal


class PointLike(Protocol):
    """Object that is indexable at 0 and 1."""

    def __getitem__(self, __v: Literal[0, 1]) -> float:
        """Returns the value at index 0 or 1 as a float."""
        ...


InputPointsLike = Sequence[PointLike]
"""Sequence of objects that are indexable at 0 and 1."""


OutputPointsLike = List[Tuple[float, float]]
"""List of tuples representing points with x and y coordinates."""

__all__ = ["PointLike", "InputPointsLike", "OutputPointsLike"]
