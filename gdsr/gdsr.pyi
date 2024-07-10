from typing import Union, List

from .path import Path
from .polygon import Polygon

class ArrayReference:
    def __init__(self) -> None: ...

class Reference:
    def __init__(self) -> None: ...

class Box:
    def __init__(self) -> None: ...

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
