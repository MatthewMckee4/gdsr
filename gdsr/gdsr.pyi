from typing import Tuple, Union, List
from .typing import OutputPointsLike, InputPointsLike

class ArrayReference:
    def __init__(self) -> None: ...

class Reference:
    def __init__(self) -> None: ...

class Box:
    """Box object"""
    def __init__(self) -> None: ...

class Path:
    """Path object"""
    def __init__(self) -> None: ...

class Polygon:
    """Polygon object"""
    @property
    def points(self) -> OutputPointsLike:
        """Returns the points of the polygon."""

    @points.setter
    def points(self, points: InputPointsLike) -> None:
        """Sets the points of the polygon."""

    layer: int
    """The layer of the polygon."""

    data_type: int
    """The data type of the polygon."""

    def __init__(
        self, points: InputPointsLike, layer: int = 0, data_type: int = 0
    ) -> None:
        """Initializes the Polygon with points, layer, and data type.

        :param InputPointsLike points: Polygon vertices. Sequence of objects that are indexable at 0 and 1. Must not be empty
        :param int layer: Polygon layer, defaults to 0
        :param int data_type: Polygon data_type, defaults to 0
        """

    @property
    def bounding_box(self) -> Tuple[Tuple[float, float], Tuple[float, float]]:
        """Returns the bounding box of the polygon."""
        ...

    @property
    def area(self) -> float:
        """Returns the area of the polygon."""
        ...

class Node:
    def __init__(self) -> None: ...

class Text:
    def __init__(self) -> None: ...

Element = Union[ArrayReference, Reference, Box, Node, Path, Polygon, Text]

class Cell:
    name: str
    @property
    def array_references(self) -> List[ArrayReference]: ...
    @property
    def polygons(self) -> List[Polygon]: ...
    @property
    def boxes(self) -> List[Box]: ...
    @property
    def nodes(self) -> List[Node]: ...
    @property
    def paths(self) -> List[Path]: ...
    @property
    def references(self) -> List[Reference]: ...
    @property
    def texts(self) -> List[Text]: ...
    def __init__(self, name: str) -> None: ...
    def add(self, *elements: Element) -> None: ...

__all__ = [
    "ArrayReference",
    "Reference",
    "Polygon",
    "Box",
    "Node",
    "Path",
    "Text",
    "Cell",
]
