import sys
from enum import Enum
from typing import Iterator, List, Sequence, Tuple, Union, overload

if sys.version_info >= (3, 8):
    from typing import Literal
else:
    from typing_extensions import Literal

if sys.version_info >= (3, 11):
    from typing import Self
else:
    from typing_extensions import Self

from .typings import InputPointsLike, Layer, PointLike

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

    def copy(self) -> Self:
        """Returns a copy of the point."""

    def rotate(self, angle: float, center: PointLike = Point(0, 0)) -> Self:
        """Rotates the point by an angle around a center point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike center: Center point of rotation, defaults to Point(0, 0).
        """

    def scale(self, factor: float, center: PointLike = Point(0, 0)) -> Self:
        """Scales the point by a factor around a center point.

        :param float factor: Scaling factor.
        :param PointLike center: Center point of scaling, defaults to Point(0, 0).
        """

    def __getitem__(self, index: Literal[0, 1]) -> float: ...
    def __bool__(self) -> bool: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __add__(self, other: PointLike) -> Self: ...
    def __radd__(self, other: PointLike) -> Self: ...
    def __sub__(self, other: PointLike) -> Self: ...
    def __rsub__(self, other: PointLike) -> Self: ...
    def __mul__(self, value: float) -> Self: ...
    def __rmul__(self, value: float) -> Self: ...
    def __truediv__(self, value: float) -> Self: ...
    def __floordiv__(self, value: float) -> Self: ...
    def __neg__(self) -> Self: ...
    def __round__(self, ndigits: int | None) -> Self: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, value: object) -> bool: ...
    def __lt__(self, other: object) -> bool: ...
    def __le__(self, other: object) -> bool: ...
    def __gt__(self, other: object) -> bool: ...
    def __ge__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __iter__(self) -> PointIterator: ...

class Grid:
    @property
    def origin(self) -> Point:
        """Returns the origin of the grid."""

    @origin.setter
    def origin(self, origin: PointLike) -> None:
        """Sets the origin of the grid."""

    columns: int
    """Number of columns in the grid."""

    rows: int
    """Number of rows in the grid."""

    @property
    def spacing_x(self) -> Point:
        """Returns the spacing in the x direction."""

    @spacing_x.setter
    def spacing_x(self, spacing: PointLike) -> None:
        """Sets the spacing in the x direction."""

    @property
    def spacing_y(self) -> Point:
        """Returns the spacing in the y direction."""

    @spacing_y.setter
    def spacing_y(self, spacing: PointLike) -> None:
        """Sets the spacing in the y direction."""

    def __init__(
        self,
        origin: PointLike = Point(0, 0),
        columns: int = 1,
        rows: int = 1,
        spacing_x: PointLike = Point(0, 0),
        spacing_y: PointLike = Point(0, 0),
    ) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

class ElementReference:
    element: Element
    grid: Grid
    def __init__(self) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

class CellReference:
    cell: Cell
    grid: Grid
    def __init__(self) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

class Path:
    """Path object"""
    def __init__(self) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

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

    def rotate(self, angle: float, center: PointLike = (0, 0)) -> Self:
        """Rotates the polygon by an angle around a center point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike center: Center point of rotation, defaults to (0, 0).

        :return: Rotated polygon.
        """

    def copy(self) -> Self:
        """Returns a copy of the polygon."""

    def __str__(self) -> str:
        """Returns a string representation of the polygon."""

    def __repr__(self) -> str:
        """Returns a string representation of the polygon."""

class Node:
    def __init__(self) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

class VerticalPresentation(Enum):
    Top = 0
    Middle = 1
    Bottom = 2

class HorizontalPresentation(Enum):
    Left = 0
    Centre = 1
    Right = 2

class Text:
    text: str
    @property
    def origin(self) -> Point: ...
    @origin.setter
    def origin(self, origin: PointLike) -> None: ...
    layer: Layer
    magnification: float
    angle: float
    x_reflection: bool
    vertical_presentation: VerticalPresentation
    horizontal_presentation: HorizontalPresentation
    def __init__(
        self,
        text: str,
        origin: PointLike = Point(0, 0),
        layer: Layer = 0,
        magnification: float = 1.0,
        angle: float = 0.0,
        x_reflection: bool = False,
        vertical_presentation: VerticalPresentation = VerticalPresentation.Middle,
        horizontal_presentation: HorizontalPresentation = HorizontalPresentation.Centre,
    ) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

Element = Union[CellReference, Node, Path, Polygon, Text, ElementReference]

class Cell:
    name: str
    @property
    def polygons(self) -> List[Polygon]: ...
    @property
    def nodes(self) -> List[Node]: ...
    @property
    def paths(self) -> List[Path]: ...
    @property
    def cell_references(self) -> List[CellReference]: ...
    @property
    def element_references(self) -> List[ElementReference]: ...
    @property
    def texts(self) -> List[Text]: ...
    def __init__(self, name: str) -> None:
        """Initializes the Cell with a name.

        :param str name: Cell name
        """

    def add(self, *elements: Element) -> None:
        """Adds elements to the cell."""
    def remove(self, *elements: Element) -> None:
        """Removes elements from the cell."""

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...
    def to_gds(
        self, filename: str, units: float = 1e-6, precision: float = 1e-10
    ) -> None:
        """Writes the Cell to a GDS file.

        :param str filename: Output GDS file name.
        """

class Library:
    name: str
    @property
    def cells(self) -> List[Cell]:
        """Returns the cells in the library."""
    def __init__(self, name: str) -> None:
        """Initializes the Library with a name

        :param str name: Library name
        """
    def add(self, *cells: Cell) -> None:
        """Adds cells to the library."""
    def remove(self, *cells: Cell) -> None:
        """Removes cells from the library."""

    def to_gds(
        self, filename: str, units: float = 1e-6, precision: float = 1e-10
    ) -> None:
        """Writes the Library to a GDS file.

        :param str filename: Output GDS file name.
        """

__all__ = [
    "Cell",
    "CellReference",
    "ElementReference",
    "Grid",
    "HorizontalPresentation",
    "Library",
    "Node",
    "Path",
    "Point",
    "PointIterator",
    "Polygon",
    "Text",
    "VerticalPresentation",
]
