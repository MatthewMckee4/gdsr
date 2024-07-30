"""Type hints for the gdsr package."""

from pathlib import Path
from typing import Iterable, Literal, Protocol, runtime_checkable


@runtime_checkable
class Indexable(Protocol):
    """Object that is indexable at 0 and 1."""

    def __getitem__(self, __v: Literal[0, 1]) -> float: ...  # noqa: D105


PointLike = Indexable
"""A object that supports indexing at positions 0 and 1 and returns float values.

It is recommended to use the gdsr.Point class but other types are also supported.

Examples:

    1. Using a list:
        >>> point = [1.0, 2.0]
        >>> assert isinstance(point, PointLike)
        >>> point[0]
        1.0
        >>> point[1]
        2.0

    2. Using a tuple:
        >>> point = (3.0, 4.0)
        >>> assert isinstance(point, PointLike)
        >>> point[0]
        3.0
        >>> point[1]
        4.0

    3. Using a dictionary:
        >>> point = {0: 5.0, 1: 6.0}
        >>> assert isinstance(point, PointLike)
        >>> point[0]
        5.0
        >>> point[1]
        6.0

    4. Using the gdsr Point :
        >>> from gdsr import Point
        >>> point = Point(7.0, 8.0)
        >>> assert isinstance(point, PointLike)
        >>> point[0]
        7.0
        >>> point[1]
        8.0
"""


@runtime_checkable
class PointIterable(Protocol):
    """An iterable of PointLike objects."""

    def __iter__(self) -> Iterable[PointLike]: ...  # noqa: D105


InputPointsLike = PointIterable
"""A Iterable of objects that support indexing at positions 0 and 1 and return float
values.

It is recommended to use a list of the gdsr.Point class but other types are also
supported.

Examples:

    1. Using a list of PointLike objects:
        >>> points = [[1.0, 2.0], [3.0, 4.0]]
        >>> isinstance(points, InputPointsLike)
        True
        >>> points[0][0]
        1.0
        >>> points[1][1]
        4.0

    2. Using a tuple of PointLike objects:
        >>> points = ((5.0, 6.0), (7.0, 8.0))
        >>> isinstance(points, InputPointsLike)
        True
        >>> points[0][1]
        6.0
        >>> points[1][0]
        7.0

    3. Using a list containing a mix of PointLike types:
        >>> points = [{0: 9.0, 1: 10.0}, (11.0, 12.0)]
        >>> isinstance(points, InputPointsLike)
        True
        >>> points[0][0]
        9.0
        >>> points[1][1]
        12.0

    4. Using a list of gdsr Point objects:
        >>> from gdsr import Point
        >>> points = [Point(13.0, 14.0), Point(15.0, 16.0)]
        >>> isinstance(points, InputPointsLike)
        True
        >>> points[0][0]
        13.0
        >>> points[1][1]
        16.0
"""

Layer = int
PathLike = Path | str
