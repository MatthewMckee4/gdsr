from abc import abstractmethod
import sys
from typing import Union, Tuple, Sequence, Mapping, List

if sys.version_info >= (3, 8):
    from typing import Protocol, SupportsIndex
else:
    from typing_extensions import Protocol, SupportsIndex


class Indexable(Protocol):
    @abstractmethod
    def __getitem__(self, __i: SupportsIndex) -> float: ...


PointLike = Union[
    Tuple[float, float],
    Indexable,
    Sequence[float],
    Mapping[int, float],
]
"""Object that is indexable at 0 and 1"""

InputPointsLike = Sequence[PointLike]
"""Sequence of objects that are indexable at 0 and 1"""

OutputPointsLike = List[Tuple[float, float]]
"""List of tuples of floats"""
