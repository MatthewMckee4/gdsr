"""GDSR: GDSII Reader and Writer for Python."""

import sys

if sys.version_info >= (3, 9):
    from typing import TypeAlias
else:
    from typing_extension import TypeAlias

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


Instance: TypeAlias = "Cell | Element"
"""Type alias for a GDSII instance."""


Element: TypeAlias = "Reference[Instance] | Polygon | Path | Text"
"""Type alias for a GDSII element."""
