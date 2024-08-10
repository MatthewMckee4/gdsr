import sys
from enum import Enum
from typing import Generic, Iterator, Literal, Mapping, TypeVar

if sys.version_info >= (3, 11):
    from typing import Self
else:
    from typing_extensions import Self

if sys.version_info >= (3, 10):
    from typing import TypeAlias
else:
    from typing_extension import TypeAlias

from .typings import InputPointsLike, Layer, PathLike, PointLike

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
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotates the point by an angle around a centre point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to Point(0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scales the point by a factor around a centre point.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to Point(0, 0).
        """
    def round(self, digits: int = 0) -> Self:
        """Return the point with rounded coordinates.

        :param int digits: Number of digits to round to, defaults to None.
        """
    def angle_to(self, other: PointLike) -> float | None:
        """Return the angle to another point in degrees.

        Returns None if the points are the same.

        :param PointLike other: The other point.
        """
    def __getitem__(self, index: Literal[0, 1]) -> float:
        """Return the x or y coordinate of the point."""
    def __bool__(self) -> bool:
        """Return True if the point is not the origin."""
    def __repr__(self) -> str:
        """Return a string representation of the point."""
    def __str__(self) -> str:
        """Return a string representation of the point."""
    def __add__(self, other: PointLike) -> Self:
        """Return the sum of the point and another point."""
    def __radd__(self, other: PointLike) -> Self:
        """Return the sum of the point and another point."""
    def __sub__(self, other: PointLike) -> Self:
        """Return the difference of the point and another point."""
    def __rsub__(self, other: PointLike) -> Self:
        """Return the difference of the point and another point."""
    def __mul__(self, value: float) -> Self:
        """Return the product of the point and a scalar."""
    def __rmul__(self, value: float) -> Self:
        """Return the product of the point and a scalar."""
    def __truediv__(self, value: float) -> Self:
        """Return the quotient of the point and a scalar."""
    def __floordiv__(self, value: float) -> Self:
        """Return the floored quotient of the point and a scalar."""
    def __neg__(self) -> Self:
        """Return the negative of the point."""
    def __round__(self, ndigits: int | None) -> Self:
        """Return the point with rounded coordinates."""
    def __eq__(self, other: object) -> bool:
        """Return True if the point is equal to another object."""
    def __ne__(self, value: object) -> bool:
        """Return True if the point is not equal to another object."""
    def __lt__(self, other: object) -> bool:
        """Return True if the point is less than another object."""
    def __le__(self, other: object) -> bool:
        """Return True if the point is less than or equal to another object."""
    def __gt__(self, other: object) -> bool:
        """Return True if the point is greater than another object."""
    def __ge__(self, other: object) -> bool:
        """Return True if the point is greater than or equal to another object."""
    def __hash__(self) -> int:
        """Return the hash of the point."""
    def __iter__(self) -> PointIterator:
        """Return an iterator over the coordinates of the point."""

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
    angle: float
    """Angle of the grid."""
    magnification: float
    """Magnification of the grid."""
    x_reflection: bool
    """X reflection of the grid."""
    def __init__(
        self,
        origin: PointLike = Point(0, 0),
        columns: int = 1,
        rows: int = 1,
        spacing_x: PointLike = Point(0, 0),
        spacing_y: PointLike = Point(0, 0),
        angle: float = 0.0,
        magnification: float = 1.0,
        x_reflection: bool = False,
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
    def copy(self) -> Self:
        """Return a copy of the grid."""
    def move_to(self, point: PointLike) -> Self:
        """Move the grid to a point.

        This method modifies the grid in place and returns itself.

        :param PointLike point: Point to move the grid to.
        """
    def move_by(self, vector: PointLike) -> Self:
        """Move the grid by a vector.

        This method modifies the grid in place and returns itself.

        :param PointLike vector: Vector to move the grid by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate the grid by an angle around a centre point.

        This method modifies the grid in place and returns itself.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to (0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale the grid by a factor around a centre point.

        This method modifies the grid in place and returns itself.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to (0, 0).
        """
    def __str__(self) -> str:
        """Return a string representation of the grid."""
    def __repr__(self) -> str:
        """Return a string representation of the grid."""

Instance: TypeAlias = Cell | Element

T_Instance = TypeVar("T_Instance", bound=Instance, covariant=True)
"""Type variable for an instance."""

class Reference(Generic[T_Instance]):
    """Reference object. Do not subscript this class, use inferred generic types."""

    instance: T_Instance
    """The instance to reference."""
    grid: Grid
    """The grid to reference the cell."""
    def __init__(self, instance: T_Instance, grid: Grid = Grid()) -> None:
        """Initialize the Reference with an instance and a grid.

        :param Instance instance: The instance to reference.
        :param Grid grid: The grid to reference the cell.
        """
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the reference."""
    def copy(self) -> Reference[T_Instance]:
        """Return a copy of the reference."""
    def move_to(self, point: PointLike) -> Self:
        """Move the reference to a point.

        This method modifies the reference in place and returns itself.

        :param PointLike point: Point to move the reference to.
        """
    def move_by(self, vector: PointLike) -> Self:
        """Move the reference by a vector.

        This method modifies the reference in place and returns itself.

        :param PointLike vector: Vector to move the reference by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate the reference by an angle around a centre point.

        This method modifies the reference in place and returns itself.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to (0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale the reference by a factor around a centre point.

        This method modifies the reference in place and returns itself.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to (0, 0).
        """
    def __str__(self) -> str:
        """Return a string representation of the reference."""
    def __repr__(self) -> str:
        """Return a string representation of the reference."""
    def __eq__(self, value: object) -> bool:
        """Return True if the reference is equal to another object."""

class PathType(Enum):
    Square = 0
    Round = 1
    Overlap = 2

    @staticmethod
    def values() -> list[PathType]:
        """Return a list of all PathType values."""

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
    width: float | None
    def __init__(
        self,
        points: InputPointsLike,
        layer: int = 0,
        data_type: int = 0,
        path_type: PathType | None = None,
        width: float | None = None,
    ) -> None: ...
    @property
    def length(self) -> float:
        """Return the length of the path."""
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the path."""
    def copy(self) -> Self:
        """Return a copy of the path."""
    def move_to(self, point: PointLike) -> Self:
        """Move the path to a point.

        This method modifies the path in place and returns itself.

        :param PointLike point: Point to move the path to.
        """
    def move_by(self, vector: PointLike) -> Self:
        """Move the path by a vector.

        This method modifies the path in place and returns itself.

        :param PointLike vector: Vector to move the path by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate the path by an angle around a centre point.

        This method modifies the path in place and returns itself.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to (0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale the path by a factor around a centre point.

        This method modifies the path in place and returns itself.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to (0, 0).
        """
    def __str__(self) -> str:
        """Return a string representation of the path."""
    def __repr__(self) -> str:
        """Return a string representation of the path."""
    def __eq__(self, value: object) -> bool:
        """Return True if the path is equal to another object."""

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
    def contains(self, point: PointLike) -> bool:
        """Return True if the polygon contains the point."""
    def contains_all(self, *points: PointLike) -> bool:
        """Return True if the polygon contains all of the points."""
    def contains_any(self, *points: PointLike) -> bool:
        """Return True if the polygon contains any of the points."""
    def on_edge(self, point: PointLike) -> bool:
        """Return True if the point is on the edge of the polygon."""
    def on_edge_all(self, *points: PointLike) -> bool:
        """Return True if all of the points are on the edge of the polygon."""
    def on_edge_any(self, *points: PointLike) -> bool:
        """Return True if any of the points are on the edge of the polygon."""
    def intersects(self, other: Polygon) -> bool:
        """Return True if the polygon intersects with another polygon."""
    def visualize(self) -> None:
        """Visualises the polygon in your default web browser."""
    def copy(self) -> Self:
        """Return a copy of the polygon."""
    def move_to(self, point: PointLike) -> Self:
        """Move the polygon to a point.

        This method modifies the polygon in place and returns itself.

        :param PointLike point: Point to move the polygon to.
        """
    def move_by(self, vector: PointLike) -> Self:
        """Move the polygon by a vector.

        This method modifies the polygon in place and returns itself.

        :param PointLike vector: Vector to move the polygon by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate the polygon by an angle around a centre point.

        This method modifies the polygon in place and returns itself.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to (0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale the polygon by a factor around a centre point.

        This method modifies the polygon in place and returns itself.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to (0, 0).
        """
    def __str__(self) -> str:
        """Return a string representation of the polygon."""
    def __repr__(self) -> str:
        """Return a string representation of the polygon."""

class VerticalPresentation(Enum):
    Top = 0
    Middle = 1
    Bottom = 2

    @staticmethod
    def values() -> list[VerticalPresentation]:
        """Return a list of all VerticalPresentation values."""

class HorizontalPresentation(Enum):
    Left = 0
    Centre = 1
    Right = 2

    @staticmethod
    def values() -> list[HorizontalPresentation]:
        """Return a list of all HorizontalPresentation values."""

class Text:
    text: str
    """Text content."""
    @property
    def origin(self) -> Point:
        """Text origin."""
    @origin.setter
    def origin(self, origin: PointLike) -> None:
        """Set the text origin."""
    layer: Layer
    """Text layer."""
    magnification: float
    """Text magnification."""
    angle: float
    """Text angle in degrees."""
    x_reflection: bool
    """Text x reflection."""
    vertical_presentation: VerticalPresentation
    """Text vertical presentation."""
    horizontal_presentation: HorizontalPresentation
    """Text horizontal presentation."""
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
    ) -> None:
        """Initialize the Text with text and origin.

        :param str text: Text content.
        :param PointLike origin: Text origin, defaults to Point(0, 0).
        :param Layer layer: Text layer, defaults to 0.
        :param float magnification: Text magnification, defaults to 1.0.
        :param float angle: Text angle in degrees, defaults to 0.0.
        :param bool x_reflection: Text x reflection, defaults to False.
        :param VerticalPresentation vertical_presentation: Text vertical presentation,
        defaults to VerticalPresentation.Middle.
        :param HorizontalPresentation horizontal_presentation: Text horizontal
        presentation, defaults to HorizontalPresentation.Centre.
        """
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the text."""
    def copy(self) -> Self:
        """Return a copy of the text."""
    def move_to(self, point: PointLike) -> Self:
        """Move the text to a point.

        This method modifies the text in place and returns itself.

        :param PointLike point: Point to move the text to.
        """
    def move_by(self, vector: PointLike) -> Self:
        """Move the text by a vector.

        This method modifies the text in place and returns itself.

        :param PointLike vector: Vector to move the text by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate the text by an angle around a centre point.

        This method modifies the text in place and returns itself.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to (0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale the text by a factor around a centre point.

        This method modifies the text in place and returns itself.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to (0, 0).
        """
    def __str__(self) -> str:
        """Return a string representation of the text."""
    def __repr__(self) -> str:
        """Return a string representation of the text."""

Element: TypeAlias = Reference[Instance] | Path | Polygon | Text

class Cell:
    name: str
    @property
    def polygons(self) -> list[Polygon]: ...
    @property
    def paths(self) -> list[Path]: ...
    @property
    def references(self) -> list[Reference[Instance]]: ...
    @property
    def texts(self) -> list[Text]: ...
    def __init__(self, name: str) -> None:
        """Initialize the Cell with a name.

        :param str name: Cell name
        """
    @property
    def bounding_box(self) -> tuple[Point, Point]:
        """Return the bounding box of the cell."""
    def add(self, *elements: Element) -> None:
        """Add elements to the cell."""
    def remove(self, *elements: Element) -> None:
        """Remove elements from the cell."""
    def contains(self, element: Element) -> bool:
        """Return True if the cell contains the element."""
    def is_empty(self) -> bool:
        """Return True if the cell has no elements."""
    def move_to(self, point: PointLike) -> Self:
        """Move everything in the cell to a point.

        :param PointLike point: Point to move the cell to.
        """

    def move_by(self, vector: PointLike) -> Self:
        """Move everything in the cell by a vector.

        :param PointLike vector: Vector to move the cell by.
        """
    def rotate(self, angle: float, centre: PointLike = Point(0, 0)) -> Self:
        """Rotate everything in the cell by an angle around a centre point.

        :param float angle: Counter-clockwise rotation angle in degrees.
        :param PointLike centre: Centre point of rotation, defaults to Point(0, 0).
        """
    def scale(self, factor: float, centre: PointLike = Point(0, 0)) -> Self:
        """Scale everything in the cell by a factor around a centre point.

        :param float factor: Scaling factor.
        :param PointLike centre: Centre point of scaling, defaults to Point(0, 0).
        """
    def copy(self) -> Self:
        """Return a copy of the cell."""
    def to_gds(
        self,
        file_name: PathLike | None = None,
        units: float = 1e-6,
        precision: float = 1e-10,
    ) -> str:
        """Write the Cell to a GDS file.

        :param PathLike file_name: Output GDS file name.
        :param float units: GDS file units in meters, defaults to 1e-6.
        :param float precision: GDS file precision, defaults to 1e-10.
        :return: GDS file name
        """
    def __str__(self) -> str:
        """Return a string representation of the cell."""
    def __repr__(self) -> str:
        """Return a string representation of the cell."""
    def __eq__(self, value: object) -> bool:
        """Return True if the cell is equal to another object."""

class Library:
    name: str
    @property
    def cells(self) -> Mapping[str, Cell]:
        """Return the cells in the library."""
    def __init__(self, name: str = "library") -> None:
        """Initialize the Library with a name.

        :param str name: Library name
        """
    def add(self, *cells: Cell, replace_pre_existing: bool = False) -> None:
        """Add cells to the library.

        The cells that are added are not copied and are added by reference.
        This means that modifying the cells after adding them to the library will
        also modify the cells in the library.

        If replace_pre_existing is True, this will also look at cells in references and
        add those to the library as well.

        :param Cell cells: Cells to add to the library.
        :param bool replace_pre_existing: Replace pre-existing cells with the same name,
        defaults to False. If this is False and a cell with the same name already exists
        in the library, a ValueError will be raised.

        ```python

        import gdsr

        lib = gdsr.Library()

        cell = gdsr.Cell("cell")

        lib.add(cell)

        cell_from_lib = lib.cells["cell"]

        cell_from_lib.add(gdsr.Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))

        assert cell is cell_from_lib
        ```
        """
    def remove(self, *cells: Cell) -> None:
        """Remove cells from the library."""
    def contains(self, cell: Cell) -> bool:
        """Return True if the library contains the cell."""
    def copy(self, deep: bool = False) -> Self:
        """Return a copy of the library.

        :param bool deep: If True, a deep copy is returned, defaults to False.
        """
    def to_gds(
        self,
        file_name: PathLike | None = None,
        units: float = 1e-6,
        precision: float = 1e-10,
    ) -> str:
        """Write the Library to a GDS file.

        :param PathLike file_name: Output GDS file name.
        :param float units: GDS file units in meters, defaults to 1e-6.
        :param float precision: GDS file precision, defaults to 1e-10.
        :return: GDS file path
        """
    @staticmethod
    def from_gds(file_name: PathLike) -> Library:
        """Read a Library from a GDS file.

        :param PathLike file_name: Input GDS file name.
        :return: Library
        """
    def __add__(self, other: Cell) -> Self:
        """Add a cell to the library.

        This simple calls the add method with the cell as an argument,
        and replace_pre_existing as True.

        :param Cell other: Cell to add to the library.

        This can be used in the following way:
        ```python
        import gdsr

        library = gdsr.Library()

        cell = gdsr.Cell("cell")

        library = library + cell
        # or
        library += cell
        # or
        library + cell
        ```
        """
    def __contains__(self, cell: Cell) -> bool:
        """Return True if the library contains the cell."""
    def __eq__(self, value: object) -> bool:
        """Return True if the library is equal to another object."""
    def __str__(self) -> str:
        """Return a string representation of the library."""
    def __repr__(self) -> str:
        """Return a string representation of the library."""

__all__ = [
    "Cell",
    "Grid",
    "HorizontalPresentation",
    "Library",
    "Path",
    "PathType",
    "Point",
    "PointIterator",
    "Polygon",
    "Reference",
    "Text",
    "VerticalPresentation",
]
