import sys
from enum import Enum
from typing import Iterator, Literal, Sequence, overload

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
        """Return the x coordinate."""
    @property
    def y(self) -> float:
        """Return the y coordinate."""
    def __init__(self, x: float, y: float) -> None:
        """Initialize the Point with x and y coordinates.

        :param float x: x coordinate
        :param float y: y coordinate
        """
    def distance_to(self, other: PointLike) -> float:
        """Return the distance to another point."""
    def cross(self, other: PointLike) -> float:
        """Return the cross product with another point."""
    def copy(self) -> Self:
        """Return a copy of the point."""
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
        """Return the origin of the grid."""
    @origin.setter
    def origin(self, origin: PointLike) -> None:
        """Set the origin of the grid."""
    columns: int
    """Number of columns in the grid."""
    rows: int
    """Number of rows in the grid."""
    @property
    def spacing_x(self) -> Point:
        """Return the spacing in the x direction."""
    @spacing_x.setter
    def spacing_x(self, spacing: PointLike) -> None:
        """Set the spacing in the x direction."""
    @property
    def spacing_y(self) -> Point:
        """Return the spacing in the y direction."""
    @spacing_y.setter
    def spacing_y(self, spacing: PointLike) -> None:
        """Set the spacing in the y direction."""
    def __init__(
        self,
        origin: PointLike = Point(0, 0),
        columns: int = 1,
        rows: int = 1,
        spacing_x: PointLike = Point(0, 0),
        spacing_y: PointLike = Point(0, 0),
    ) -> None:
        """Initialize the Grid with origin, columns, rows and spacing.

        :param PointLike origin: The origin of the grid, defaults to Point(0, 0)
        :param int columns: The number of columns in the grid, defaults to 1
        :param int rows: The number of rows in the grid, defaults to 1
        :param PointLike spacing_x: The spacing in the x direction, defaults to
        Point(0, 0)
        :param PointLike spacing_y: The spacing in the y direction, defaults to
        Point(0, 0)
        """
    def __str__(self) -> str:
        """Return a string representation of the grid."""
    def __repr__(self) -> str:
        """Return a string representation of the grid."""
    def copy(self) -> Self:
        """Return a copy of the grid."""
    @property
    def width(self) -> float:
        """Return the total width of the grid."""
    @property
    def height(self) -> float:
        """Return the total height of the grid."""
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the grid."""

class ElementReference:
    element: Element
    grid: Grid
    def __init__(self, element: Element, grid: Grid) -> None:
        """Initialize the ElementReference with an element and a grid.

        :param Element element: The element to reference.
        :param Grid grid: The grid to reference the element.
        """
    def __str__(self) -> str:
        """Return a string representation of the element reference."""
    def __repr__(self) -> str:
        """Return a string representation of the element reference."""
    def copy(self) -> Self:
        """Return a copy of the element reference."""

class CellReference:
    cell: Cell
    grid: Grid
    def __init__(self) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def copy(self) -> Self: ...

class PathType(Enum):
    Square = 0
    Round = 1
    Overlap = 2

class Path:
    @property
    def points(self) -> list[Point]:
        """Return the points of the path."""
    @points.setter
    def points(self, points: InputPointsLike) -> None:
        """Set the points of the path."""
    layer: int
    data_type: int
    path_type: PathType | None
    width: int | None
    def __init__(
        self,
        points: InputPointsLike,
        layer: int = 0,
        data_type: int = 0,
        path_type: PathType | None = None,
        width: int | None = None,
    ) -> None: ...
    def copy(self) -> Self:
        """Return a copy of the path."""
    def __str__(self) -> str:
        """Return a string representation of the path."""
    def __repr__(self) -> str:
        """Return a string representation of the path."""
    @property
    def length(self) -> float:
        """Return the length of the path."""

class Polygon:
    """Polygon object."""

    @property
    def points(self) -> list[Point]:
        """Return the points of the polygon."""
    @points.setter
    def points(self, points: InputPointsLike) -> None:
        """Set the points of the polygon."""
    layer: Layer
    """The layer of the polygon."""
    data_type: int
    """The data type of the polygon."""
    def __init__(
        self, points: InputPointsLike, layer: Layer = 0, data_type: int = 0
    ) -> None:
        """Initialize the Polygon.

        If the first and last points are not the same,
        the first point is appended to the end, to ensure that the polygon is closed.

        :param InputPointsLike points: Polygon vertices. Sequence of objects that are
        indexable at 0 and 1. Must not be empty
        :param Layer layer: Polygon layer, defaults to 0
        :param int data_type: Polygon data_type, defaults to 0
        """
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the polygon."""
    @property
    def area(self) -> float:
        """Return the area of the polygon."""
    @property
    def perimeter(self) -> float:
        """Return the perimeter of the polygon."""
    @overload
    def contains(self, point: PointLike) -> bool:
        """Return True if the polygon contains the point."""
    @overload
    def contains(self, points: Sequence[PointLike]) -> tuple[bool, ...]:
        """Return a tuple of booleans.

        Each boolean indicates if the polygon contains the corresponding point.
        """
    def contains_all(self, *points: PointLike) -> bool:
        """Return True if the polygon contains all of the points."""
    def contains_any(self, *points: PointLike) -> bool:
        """Return True if the polygon contains any of the points."""
    @overload
    def on_edge(self, point: PointLike) -> bool:
        """Return True if the point is on the edge of the polygon."""

    @overload
    def on_edge(self, points: Sequence[PointLike]) -> tuple[bool, ...]:
        """Return a tuple of booleans.

        Each boolean indicates if the corresponding point is on the edge of the polygon.
        """
    def on_edge_all(self, *points: PointLike) -> bool:
        """Return True if all of the points are on the edge of the polygon."""
    def on_edge_any(self, *points: PointLike) -> bool:
        """Return True if any of the points are on the edge of the polygon."""
    def intersects(self, other: Polygon) -> bool:
        """Return True if the polygon intersects with another polygon."""
    def rotate(self, angle: float, center: PointLike = (0, 0)) -> Self:
        """Rotates the polygon by an angle around a center point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike center: Center point of rotation, defaults to (0, 0).

        :return: Rotated polygon.
        """
    def visualize(self) -> None:
        """Visualises the polygon in your default web browser."""
    def copy(self) -> Self:
        """Return a copy of the polygon."""
    def __str__(self) -> str:
        """Return a string representation of the polygon."""
    def __repr__(self) -> str:
        """Return a string representation of the polygon."""

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

Element = CellReference | Path | Polygon | Text | ElementReference

class Cell:
    name: str
    @property
    def polygons(self) -> list[Polygon]: ...
    @property
    def paths(self) -> list[Path]: ...
    @property
    def cell_references(self) -> list[CellReference]: ...
    @property
    def element_references(self) -> list[ElementReference]: ...
    @property
    def texts(self) -> list[Text]: ...
    def __init__(self, name: str) -> None:
        """Initialize the Cell with a name.

        :param str name: Cell name
        """
    def add(self, *elements: Element) -> None:
        """Add elements to the cell."""
    def remove(self, *elements: Element) -> None:
        """Remove elements from the cell."""
    def __str__(self) -> str:
        """Return a string representation of the cell."""
    def __repr__(self) -> str:
        """Return a string representation of the cell."""
    def copy(self) -> Self:
        """Return a copy of the cell."""
    def width(self) -> float:
        """Return the width of the cell."""
    def height(self) -> float:
        """Return the height of the cell."""
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the cell."""
    def to_gds(
        self, filename: str, units: float = 1e-6, precision: float = 1e-10
    ) -> None:
        """Write the Cell to a GDS file.

        :param str filename: Output GDS file name.
        :param float units: GDS file units in meters, defaults to 1e-6.
        :param float precision: GDS file precision, defaults to 1e-10.
        """

class Library:
    name: str
    @property
    def cells(self) -> list[Cell]:
        """Return the cells in the library."""
    def __init__(self, name: str) -> None:
        """Initialize the Library with a name.

        :param str name: Library name
        """
    def add(self, *cells: Cell) -> None:
        """Add cells to the library."""
    def remove(self, *cells: Cell) -> None:
        """Remove cells from the library."""
    def to_gds(
        self, filename: str, units: float = 1e-6, precision: float = 1e-10
    ) -> None:
        """Write the Library to a GDS file.

        :param str filename: Output GDS file name.
        """

__all__ = [
    "Cell",
    "CellReference",
    "ElementReference",
    "Grid",
    "HorizontalPresentation",
    "Library",
    "Path",
    "PathType",
    "Point",
    "PointIterator",
    "Polygon",
    "Text",
    "VerticalPresentation",
]
