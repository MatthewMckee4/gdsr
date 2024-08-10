import math

import pytest

from gdsr import InputPointsLike, Path, PathType, Point
from gdsr.typings import PointLike


@pytest.fixture
def sample_points() -> InputPointsLike:
    return [(0.0, 0.0), [1.0, 1.0], {0: 2.0, 1: 2.0}, Point(0, 0)]


# Path init


def test_path_init(sample_points: InputPointsLike):
    path = Path(sample_points)
    assert path.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0, 0)]
    assert path.layer == 0
    assert path.data_type == 0


def test_path_init_with_layer_and_data_type(sample_points: InputPointsLike):
    path = Path(sample_points, layer=5, data_type=10)
    assert path.points == [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0, 0)]
    assert path.layer == 5
    assert path.data_type == 10


def test_path_out_of_bounds_layer():
    with pytest.raises(ValueError, match="Layer must be in the range 0-255"):
        Path([(0, 0), (0, 1)], layer=256)


def test_path_non_integer_data_type():
    with pytest.raises(TypeError):
        Path([(0, 0), (0, 1)], data_type="string")  # type: ignore


def test_path_invalid_point_type():
    invalid_points = ["invalid", (1.0, 1.0), (2.0, 2.0)]

    with pytest.raises(TypeError, match="Invalid point format"):
        Path(invalid_points)  # type: ignore


def test_path_init_invalid_layer():
    with pytest.raises(ValueError, match="Layer must be in the range 0-255"):
        Path([(0, 0), (0, 1)], layer=-1)


def test_path_empty_points_raises_error():
    with pytest.raises(TypeError, match="Points cannot be empty"):
        Path([])


def test_path_tuple_points_type():
    valid_point_data_type = ((1, 2), (3, 4))
    try:
        Path(valid_point_data_type)
    except:  # noqa: E722
        pytest.fail("Path should accept tuple points")


# Path length


def test_length_basic():
    path = Path([(0, 0), (1, 0), (1, 1)])
    assert path.length == 2.0


def test_length_complex_path():
    path = Path(
        [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2), (3, 2), (3, 3), (4, 3)],
    )
    assert path.length == 7.0


# Path bounding_box


def test_bounding_box_basic():
    path = Path([(0, 0), (1, 0), (1, 1)])
    assert path.bounding_box == ((0, 0), (1, 1))


def test_bounding_box_two_points():
    path = Path([(0, 0), (1, 1)])
    assert path.bounding_box == ((0, 0), (1, 1))


def test_bounding_box_with_negative_points():
    path = Path([(-1, -1), (1, 1)])
    assert path.bounding_box == ((-1, -1), (1, 1))


def assert_almost_equal(point1: PointLike, point2: PointLike, tolerance: float = 1e-9):
    assert math.isclose(point1[0], point2[0], abs_tol=tolerance)
    assert math.isclose(point1[1], point2[1], abs_tol=tolerance)


def test_bounding_box_with_overlap_type_vertical_path():
    path = Path([(0, 0), (0, 1)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert_almost_equal(bounding_box[0], (-0.25, -0.25))
    assert_almost_equal(bounding_box[1], (0.25, 1.25))


def test_bounding_box_with_overlap_type_diagonal_path():
    path = Path([(0, 0), (1, 1)], width=0.5, path_type=PathType.Overlap)
    extra = 0.25 * 2**0.5
    bounding_box = path.bounding_box
    assert_almost_equal(bounding_box[0], (-extra, -extra))
    assert_almost_equal(bounding_box[1], (1 + extra, 1 + extra))


def test_bounding_box_with_overlap_type_horizontal_path():
    path = Path([(0, 0), (1, 0)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert_almost_equal(bounding_box[0], (-0.25, -0.25))
    assert_almost_equal(bounding_box[1], (1.25, 0.25))


def test_bounding_box_with_overlap_type_basic_path():
    path = Path([(0, 0), (1, 0), (1, 1)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert_almost_equal(bounding_box[0], (-0.25, -0.25))
    assert_almost_equal(bounding_box[1], (1.25, 1.25))


def test_bounding_box_with_overlap_type_complex_path():
    path = Path(
        [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2), (3, 2), (3, 3), (4, 3)],
        width=0.5,
        path_type=PathType.Overlap,
    )
    bounding_box = path.bounding_box
    assert_almost_equal(bounding_box[0], (-0.25, -0.25))
    assert_almost_equal(bounding_box[1], (4.25, 3.25))


# Path move


def test_move_to_returns_self():
    path = Path([(0, 0), (1, 0), (1, 1)], layer=5, data_type=10, width=0.5)
    new_path = path.move_to((1, 1))
    assert path is new_path
    assert path == new_path
    assert path.points == [(1, 1), (2, 1), (2, 2)]


def test_move_by_returns_self():
    path = Path([(0, 0), (1, 0), (1, 1)], layer=5, data_type=10, width=0.5)
    new_path = path.move_by((1, 1))
    assert path is new_path
    assert path == new_path
    assert path.points == [(1, 1), (2, 1), (2, 2)]
