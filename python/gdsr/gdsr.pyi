import typing

class ArrayReference:
    def __init__(self) -> None: ...

class Reference:
    def __init__(self) -> None: ...

class Box:
    def __init__(self) -> None: ...

class Node:
    def __init__(self) -> None: ...

class Path:
    def __init__(self) -> None: ...

class Polygon:
    def __init__(self) -> None: ...

class Text:
    def __init__(self) -> None: ...

Element = typing.Union[ArrayReference, Reference, Box, Node, Path, Polygon, Text]

class Cell:
    @property
    def array_references(self) -> list[ArrayReference]: ...
    @property
    def polygons(self) -> list[Polygon]: ...
    @property
    def boxes(self) -> list[Box]: ...
    @property
    def nodes(self) -> list[Node]: ...
    @property
    def paths(self) -> list[Path]: ...
    @property
    def references(self) -> list[Reference]: ...
    @property
    def texts(self) -> list[Text]: ...
    def __init__(self, name: str) -> None: ...
    def add(self, element: Element) -> None: ...
