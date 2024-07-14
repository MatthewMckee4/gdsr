from typing import Iterator, List, Sequence, Tuple, Union, overload

from typing_extensions import Literal

from .typing import InputPointsLike, Layer, PointLike

class PointIterator(Iterator[float]):
    def __next__(self) -> float: ...

class Point:
    @property
    def x(self) -> float:
        """Returns the x coordinate."""
    @property
    def y(self) -> float:
        """Returns the y coordinate."""
    def __init__(self, x: float, y: float) -> None:
        """Initializes the Point with x and y coordinates.

        :param float x: x coordinate
        :param float y: y coordinate
        """
    def distance_to(self, other: PointLike) -> float:
        """Returns the distance to another point."""

    def cross(self, other: PointLike) -> float:
        """Returns the cross product with another point."""

    def copy(self) -> Point:
        """Returns a copy of the point."""

    def rotate(self, angle: float, center: PointLike = Point(0, 0)) -> Point:
        """Rotates the point by an angle around a center point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike center: Center point of rotation, defaults to Point(0, 0).

        :return: Rotated point.
        """

    def scale(self, factor: float, center: PointLike = Point(0, 0)) -> Point:
        """Scales the point by a factor around a center point.

        :param float factor: Scaling factor.
        :param PointLike center: Center point of scaling, defaults to Point(0, 0).

        :return: Scaled point.
        """

    def __getitem__(self, index: Literal[0, 1]) -> float: ...
    def __bool__(self) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __add__(self, other: PointLike) -> Point: ...
    def __radd__(self, other: PointLike) -> Point: ...
    def __sub__(self, other: PointLike) -> Point: ...
    def __rsub__(self, other: PointLike) -> Point: ...
    def __mul__(self, value: float) -> Point: ...
    def __rmul__(self, value: float) -> Point: ...
    def __truediv__(self, value: float) -> Point: ...
    def __floordiv__(self, value: float) -> Point: ...
    def __neg__(self) -> Point: ...
    def __round__(self, ndigits: int | None) -> Point: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, value: object) -> bool: ...
    def __lt__(self, other: object) -> bool: ...
    def __le__(self, other: object) -> bool: ...
    def __gt__(self, other: object) -> bool: ...
    def __ge__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __iter__(self) -> PointIterator: ...

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
    def points(self) -> List[Point]:
        """Returns the points of the polygon."""

    @points.setter
    def points(self, points: InputPointsLike) -> None:
        """Sets the points of the polygon."""

    layer: Layer
    """The layer of the polygon."""

    data_type: int
    """The data type of the polygon."""

    def __init__(
        self, points: InputPointsLike, layer: Layer = 0, data_type: int = 0
    ) -> None:
        """Initializes the Polygon with points, layer, and data type. If the first
        and last points are not the same, the first point is appended to the end.
        To ensure that the polygon is closed.

        :param InputPointsLike points: Polygon vertices. Sequence of objects that are
        indexable at 0 and 1. Must not be empty
        :param Layer layer: Polygon layer, defaults to 0
        :param int data_type: Polygon data_type, defaults to 0
        """

    @property
    def bounding_box(self) -> Tuple[Point, Point]:
        """Returns the bounding box of the polygon."""

    @property
    def area(self) -> float:
        """Returns the area of the polygon."""

    @property
    def perimeter(self) -> float:
        """Returns the perimeter of the polygon."""

    @overload
    def contains(self, point: PointLike) -> bool:
        """Returns True if the polygon contains the point."""

    @overload
    def contains(self, points: Sequence[PointLike]) -> Tuple[bool, ...]:
        """Returns a tuple of booleans indicating if the polygon contains each
        of the points."""

    def contains_all(self, *points: PointLike) -> bool:
        """Returns True if the polygon contains all of the points."""

    def contains_any(self, *points: PointLike) -> bool:
        """Returns True if the polygon contains any of the points."""

    @overload
    def on_edge(self, point: PointLike) -> bool:
        """Returns True if the point is on the edge of the polygon."""

    @overload
    def on_edge(self, points: Sequence[PointLike]) -> Tuple[bool, ...]:
        """Returns a tuple of booleans indicating if each of the points is on the
        edge of the polygon."""

    def on_edge_all(self, *points: PointLike) -> bool:
        """Returns True if all of the points are on the edge of the polygon."""

    def on_edge_any(self, *points: PointLike) -> bool:
        """Returns True if any of the points are on the edge of the polygon."""

    def intersects(self, other: Polygon) -> bool:
        """Returns True if the polygon intersects with another polygon."""

    def rotate(self, angle: float, center: PointLike = (0, 0)) -> Polygon:
        """Rotates the polygon by an angle around a center point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike center: Center point of rotation, defaults to (0, 0).

        :return: Rotated polygon.
        """

    def copy(self) -> Polygon:
        """Returns a copy of the polygon."""

    def __str__(self) -> str:
        """Returns a string representation of the polygon."""

    def __repr__(self) -> str:
        """Returns a string representation of the polygon."""

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
    def add(self, *elements: Element) -> None:
        """Adds elements to the cell."""

class Library:
    name: str
    @property
    def cells(self) -> List[Cell]:
        """Returns the cells in the library."""
    def add(self, *cells: Cell) -> None:
        """Adds cells to the library."""
    def __init__(self, name: str) -> None:
        """Initializes the Library with a name, units, and precision.

        :param str name: Library name
        """

    def to_gds(self, filename: str, units: float, precision: float) -> None:
        """Writes the library to a GDS file.

        :param str filename: Output GDS file name.
        """

__all__ = [
    "PointIterator",
    "Point",
    "ArrayReference",
    "Reference",
    "Polygon",
    "Box",
    "Node",
    "Path",
    "Text",
    "Cell",
    "Library",
]
