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

InputPointsLike = Sequence[PointLike]
OutputPointsLike = List[Tuple[float, float]]
