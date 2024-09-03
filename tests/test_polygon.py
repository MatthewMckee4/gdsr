import math
import sys

import hypothesis.strategies as st
import pytest
from hypothesis import assume, given

from gdsr import InputPointsLike, Point, Polygon
from tests.conftest import point_strategy


@pytest.fixture
def sample_points() -> InputPointsLike:
    return [(0.0, 0.0), [1.0, 1.0], {0: 2.0, 1: 2.0}, Point(0, 0)]


# Polygon init


def test_polygon_init(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0, 0)]
    assert polygon.layer == 0
    assert polygon.data_type == 0


def test_polygon_init_with_layer_and_data_type(sample_points: InputPointsLike):
    polygon = Polygon(sample_points, layer=5, data_type=10)
    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0, 0)]
    assert polygon.layer == 5
    assert polygon.data_type == 10


def test_polygon_out_of_bounds_layer():
    with pytest.raises(ValueError, match="Layer must be in the range 0-255"):
        Polygon([(0, 0)], layer=256)


def test_polygon_non_integer_data_type():
    with pytest.raises(TypeError):
        Polygon([(0, 0)], data_type="string")  # type: ignore


def test_polygon_invalid_point_type():
    invalid_points = ["invalid", (1.0, 1.0), (2.0, 2.0)]

    with pytest.raises(TypeError, match="Invalid point format"):
        Polygon(invalid_points)  # type: ignore


def test_polygon_init_invalid_layer():
    with pytest.raises(ValueError, match="Layer must be in the range 0-255"):
        Polygon([(0, 0)], layer=-1)


def test_polygon_empty_points_raises_error():
    with pytest.raises(TypeError, match="Points cannot be empty"):
        Polygon([])


def test_polygon_tuple_points_type():
    invalid_point_data_type = ((1, 2), (3, 4))
    try:
        Polygon(invalid_point_data_type)
    except:  # noqa: E722
        pytest.fail("Polygon should accept tuple points")


# Polygon closing functionality


def test_unclosed_points_gives_closed_polygon():
    points = [Point(0, 0), Point(1, 0), Point(1, 1), Point(0, 1)]
    polygon = Polygon(points)
    assert polygon.points == [(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]


def test_closed_points_gives_closed_polygon():
    points = [Point(0, 0), Point(1, 0), Point(1, 1), Point(0, 1), Point(0, 0)]
    polygon = Polygon(points)
    assert polygon.points == [(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]


# Points setter


def test_polygon_points_setter_invalid_type():
    polygon = Polygon([(0.0, 0.0)])
    with pytest.raises(TypeError, match="Invalid point format"):
        polygon.points = ["invalid", (1.0, 1.0)]  # type: ignore


def test_polygon_setter_empty_tuple():
    polygon = Polygon([(0.0, 0.0)])
    with pytest.raises(TypeError, match="Points cannot be empty"):
        polygon.points = ()


def test_polygon_setter_non_iterable():
    polygon = Polygon([(0.0, 0.0)])
    with pytest.raises(TypeError):
        polygon.points = None  # type: ignore


def test_polygon_points_property(sample_points: InputPointsLike):
    polygon = Polygon(sample_points)

    assert polygon.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0, 0)]

    new_points = [(4.0, 4.0), (5.0, 5.0)]
    polygon.points = new_points  # type: ignore
    assert polygon.points == [(4.0, 4.0), (5.0, 5.0), (4.0, 4.0)]


def test_polygon_properties_after_modification():
    polygon = Polygon([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
    original_area = polygon.area
    original_perimeter = polygon.perimeter

    polygon.points = [
        (0.0, 0.0),
        (2.0, 0.0),
        (2.0, 2.0),
        (0.0, 2.0),
    ]

    assert polygon.area != original_area
    assert polygon.perimeter != original_perimeter


def test_polygon_set_points_method():
    polygon = Polygon([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
    new_points = [(4.0, 4.0), (5.0, 5.0)]
    new_polygon = polygon.set_points(new_points)
    assert polygon.points == [(4.0, 4.0), (5.0, 5.0), (4.0, 4.0)]
    assert new_polygon is polygon


def test_polygon_set_layer_method():
    polygon = Polygon([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
    new_polygon = polygon.set_layer(5)
    assert polygon.layer == 5
    assert new_polygon is polygon


def test_polygon_set_data_type_method():
    polygon = Polygon([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])
    new_polygon = polygon.set_data_type(5)
    assert polygon.data_type == 5
    assert new_polygon is polygon


# Polygon str


def test_str_one_point():
    polygon = Polygon([(0, 0)], layer=0, data_type=0)
    assert (
        str(polygon)
        == "Polygon with 1 point(s), starting at (0, 0) on layer 0, data type 0"
    )


def test_str_two_points():
    polygon = Polygon([(0, 0), (1, 1)], layer=0, data_type=0)
    assert (
        str(polygon)
        == "Polygon with 3 point(s), starting at (0, 0) on layer 0, data type 0"
    )


# Polygon repr


def test_repr_one_point():
    polygon = Polygon([(0, 0)], layer=0, data_type=0)
    assert repr(polygon) == "Polygon([(0, 0), ..., (0, 0)], 0, 0)"


def test_repr_two_points():
    polygon = Polygon([(0, 0), (1, 1)], layer=0, data_type=0)
    assert repr(polygon) == "Polygon([(0, 0), ..., (1, 1)], 0, 0)"


def test_repr_three_points():
    polygon = Polygon([(0, 0), (1, 1), (2, 2)], layer=0, data_type=0)
    assert repr(polygon) == "Polygon([(0, 0), ..., (2, 2)], 0, 0)"


def test_repr_four_points():
    polygon = Polygon([(0, 0), (1, 1), (2, 2), (3, 3)], layer=0, data_type=0)
    assert repr(polygon) == "Polygon([(0, 0), ..., (3, 3)], 0, 0)"


# Bounding box


def test_bounding_box_single_point():
    polygon = Polygon([(0.0, 0.0)], layer=0, data_type=0)
    assert polygon.bounding_box == (Point(0.0, 0.0), Point(0.0, 0.0))


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


def test_bounding_box_with_repeated_points():
    points = [(0.0, 0.0), (0.0, 1.0), (0.0, 1.0), (1.0, 1.0)]
    polygon = Polygon(points)
    assert polygon.bounding_box == ((0.0, 0.0), (1.0, 1.0))


# Area


def test_area_single_point():
    polygon = Polygon([(0.0, 0.0)], layer=0, data_type=0)
    assert polygon.area == 0.0


def test_area_square():
    points = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 1.0


def test_area_triangle():
    points = [(0.0, 0.0), (2.0, 0.0), (1.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 1.0


def test_area_complex_polygon():
    points = [(0.0, 0.0), (3.0, 0.0), (2.0, 1.0), (1.0, 3.0), (-1.0, 2.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 6.5


def test_area_negative_coordinates():
    points = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 4.0


def test_area_minimal_coordinates():
    points = [(0.0, 0.0), (sys.float_info.min, sys.float_info.min)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 0.0


def test_area_maximal_coordinates():
    points = [(0.0, 0.0), (sys.float_info.max, sys.float_info.max)]
    polygon = Polygon(points, layer=0, data_type=0)
    assert polygon.area == 0.0


def test_area_with_overlapping_points():
    points = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (1.0, 0.0)]
    polygon = Polygon(points)
    assert polygon.area == 0


def test_area_mixed_point_formats():
    points = [Point(0, 0), (1, 0), [1, 1], {0: 0, 1: 1}]
    polygon = Polygon(points)
    assert polygon.area == 1


# Perimeter


def test_perimeter_square():
    points = [Point(0, 0), Point(0, 1), Point(1, 1), Point(1, 0)]
    polygon = Polygon(points)
    assert polygon.perimeter == 4.0


def test_perimeter_triangle():
    points = [Point(0, 0), Point(0, 1), Point(1, 0)]
    polygon = Polygon(points)
    assert pytest.approx(polygon.perimeter, rel=1e-9) == 2 + math.sqrt(2)  # type: ignore


def test_perimeter_complex_shape():
    points = [Point(0, 0), Point(0, 2), Point(2, 2), Point(2, 0)]
    polygon = Polygon(points)
    assert polygon.perimeter == 8.0


def test_perimeter_non_convex_polygon():
    points = [Point(0, 0), Point(2, 1), Point(1, 2), Point(0, 1)]
    polygon = Polygon(points)
    assert pytest.approx(polygon.perimeter, rel=1e-9) == (  # type: ignore
        math.sqrt(2) + math.sqrt(2) + math.sqrt(5) + 1
    )


def test_perimeter_single_point():
    polygon = Polygon([(0.0, 0.0)], layer=0, data_type=0)
    assert polygon.perimeter == 0.0


def test_perimeter_two_points():
    polygon = Polygon([(0.0, 0.0), (1.0, 1.0)], layer=0, data_type=0)
    assert polygon.perimeter == 2 * math.sqrt(2.0)


# Equality


def test_polygon_equal():
    polygon1 = Polygon([(0, 0), (1, 1)])
    polygon2 = Polygon([(0, 0), (1, 1)])
    assert polygon1 == polygon2


def test_polygon_not_equal_different_points():
    polygon1 = Polygon([(0, 0), (1, 1)])
    polygon2 = Polygon([(0, 0), (1, 2)])
    assert polygon1 != polygon2


def test_polygon_not_equal_different_layer():
    polygon1 = Polygon([(0, 0), (1, 1)], layer=0)
    polygon2 = Polygon([(0, 0), (1, 1)], layer=1)
    assert polygon1 != polygon2


def test_polygon_not_equal_different_data_type():
    polygon1 = Polygon([(0, 0), (1, 1)], data_type=0)
    polygon2 = Polygon([(0, 0), (1, 1)], data_type=1)
    assert polygon1 != polygon2


def test_polygon_not_equal_different_points_length():
    polygon1 = Polygon([(0, 0), (1, 1)])
    polygon2 = Polygon([(0, 0), (1, 1), (2, 2)])
    assert polygon1 != polygon2


# Containment


@pytest.fixture
def square_polygon():
    points = [Point(0, 0), Point(0, 2), Point(2, 2), Point(2, 0), Point(0, 0)]
    return Polygon(points)


def test_contains_single_point_inside(square_polygon: Polygon):
    assert square_polygon.contains(Point(1, 1))


def test_contains_single_point_on_edge(square_polygon: Polygon):
    assert square_polygon.contains(Point(2, 0))


def test_contains_single_point_outside(square_polygon: Polygon):
    assert not square_polygon.contains(Point(3, 3))


def test_contains_all_points_inside(square_polygon: Polygon):
    points = [Point(1, 1), Point(1, 0.5)]
    assert square_polygon.contains_all(*points)


def test_contains_all_points_some_outside(square_polygon: Polygon):
    points = [Point(1, 1), Point(3, 1)]
    assert not square_polygon.contains_all(*points)


def test_contains_all_points_all_outside(square_polygon: Polygon):
    points = [Point(3, 3), Point(4, 4)]
    assert not square_polygon.contains_all(*points)


def test_contains_any_points_all_inside(square_polygon: Polygon):
    points = [Point(1, 1), Point(1, 0.5)]
    assert square_polygon.contains_any(*points)


def test_contains_any_points_one_inside(square_polygon: Polygon):
    points = [Point(1, 1), Point(3, 1)]
    assert square_polygon.contains_any(*points)


def test_contains_any_points_all_outside(square_polygon: Polygon):
    assert not square_polygon.contains_any(Point(3, 3), Point(4, 4))


def test_contains_on_edge(square_polygon: Polygon):
    assert square_polygon.contains(Point(0, 0))
    assert square_polygon.contains(Point(0, 2))
    assert square_polygon.contains(Point(2, 2))
    assert square_polygon.contains(Point(2, 0))


# On edge


def test_on_edge_single_point_on_corner():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert polygon.on_edge(Point(0, 0))
    assert polygon.on_edge(Point(5, 0))
    assert polygon.on_edge(Point(0, 5))
    assert polygon.on_edge(Point(5, 5))


def test_on_edge_single_point_inside():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert not polygon.on_edge(Point(2, 2))


def test_on_edge_single_point_outside():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert not polygon.on_edge(Point(6, 6))
    assert not polygon.on_edge(Point(-1, -1))


def test_on_edge_all_true():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert polygon.on_edge_all(Point(0, 0), Point(0, 5), Point(5, 0))


def test_on_edge_all_false():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert not polygon.on_edge_all(Point(1, 1), Point(2, 2))


def test_on_edge_any_true():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert polygon.on_edge_any(Point(1, 1), Point(0, 5))
    assert polygon.on_edge_any(Point(2, 2), Point(5, 5))


def test_on_edge_any_false():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert not polygon.on_edge_any(Point(1, 1), Point(2, 2))


def test_on_edge_single_point_with_empty_list():
    polygon = Polygon([Point(0, 0), Point(1, 1)])
    assert polygon.on_edge(Point(0, 0))
    assert polygon.on_edge(Point(1, 1))
    assert polygon.on_edge(Point(0.5, 0.5))


def test_on_edge_points_on_diagonal():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert polygon.on_edge(Point(2.5, 5))
    assert polygon.on_edge(Point(5, 2.5))
    assert polygon.on_edge(Point(2.5, 0))
    assert polygon.on_edge(Point(0, 2.5))


def test_on_edge_varied_shape():
    polygon = Polygon([Point(1, 1), Point(1, 4), Point(4, 4), Point(4, 1)])
    assert polygon.on_edge(Point(1, 1))
    assert polygon.on_edge(Point(1, 2))
    assert not polygon.on_edge(Point(3, 3))
    assert polygon.on_edge(Point(4, 4))


def test_on_edge_rectangular_with_different_points():
    polygon = Polygon([Point(0, 0), Point(0, 10), Point(10, 10), Point(10, 0)])
    assert polygon.on_edge(Point(5, 10))
    assert polygon.on_edge(Point(10, 5))
    assert polygon.on_edge(Point(0, 5))
    assert polygon.on_edge(Point(5, 0))


def test_on_edge_concave_polygon():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(3, 3), Point(5, 5), Point(5, 0)])
    assert polygon.on_edge(Point(3, 3))
    assert polygon.on_edge(Point(0, 0))
    assert polygon.on_edge(Point(5, 0))


def test_on_edge_with_non_point_objects():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    with pytest.raises(TypeError):
        polygon.on_edge("not_a_point")  # type: ignore


def test_on_edge_large_polygon():
    points = (
        [Point(i, 0) for i in range(1000)]
        + [Point(1000, i) for i in range(1001)]
        + [Point(i, 1000) for i in range(999, -1, -1)]
        + [Point(0, i) for i in range(1000, -1, -1)]
    )
    polygon = Polygon(points)
    assert polygon.on_edge(Point(500, 0))
    assert polygon.on_edge(Point(1000, 500))
    assert not polygon.on_edge(Point(500, 500))


def test_on_edge_collinear_points():
    polygon = Polygon([Point(0, 0), Point(5, 5), Point(10, 10)])
    assert polygon.on_edge(Point(2, 2))
    assert polygon.on_edge(Point(7, 7))


def test_on_edge_with_negative_coordinates():
    polygon = Polygon([Point(-5, -5), Point(-5, 0), Point(0, 0), Point(0, -5)])
    assert polygon.on_edge(Point(-5, -2))
    assert not polygon.on_edge(Point(-3, -3))
    assert polygon.on_edge(Point(0, -5))


# Copy


def test_polygon_copy_is_equal():
    polygon = Polygon([Point(0, 0), Point(1, 1)])
    copied_polygon = polygon.copy()
    assert polygon == copied_polygon
    assert polygon is not copied_polygon
    assert polygon.points == copied_polygon.points
    assert polygon.layer == copied_polygon.layer
    assert polygon.data_type == copied_polygon.data_type


# Intersect


def test_polygon_intersects_self():
    polygon = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    assert polygon.intersects(polygon)


def test_polygon_intersects_other():
    polygon1 = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    polygon2 = Polygon([Point(2, 2), Point(2, 7), Point(7, 7), Point(7, 2)])
    assert polygon1.intersects(polygon2)


def test_polygon_intersects_other_with_shared_edge():
    polygon1 = Polygon([Point(0, 0), Point(0, 5), Point(5, 5), Point(5, 0)])
    polygon2 = Polygon([Point(5, 0), Point(5, 5), Point(10, 5), Point(10, 0)])
    assert polygon1.intersects(polygon2)


# Rotate


def test_rotate_90_degrees():
    points = [(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(90)

    expected_points = [(0, 0), (-1, 0), (-1, 1), (0, 1), (0, 0)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_180_degrees():
    points = [(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(180)

    expected_points = [(0, 0), (0, -1), (-1, -1), (-1, 0), (0, 0)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_270_degrees():
    points = [(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(270)

    expected_points = [(0, 0), (1, 0), (1, -1), (0, -1), (0, 0)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_with_centre():
    points = [(1, 1), (1, 2), (2, 2), (2, 1), (1, 1)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(90, centre=(1, 1))

    expected_points = [(1, 1), (0, 1), (0, 2), (1, 2), (1, 1)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_with_centre_and_negative_angle():
    points = [(1, 1), (1, 2), (2, 2), (2, 1), (1, 1)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(-90, centre=(1, 1))

    expected_points = [(1, 1), (2, 1), (2, 0), (1, 0), (1, 1)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_with_centre_and_large_angle():
    points = [(1, 1), (1, 2), (2, 2), (2, 1), (1, 1)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(270, (2, 2))

    expected_points = [(1, 3), (2, 3), (2, 2), (1, 2), (1, 3)]
    for point, expected_point in zip(rotated_polygon.points, expected_points):
        assert point.is_close(expected_point)


def test_rotate_no_change():
    points = [(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(0)

    for point, expected_point in zip(rotated_polygon.points, points):
        assert point.is_close(expected_point)


def test_rotate_full_circle():
    points = [(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)]
    polygon = Polygon(points)

    rotated_polygon = polygon.rotate(360)

    for point, expected_point in zip(rotated_polygon.points, points):
        assert point.is_close(expected_point)


def test_rotate_invalid_angle():
    points = [(0, 0), (0, 1), (1, 1), (1, 0)]
    polygon = Polygon(points)

    with pytest.raises(TypeError):
        polygon.rotate("not_a_number")  # type: ignore


# Polygon move


def test_move_to_returns_self():
    polygon = Polygon([(0, 0), (1, 0), (1, 1), (0, 1)])
    new_polygon = polygon.move_to((1, 1))
    assert polygon is new_polygon
    assert polygon == new_polygon
    assert polygon.points[0] == (1, 1)


def test_move_by_returns_self():
    polygon = Polygon([(0, 0), (1, 0), (1, 1), (0, 1)])
    new_polygon = polygon.move_by((1, 1))
    assert polygon is new_polygon
    assert polygon == new_polygon
    assert polygon.points[0]


# Polygon regular


@given(
    centre=point_strategy(),
    radius=st.floats(min_value=0.1, max_value=1000),
    n_sides=st.integers(min_value=3, max_value=12),
    rotation=st.floats(min_value=0, max_value=360),
    layer=st.integers(min_value=0, max_value=10),
    datatype=st.integers(min_value=0, max_value=10),
)
def test_regular_polygon(
    centre: Point,
    radius: float,
    n_sides: int,
    rotation: float,
    layer: int,
    datatype: int,
):
    polygon = Polygon.regular(centre, radius, n_sides, rotation, layer, datatype)

    assert len(polygon.points) == n_sides + 1

    for point in polygon.points:
        distance = ((point.x - centre.x) ** 2 + (point.y - centre.y) ** 2) ** 0.5
        assert math.isclose(distance, radius, rel_tol=1e-5)

    assert polygon.layer == layer
    assert polygon.data_type == datatype

    first_side_length = abs(polygon.points[1].distance_to(polygon.points[0]))
    assert all(
        math.isclose(
            abs(polygon.points[i].distance_to(polygon.points[i - 1])),
            first_side_length,
            rel_tol=1e-5,
        )
        for i in range(1, n_sides)
    )


# Polygon ellipse


@given(
    centre=point_strategy(),
    horizontal_radius=st.floats(min_value=0.5, max_value=100),
    vertical_radius=st.one_of(st.floats(min_value=0.5, max_value=100), st.none()),
    initial_angle=st.floats(min_value=0, max_value=360),
    final_angle=st.floats(min_value=10, max_value=360),
    n_sides=st.sampled_from([1000]),
    layer=st.integers(min_value=0, max_value=10),
    data_type=st.integers(min_value=0, max_value=10),
)
def test_ellipse(
    centre: Point,
    horizontal_radius: float,
    vertical_radius: float | None,
    initial_angle: float,
    final_angle: float,
    n_sides: int,
    layer: int,
    data_type: int,
):
    assume(initial_angle < final_angle)

    polygon = Polygon.ellipse(
        centre,
        horizontal_radius,
        vertical_radius,
        initial_angle,
        final_angle,
        n_sides,
        layer,
        data_type,
    )

    assert len(polygon.points) > 1

    for point in polygon.points:
        if point == centre:
            continue

        x = point.x - centre.x
        y = point.y - centre.y

        angle = math.degrees(math.atan2(y, x))

        if angle < 0:
            angle += 360

        if vertical_radius is not None:
            assert math.isclose(
                (x / horizontal_radius) ** 2 + (y / vertical_radius) ** 2,
                1,
                rel_tol=1e-5,
            )

    assert polygon.layer == layer
    assert polygon.data_type == data_type
