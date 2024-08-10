from hypothesis import assume, given

from gdsr import Cell, Grid, Instance, Library, Reference

from .conftest import check_references, grid_strategy, instance_param_strategy

# Reference init


@given(instance=instance_param_strategy())
def test_reference_init_with_instance(instance: Instance):
    reference = Reference(instance)
    assert reference.instance == instance
    assert reference.grid == Grid()


@given(instance=instance_param_strategy(), grid=grid_strategy())
def test_reference_init_with_instance_and_grid(instance: Instance, grid: Grid):
    reference = Reference(instance, grid)
    assert reference.instance == instance
    assert reference.grid == grid


# Reference setters


@given(
    instance=instance_param_strategy(),
    other_instance=instance_param_strategy(),
)
def test_set_instance(instance: Instance, other_instance: Instance):
    assume(instance != other_instance)
    reference = Reference(instance)
    assert reference.instance == instance
    reference.instance = other_instance
    assert reference.instance == other_instance


@given(instance=instance_param_strategy(), grid=grid_strategy())
def test_set_grid(instance: Instance, grid: Grid):
    reference = Reference(instance)
    assert reference.grid == Grid()
    reference.grid = grid
    assert reference.grid == grid


# Reference copy


@given(instance=instance_param_strategy(), grid=grid_strategy())
def test_reference_copy(instance: Instance, grid: Grid):
    reference = Reference(instance, grid)
    new_reference = reference.copy()
    assert reference is not new_reference
    assert reference == new_reference


# Reference move


@given(instance=instance_param_strategy())
def test_move_to(instance: Instance):
    reference = Reference(instance)
    reference.move_to((1, 1))
    assert reference.grid.origin == (1, 1)


@given(instance=instance_param_strategy())
def test_move_to_multiple(instance: Instance):
    reference = Reference(instance)
    reference.move_to((1, 1)).move_to((2, 2)).move_to((3, 3))
    assert reference.grid.origin == (3, 3)


@given(instance=instance_param_strategy())
def test_move_to_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.move_to((1, 1))
    assert reference is new_reference
    assert reference == new_reference


# Reference move by


@given(instance=instance_param_strategy())
def test_move_by(instance: Instance):
    reference = Reference(instance)
    reference.move_by((1, 1))
    assert reference.grid.origin == (1, 1)


@given(instance=instance_param_strategy())
def test_move_by_multiple(instance: Instance):
    reference = Reference(instance)
    reference.move_by((1, 1)).move_by((2, 2)).move_by((3, 3))
    assert reference.grid.origin == (6, 6)


@given(instance=instance_param_strategy())
def test_move_by_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.move_by((1, 1))
    assert reference is new_reference
    assert reference == new_reference


# Reference rotate


@given(instance=instance_param_strategy())
def test_rotate(instance: Instance):
    reference = Reference(instance)
    reference.rotate(90)
    assert reference.grid.angle == 90


@given(instance=instance_param_strategy())
def test_rotate_multiple(instance: Instance):
    reference = Reference(instance)
    reference.rotate(90).rotate(180).rotate(270)
    assert reference.grid.angle == 180


@given(instance=instance_param_strategy())
def test_rotate_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.rotate(90)
    assert reference is new_reference
    assert reference == new_reference


# Reference scale


@given(instance=instance_param_strategy())
def test_scale_default_grid(instance: Instance):
    reference = Reference(instance)
    reference.scale(2)
    assert reference.grid == Grid(magnification=2)


@given(instance=instance_param_strategy(), grid=grid_strategy())
def test_scale_custom_grid(instance: Instance, grid: Grid):
    reference = Reference(instance, grid)
    reference.scale(2)
    assert reference.grid == grid.scale(2)


@given(instance=instance_param_strategy())
def test_scale_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.scale(2)
    assert reference is new_reference
    assert reference == new_reference


# Reference str


@given(instance=instance_param_strategy())
def test_str(instance: Instance):
    reference = Reference(instance)
    grid = reference.grid
    assert str(reference) == (
        f"Reference of {instance} with Grid at {grid.origin!r} with {grid.columns} "
        f"columns and {grid.rows} rows, spacing ({grid.spacing_x}, {grid.spacing_y}), "
        f"magnification {grid.magnification}, angle {grid.angle}, "
        f"x_reflection {str(grid.x_reflection).lower()}"
    )


# Reference repr


@given(instance=instance_param_strategy())
def test_repr(instance: Instance):
    reference = Reference(instance)
    assert repr(reference) == f"Reference({instance!r})"


# Reference eq


@given(instance=instance_param_strategy(), other_instance=instance_param_strategy())
def test_not_equal(instance: Instance, other_instance: Instance):
    assume(instance != other_instance)
    reference = Reference(instance)
    other_reference = Reference(other_instance)
    assert reference != other_reference


# Reference read and write
@given(instance=instance_param_strategy())
def test_read_write(instance: Instance):
    library = Library("library")
    cell = Cell("parent")
    library.add(cell)
    reference = Reference(instance)
    reference_cell = _get_cell_from_recursive_reference(reference)
    cell.add(reference)
    if reference_cell is not None:
        library.add(reference_cell)
    path = library.to_gds()
    new_library = Library.from_gds(path)
    new_cell = new_library.cells["parent"]
    check_references(new_library, instance, new_cell)


def _get_cell_from_recursive_reference(reference: "Reference[Instance]") -> Cell | None:
    if isinstance(reference.instance, Cell):
        return reference.instance
    elif isinstance(reference.instance, Reference):
        return _get_cell_from_recursive_reference(reference.instance)
    return None
