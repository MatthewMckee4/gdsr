"""GDSR: GDSII Reader and Writer for Python."""

from .logging import setup_logger

setup_logger()

from ._gdsr import (
    Cell,
    CellReference,
    ElementReference,
    Grid,
    HorizontalPresentation,
    Library,
    Path,
    PathType,
    Point,
    PointIterator,
    Polygon,
    Text,
    VerticalPresentation,
)
from .typings import InputPointsLike, PointLike

__all__ = [
    "Cell",
    "CellReference",
    "ElementReference",
    "Grid",
    "HorizontalPresentation",
    "InputPointsLike",
    "Library",
    "Path",
    "PathType",
    "Point",
    "PointIterator",
    "PointLike",
    "Polygon",
    "Text",
    "VerticalPresentation",
]
