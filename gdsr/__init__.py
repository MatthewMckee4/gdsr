"""GDSR: GDSII Reader and Writer for Python."""

from .logging import setup_logger

setup_logger()

from ._gdsr import (
    Cell,
    Grid,
    HorizontalPresentation,
    Library,
    Path,
    PathType,
    Point,
    PointIterator,
    Polygon,
    Reference,
    Text,
    VerticalPresentation,
)
from .typings import InputPointsLike, PointLike

Element = Reference | Polygon | Path | Text
Instance = Cell | Element

__all__ = [
    "Cell",
    "Element",
    "Grid",
    "HorizontalPresentation",
    "InputPointsLike",
    "Instance",
    "Library",
    "Path",
    "PathType",
    "Point",
    "PointIterator",
    "PointLike",
    "Polygon",
    "Reference",
    "Text",
    "VerticalPresentation",
]
