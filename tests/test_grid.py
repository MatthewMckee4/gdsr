import hypothesis.strategies as st
from hypothesis import assume, given

from gdsr import Grid, Point

from .conftest import float_strategy, grid_strategy, point_strategy, row_col_strategy

# Grid init


@given(
    origin=point_strategy(),
    columns=row_col_strategy(),
    rows=row_col_strategy(),
    spacing_x=point_strategy(),
    spacing_y=point_strategy(),
    magnification=float_strategy(),
    angle=float_strategy(),
    x_reflection=st.booleans(),
)
def test_grid_init(
    origin: Point,
    columns: int,
    rows: int,
    spacing_x: Point,
    spacing_y: Point,
    magnification: float,
    angle: float,
    x_reflection: bool,
):
    grid = Grid(
        origin, columns, rows, spacing_x, spacing_y, magnification, angle, x_reflection
    )
    assert grid.origin == origin
    assert grid.columns == columns
    assert grid.rows == rows
    assert grid.spacing_x == spacing_x
    assert grid.spacing_y == spacing_y
    assert grid.angle == angle
    assert grid.magnification == magnification
    assert grid.x_reflection == x_reflection


# Grid setters


@given(grid=grid_strategy(), origin=point_strategy())
def test_grid_set_origin(grid: Grid, origin: Point):
    assume(grid.origin != origin)
    grid.origin = origin
    assert grid.origin == origin


@given(grid=grid_strategy(), columns=row_col_strategy())
def test_grid_set_columns(grid: Grid, columns: int):
    assume(grid.columns != columns)
    grid.columns = columns
    assert grid.columns == columns


@given(grid=grid_strategy(), rows=row_col_strategy())
def test_grid_set_rows(grid: Grid, rows: int):
    assume(grid.rows != rows)
    grid.rows = rows
    assert grid.rows == rows


@given(grid=grid_strategy(), spacing_x=point_strategy())
def test_grid_set_spacing_x(grid: Grid, spacing_x: Point):
    assume(grid.spacing_x != spacing_x)
    grid.spacing_x = spacing_x
    assert grid.spacing_x == spacing_x


@given(grid=grid_strategy(), spacing_y=point_strategy())
def test_grid_set_spacing_y(grid: Grid, spacing_y: Point):
    assume(grid.spacing_y != spacing_y)
    grid.spacing_y = spacing_y
    assert grid.spacing_y == spacing_y


@given(grid=grid_strategy(), magnification=float_strategy())
def test_grid_set_magnification(grid: Grid, magnification: float):
    assume(grid.magnification != magnification)
    grid.magnification = magnification
    assert grid.magnification == magnification


@given(grid=grid_strategy(), angle=float_strategy())
def test_grid_set_angle(grid: Grid, angle: float):
    assume(grid.angle != angle)
    grid.angle = angle
    assert grid.angle == angle


@given(grid=grid_strategy(), x_reflection=st.booleans())
def test_grid_set_x_reflection(grid: Grid, x_reflection: bool):
    assume(grid.x_reflection != x_reflection)
    grid.x_reflection = x_reflection
    assert grid.x_reflection == x_reflection


# Grid copy


@given(grid=grid_strategy())
def test_grid_copy(grid: Grid):
    grid_copy = grid.copy()
    assert grid_copy == grid
    assert grid_copy is not grid


# Grid move_to


@given(grid=grid_strategy(), point=point_strategy())
def test_grid_move_to(grid: Grid, point: Point):
    grid.move_to(point)
    assert grid.origin == point


@given(grid=grid_strategy(), point=point_strategy())
def test_grid_move_to_returns_grid(grid: Grid, point: Point):
    new_grid = grid.move_to(point)
    assert grid is new_grid


# Grid move_by


@given(grid=grid_strategy(), point=point_strategy())
def test_grid_move_by(grid: Grid, point: Point):
    old_grid = grid.copy()
    grid.move_by(point)
    assert grid.origin == old_grid.origin + point


@given(grid=grid_strategy(), point=point_strategy())
def test_grid_move_by_returns_grid(grid: Grid, point: Point):
    old_grid = grid.copy()
    new_grid = grid.move_by(point)
    assert grid is new_grid
    assert grid.origin == old_grid.origin + point


# Grid rotate


@given(grid=grid_strategy(), angle=float_strategy(), centre=point_strategy())
def test_grid_rotate(grid: Grid, angle: float, centre: Point):
    old_grid = grid.copy()
    grid.rotate(angle, centre)
    assert grid.angle == (old_grid.angle + angle) % 360
    assert grid.origin.is_close(old_grid.origin.rotate(angle, centre))


@given(grid=grid_strategy(), angle=float_strategy(), centre=point_strategy())
def test_grid_rotate_returns_grid(grid: Grid, angle: float, centre: Point):
    new_grid = grid.rotate(angle, centre)
    assert grid is new_grid


# Grid scale


@given(grid=grid_strategy(), factor=float_strategy(), centre=point_strategy())
def test_grid_scale(grid: Grid, factor: float, centre: Point):
    old_grid = grid.copy()
    grid.scale(factor, centre)
    assert grid.origin.is_close(old_grid.origin.scale(factor, centre))
    assert grid.spacing_x.is_close(old_grid.spacing_x.scale(factor, centre))
    assert grid.spacing_y.is_close(old_grid.spacing_y.scale(factor, centre))
    assert grid.magnification == old_grid.magnification * factor


@given(grid=grid_strategy(), factor=float_strategy())
def test_grid_scale_returns_grid(grid: Grid, factor: float):
    new_grid = grid.scale(factor)
    assert grid is new_grid


# Grid eq


@given(grid=grid_strategy())
def test_grid_eq(grid: Grid):
    assert grid == grid


@given(grid=grid_strategy())
def test_grid_eq_to_copy(grid: Grid):
    assert grid == grid.copy()


@given(grid=grid_strategy(), point=point_strategy())
def test_grid_eq_to_different_origin(grid: Grid, point: Point):
    assume(grid.origin != point)
    new_grid = grid.copy()
    new_grid.origin = point
    assert grid != new_grid


# Grid str


@given(grid=grid_strategy())
def test_grid_str(grid: Grid):
    assert str(grid) == (
        f"Grid at {grid.origin!r} with {grid.columns} columns and {grid.rows} rows, "
        f"spacing ({grid.spacing_x!r}, {grid.spacing_y!r}), magnification "
        f"{grid.magnification}, angle {grid.angle}, x_reflection "
        f"{str(grid.x_reflection).lower()}"
    )


# Grid repr


@given(grid=grid_strategy())
def test_grid_repr(grid: Grid):
    assert repr(grid) == (
        f"Grid({grid.origin!r}, {grid.columns}, {grid.rows}, {grid.spacing_x!r}, "
        f"{grid.spacing_y!r}, {grid.magnification}, {grid.angle}, "
        f"{str(grid.x_reflection).lower()})"
    )
