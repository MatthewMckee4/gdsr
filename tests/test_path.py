import pytest

from gdsr import InputPointsLike, Path, Point


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
        Path([(0, 0)], layer=256)


def test_path_non_integer_data_type():
    with pytest.raises(TypeError):
        Path([(0, 0)], data_type="string")  # type: ignore


def test_path_invalid_point_type():
    invalid_points = ["invalid", (1.0, 1.0), (2.0, 2.0)]

    with pytest.raises(TypeError, match="Invalid point format"):
        Path(invalid_points)  # type: ignore


def test_path_init_invalid_layer():
    with pytest.raises(ValueError, match="Layer must be in the range 0-255"):
        Path([(0, 0)], layer=-1)


def test_path_empty_points_raises_error():
    with pytest.raises(TypeError, match="Points cannot be empty"):
        Path([])


def test_path_tuple_points_type():
    invalid_point_data_type = ((1, 2), (3, 4))
    try:
        Path(invalid_point_data_type)
    except:  # noqa: E722
        pytest.fail("Path should accept tuple points")


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
