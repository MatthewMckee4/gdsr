import hypothesis.strategies as st
from hypothesis import assume, given, settings

from gdsr import Cell, Element, Grid, Instance, Library, Reference

from .conftest import (
    check_references,
    element_param_strategy,
    get_cell_from_recursive_reference,
    grid_strategy,
    instance_param_strategy,
)

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


# Reference move_to


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


# Reference move_by


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


# Reference flatten


@settings(deadline=None, max_examples=10)
@given(
    element=element_param_strategy(),
    grid=grid_strategy(columns_max=7, rows_max=7),
    cell_name=st.text(),
)
def test_flatten_reference_with_element_depth_one(
    element: Element, grid: Grid, cell_name: str
):
    reference = Reference(element, grid)
    elements = reference.flatten(depth=1)
    assert len(elements) == grid.columns * grid.rows
    assert all(isinstance(new_element, element.__class__) for new_element in elements)

    cell = Cell(repr(cell_name))
    cell.add(reference)

    library = Library.from_gds(cell.to_gds())

    output_cell = library.cells[repr(cell_name)]

    assert all(element in output_cell for element in elements)


# Reference str


@given(instance=instance_param_strategy())
def test_str(instance: Instance):
    reference = Reference(instance)
    grid = reference.grid
    assert str(reference) == f"Reference of {instance} with {grid}"


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
@settings(deadline=None, max_examples=10)
@given(instance=instance_param_strategy())
def test_read_write(instance: Instance):
    library = Library("library")
    cell = Cell("parent")
    library.add(cell)
    reference = Reference(instance)
    reference_cell = get_cell_from_recursive_reference(reference)
    cell.add(reference)
    if reference_cell is not None:
        library.add(reference_cell)
    path = library.to_gds()
    new_library = Library.from_gds(path)
    new_cell = new_library.cells["parent"]
    check_references(new_library, instance, new_cell)
