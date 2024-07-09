import pytest
from gdsr import Polygon
from gdsr.types import InputPointsLike


@pytest.fixture
def sample_points() -> InputPointsLike:
    return [(0.0, 0.0), [1.0, 1.0], {0: 2.0, 1: 2.0}, [3.0, 3.0]]


def test_polygon_init(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]
    assert polygon.layer == 0
    assert polygon.data_type == 0


def test_polygon_init_with_layer_and_data_type(sample_points: InputPointsLike):
    polygon = Polygon(sample_points, layer=5, data_type=10)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]
    assert polygon.layer == 5
    assert polygon.data_type == 10


def test_polygon_points_property(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)

    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)]

    new_points = [(4.0, 4.0), (5.0, 5.0)]
    polygon.points = new_points
    assert polygon.points == new_points


def test_polygon_empty_points_raises_error():
    with pytest.raises(TypeError, match="Points list cannot be empty"):
        Polygon([])


def test_polygon_invalid_points_format():
    invalid_points = [
        "invalid",
        [(1.0, 1.0), (2.0, 2.0)],
        [(1.0,)],
    ]

    with pytest.raises(TypeError, match="Invalid point format"):
        Polygon(invalid_points)
