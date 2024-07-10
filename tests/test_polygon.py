import pytest
from gdsr import Polygon
from gdsr.typing import InputPointsLike


@pytest.fixture
def sample_points() -> InputPointsLike:
    return [(0.0, 0.0), [1.0, 1.0], {0: 2.0, 1: 2.0}]


def test_polygon_init(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]
    assert polygon.layer == 0
    assert polygon.data_type == 0


def test_polygon_init_with_layer_and_data_type(sample_points: InputPointsLike):
    polygon = Polygon(sample_points, layer=5, data_type=10)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]
    assert polygon.layer == 5
    assert polygon.data_type == 10


def test_polygon_points_property(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)

    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]

    new_points = [(4.0, 4.0), (5.0, 5.0)]
    polygon.points = new_points
    assert polygon.points == new_points


def test_polygon_empty_points_raises_error():
    with pytest.raises(TypeError, match="Points cannot be empty"):
        Polygon([])


def test_polygon_invalid_point_type():
    invalid_points = ["invalid", (1.0, 1.0), (2.0, 2.0)]

    with pytest.raises(TypeError, match="Invalid point format"):
        Polygon(invalid_points)  # type: ignore


def test_polygon_tuple_points_type():
    invalid_point_data_type = ((1, 2), (3, 4))
    try:
        Polygon(invalid_point_data_type)
    except:  # noqa: E722
        pytest.fail("Polygon should accept tuple points")


def test_bounding_box_single_point():
    polygon = Polygon([(0.0, 0.0)], layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (0.0, 0.0))


def test_bounding_box_square():
    points = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (1.0, 1.0))


def test_bounding_box_rectangle():
    points = [(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (0.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (2.0, 1.0))


def test_bounding_box_negative_coordinates():
    points = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((-1.0, -1.0), (1.0, 1.0))


def test_bounding_box_triangle():
    points = [(0.0, 0.0), (2.0, 0.0), (1.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (2.0, 1.0))


def test_bounding_box_horizontal_line():
    points = [(0.0, 0.0), (2.0, 0.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (2.0, 0.0))


def test_bounding_box_vertical_line():
    points = [(0.0, 0.0), (0.0, 2.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (0.0, 2.0))


def test_bounding_box_collinear_points():
    points = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, 0.0), (2.0, 2.0))


def test_bounding_box_complex_polygon():
    points = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (1.5, -1.0), (0.5, -1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.bounding_box == ((0.0, -1.0), (2.0, 1.0))
