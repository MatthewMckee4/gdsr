from gdsr import Cell, Grid, Instance, Library, Path, Polygon, Reference, Text

from .conftest import instance_param, unique_instance_pairs_param

# Reference init


@instance_param
def test_reference_init_with_instance(instance: Instance):
    reference = Reference(instance)
    assert reference.instance == instance
    assert reference.grid == Grid()


@instance_param
def test_reference_init_with_instance_and_grid(instance: Instance, sample_grid: Grid):
    reference = Reference(instance, sample_grid)
    assert reference.instance == instance
    assert reference.grid == sample_grid


# Reference setters


@unique_instance_pairs_param
def test_set_instance(instance: Instance, other_instance: Instance):
    reference = Reference(instance)
    assert reference.instance == instance
    reference.instance = other_instance
    assert reference.instance == other_instance


@instance_param
def test_set_grid(instance: Instance, sample_grid: Grid):
    reference = Reference(instance)
    assert reference.grid == Grid()
    reference.grid = sample_grid
    assert reference.grid == sample_grid


# Reference copy


@instance_param
def test_reference_copy(instance: Instance, sample_grid: Grid):
    reference = Reference(instance, sample_grid)
    new_reference = reference.copy()
    assert reference is not new_reference
    assert reference == new_reference


# Reference move


@instance_param
def test_move_to(instance: Instance):
    reference = Reference(instance)
    reference.move_to((1, 1))
    assert reference.grid.origin == (1, 1)


@instance_param
def test_move_to_multiple(instance: Instance):
    reference = Reference(instance)
    reference.move_to((1, 1)).move_to((2, 2)).move_to((3, 3))
    assert reference.grid.origin == (3, 3)


@instance_param
def test_move_to_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.move_to((1, 1))
    assert reference is new_reference
    assert reference == new_reference


# Reference move by


@instance_param
def test_move_by(instance: Instance):
    reference = Reference(instance)
    reference.move_by((1, 1))
    assert reference.grid.origin == (1, 1)


@instance_param
def test_move_by_multiple(instance: Instance):
    reference = Reference(instance)
    reference.move_by((1, 1)).move_by((2, 2)).move_by((3, 3))
    assert reference.grid.origin == (6, 6)


@instance_param
def test_move_by_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.move_by((1, 1))
    assert reference is new_reference
    assert reference == new_reference


# Reference rotate


@instance_param
def test_rotate(instance: Instance):
    reference = Reference(instance)
    reference.rotate(90)
    assert reference.grid.angle == 90


@instance_param
def test_rotate_multiple(instance: Instance):
    reference = Reference(instance)
    reference.rotate(90).rotate(180).rotate(270)
    assert reference.grid.angle == 180


@instance_param
def test_rotate_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.rotate(90)
    assert reference is new_reference
    assert reference == new_reference


# Reference scale


@instance_param
def test_scale_default_grid(instance: Instance):
    reference = Reference(instance)
    reference.scale(2)
    assert reference.grid == Grid(magnification=2)


@instance_param
def test_scale_custom_grid(instance: Instance, sample_grid: Grid):
    reference = Reference(instance, sample_grid)
    reference.scale(2)
    assert reference.grid == sample_grid.scale(2)


@instance_param
def test_scale_returns_self(instance: Instance):
    reference = Reference(instance)
    new_reference = reference.scale(2)
    assert reference is new_reference
    assert reference == new_reference


# Reference str


@instance_param
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


@instance_param
def test_repr(instance: Instance):
    reference = Reference(instance)
    assert repr(reference) == f"Reference({instance!r})"


# Reference eq


@unique_instance_pairs_param
def test_not_equal(instance: Instance, other_instance: Instance):
    reference = Reference(instance)
    other_reference = Reference(other_instance)
    assert reference != other_reference


# Reference read and write
@instance_param
def test_read_write(instance: Instance):
    cell = Cell("parent")
    reference = Reference(instance)
    cell.add(reference)
    path = cell.to_gds()
    library = Library.from_gds(path)
    new_cell = library.cells["parent"]
    _check_references(library, instance, new_cell)


def _check_references(library: Library, instance: Instance, new_cell: Cell):
    if isinstance(instance, Cell):
        assert library.cells[instance.name] == instance
    elif isinstance(instance, Polygon):
        assert instance == new_cell.polygons[0]
    elif isinstance(instance, Path):
        assert instance == new_cell.paths[0]
    elif isinstance(instance, Text):
        assert instance == new_cell.texts[0]
    else:
        _check_references(library, instance.instance, new_cell)
