from ..typing import OutputPointsLike, InputPointsLike

class Polygon:
    """Polygon object"""
    @property
    def points(self) -> OutputPointsLike:
        """Returns the points of the polygon."""

    @points.setter
    def points(self, points: InputPointsLike) -> None:
        """Sets the points of the polygon."""

    layer: int
    """The layer of the polygon."""

    data_type: int
    """The data type of the polygon."""

    def __init__(
        self, points: InputPointsLike, layer: int = 0, data_type: int = 0
    ) -> None:
        """Initializes the Polygon with points, layer, and data type.

        :param InputPointsLike points: Polygon vertices. Sequence of objects that are indexable at 0 and 1. Must not be empty
        :param int layer: Polygon layer, defaults to 0
        :param int data_type: Polygon data_type, defaults to 0
        """

__all__ = ["Polygon"]
