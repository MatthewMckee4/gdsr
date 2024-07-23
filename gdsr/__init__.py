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
    "Library",
    "Path",
    "Point",
    "PointIterator",
    "Polygon",
    "Text",
    "VerticalPresentation",
    # typings
    "InputPointsLike",
    "PointLike",
]
