import pytest
from hypothesis import given

from gdsr import InputPointsLike, Path, PathType, Point
from tests.conftest import path_strategy


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


# Path setters


@given(path=path_strategy())
def test_path_points_setter(path: Path):
    new_points = [(1, 1), (2, 2)]
    path.points = new_points  # type: ignore
    assert path.points == new_points


@given(path=path_strategy())
def test_path_points_setter_method(path: Path):
    new_points = [(1, 1), (2, 2)]
    new_path = path.set_points(new_points)
    assert path.points == new_points
    assert new_path is path


@given(path=path_strategy())
def test_path_layer_setter(path: Path):
    path.layer = 5
    assert path.layer == 5


@given(path=path_strategy())
def test_path_layer_setter_method(path: Path):
    new_path = path.set_layer(5)
    assert path.layer == 5
    assert new_path is path


@given(path=path_strategy())
def test_path_data_type_setter(path: Path):
    path.data_type = 5
    assert path.data_type == 5


@given(path=path_strategy())
def test_path_data_type_setter_method(path: Path):
    new_path = path.set_data_type(5)
    assert path.data_type == 5
    assert new_path is path


@given(path=path_strategy())
def test_path_width_setter(path: Path):
    path.width = 5
    assert path.width == 5


@given(path=path_strategy())
def test_path_width_setter_method(path: Path):
    new_path = path.set_width(5)
    assert path.width == 5
    assert new_path is path


@given(path=path_strategy())
def test_path_path_type_setter(path: Path):
    path.path_type = PathType.Overlap
    assert path.path_type == PathType.Overlap


@given(path=path_strategy())
def test_path_path_type_setter_method(path: Path):
    new_path = path.set_path_type(PathType.Overlap)
    assert path.path_type == PathType.Overlap
    assert new_path is path


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


def test_bounding_box_with_overlap_type_vertical_path():
    path = Path([(0, 0), (0, 1)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert bounding_box[0].is_close((-0.25, -0.25))
    assert bounding_box[1].is_close((0.25, 1.25))


def test_bounding_box_with_overlap_type_diagonal_path():
    path = Path([(0, 0), (1, 1)], width=0.5, path_type=PathType.Overlap)
    extra = 0.25 * 2**0.5
    bounding_box = path.bounding_box
    assert bounding_box[0].is_close((-extra, -extra))
    assert bounding_box[1].is_close((1 + extra, 1 + extra))


def test_bounding_box_with_overlap_type_horizontal_path():
    path = Path([(0, 0), (1, 0)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert bounding_box[0].is_close((-0.25, -0.25))
    assert bounding_box[1].is_close((1.25, 0.25))


def test_bounding_box_with_overlap_type_basic_path():
    path = Path([(0, 0), (1, 0), (1, 1)], width=0.5, path_type=PathType.Overlap)
    bounding_box = path.bounding_box
    assert bounding_box[0].is_close((-0.25, -0.25))
    assert bounding_box[1].is_close((1.25, 1.25))


def test_bounding_box_with_overlap_type_complex_path():
    path = Path(
        [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2), (3, 2), (3, 3), (4, 3)],
        width=0.5,
        path_type=PathType.Overlap,
    )
    bounding_box = path.bounding_box
    assert bounding_box[0].is_close((-0.25, -0.25))
    assert bounding_box[1].is_close((4.25, 3.25))


# Path move_to


@given(path=path_strategy())
def test_path_move_to(path: Path):
    path_copy = path.copy()
    new_path = path.move_to((1, 1))
    new_path_points = list(
        map(lambda p: p + (1, 1) - path_copy.points[0], path_copy.points)  # noqa: RUF005
    )
    for point1, point2 in zip(new_path.points, new_path_points):
        assert point1.is_close(point2)


@given(path=path_strategy())
def test_move_to_returns_self(path: Path):
    new_path = path.move_to((1, 1))
    assert path is new_path


# Path move_by


@given(path=path_strategy())
def test_path_move_by(path: Path):
    path_copy = path.copy()
    new_path = path.move_by((1, 1))
    assert new_path.points == list(map(lambda p: p + (1, 1), path_copy.points))  # noqa: RUF005


@given(path=path_strategy())
def test_move_by_returns_self(path: Path):
    new_path = path.move_by((1, 1))
    assert path is new_path


# Path rotate


@given(path=path_strategy())
def test_path_rotate(path: Path):
    path_copy = path.copy()
    new_path = path.rotate(90)
    new_path_points = list(map(lambda p: p.rotate(90), path_copy.points))
    for point1, point2 in zip(new_path.points, new_path_points):
        assert point1.is_close(point2)


@given(path=path_strategy())
def test_rotate_returns_self(path: Path):
    new_path = path.rotate(90)
    assert path is new_path


# Path scale


@given(path=path_strategy())
def test_path_scale(path: Path):
    path_copy = path.copy()
    new_path = path.scale(2)
    new_path_points = list(map(lambda p: p * 2, path_copy.points))
    for point1, point2 in zip(new_path.points, new_path_points):
        assert point1.is_close(point2)


@given(path=path_strategy())
def test_scale_returns_self(path: Path):
    new_path = path.scale(2)
    assert path is new_path


# Path is_on


@given(path=path_strategy())
def test_path_is_on(path: Path):
    assert path.is_on((path.layer, path.data_type))


@given(path=path_strategy())
def test_path_is_not_on_with_different_layer(path: Path):
    assert not path.is_on((path.layer + 1, path.data_type))


@given(path=path_strategy())
def test_path_is_not_on_with_different_data_type(path: Path):
    assert not path.is_on((path.layer, path.data_type + 1))


@given(path=path_strategy())
def test_path_is_not_on_with_different_layer_and_data_type(path: Path):
    assert not path.is_on((path.layer + 1, path.data_type + 1))


@given(path=path_strategy())
def test_path_is_on_with_other_pairs(path: Path):
    assert path.is_on((path.layer, path.data_type), (0, 0), (1, 1))


# Path str


def test_path_string():
    path = Path([(0, 0), (1, 1), (2, 2)])
    assert (
        str(path)
        == "Path with 3 points on layer 0 with data type 0, Square Ends and width 0"
    )


# Path repr


def test_path_repr_two_points():
    path = Path([(0, 0), (1, 1)], layer=4)
    assert repr(path) == "Path([(0, 0), (1, 1)], 4, 0, Square Ends, 0)"


def test_path_repr_three_points():
    path = Path([(0, 0), (1, 1), (2, 2)], width=10)
    assert repr(path) == "Path([(0, 0), ..., (2, 2)], 0, 0, Square Ends, 10)"


def test_path_repr_four_points():
    path = Path([(0, 0), (1, 1), (2, 2), (3, 3)], path_type=PathType.Overlap)
    assert repr(path) == "Path([(0, 0), ..., (3, 3)], 0, 0, Overlap Ends, 0)"


# Path eq


@given(path=path_strategy())
def test_path_eq(path: Path):
    assert path == path


@given(path=path_strategy())
def test_path_eq_to_copy(path: Path):
    assert path == path.copy()


@given(path=path_strategy())
def test_path_eq_to_same_path(path: Path):
    assert path == Path(
        path.points,
        path.layer,
        path.data_type,
        path.path_type,
        path.width,
    )


def test_path_not_eq_to_different_points():
    path1 = Path([(0, 0), (1, 1)])
    path2 = Path([(0, 0), (1, 2)])
    assert path1 != path2
