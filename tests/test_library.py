import pytest
from hypothesis import assume, given, settings
from hypothesis import strategies as st

from gdsr import Cell, Element, Library, Reference

from .conftest import (
    cell_strategy,
    element_param_strategy,
    get_cell_from_recursive_reference,
    library_strategy,
)

# Library init


@given(st.text())
def test_library_init(name: str):
    lib = Library(name)
    assert lib.name == name


# Library setters


@given(library=library_strategy(), name=st.text())
def test_library_name(library: Library, name: str):
    library.name = name
    assert library.name == name


@given(library=library_strategy(), cell1=cell_strategy(), cell2=cell_strategy())
def test_library_cells_setter_has_no_effect(library: Library, cell1: Cell, cell2: Cell):
    assume(cell1.name != cell2.name)
    cell1_name = cell1.name
    library.add(cell1)
    assert library.cells[cell1_name] is cell1
    library.cells[cell1_name] = cell2  # type: ignore
    assert library.cells[cell1_name] is cell1


# Library add


@given(library=library_strategy(), cell=cell_strategy())
def test_library_add(library: Library, cell: Cell):
    library.add(cell)
    assert library.cells[cell.name] is cell


@given(library=library_strategy(), cell_name=st.text())
def test_library_add_cell_raises_error_when_cell_with_same_name_exists(
    library: Library, cell_name: str
):
    cell1 = Cell(cell_name)
    cell2 = Cell(cell_name)
    library.add(cell1)
    assert library.cells[cell_name] is cell1
    with pytest.raises(ValueError):
        library.add(cell2, replace_pre_existing=False)


@given(library=library_strategy(), cell_name=st.text())
def test_library_add_cell_replaces_pre_existing_cell(library: Library, cell_name: str):
    cell1 = Cell(cell_name)
    cell2 = Cell(cell_name)
    library.add(cell1)
    assert library.cells[cell_name] is cell1
    library.add(cell2, replace_pre_existing=True)
    assert library.cells[cell_name] is cell2


@given(library=library_strategy(), cell=cell_strategy())
def test_library_add_dunder(library: Library, cell: Cell):
    library += cell
    assert library.cells[cell.name] is cell


@given(library=library_strategy(), cell_name=st.text())
def test_library_add_dunder_does_not_raise_error_when_cell_with_same_name_exists(
    library: Library, cell_name: str
):
    cell1 = Cell(cell_name)
    cell2 = Cell(cell_name)
    library += cell1
    assert library.cells[cell_name] is cell1
    library += cell2
    assert library.cells[cell_name] is cell2


# Library remove


@given(library=library_strategy(), cell=cell_strategy())
def test_library_remove(library: Library, cell: Cell):
    library.add(cell)
    assert library.cells[cell.name] is cell
    library.remove(cell)
    assert library.cells.get(cell.name) is None


@given(library=library_strategy(), cell=cell_strategy())
def test_library_remove_does_nothing_when_cell_not_in_library(
    library: Library, cell: Cell
):
    assert library.cells.get(cell.name) is None
    library.remove(cell)
    assert library.cells.get(cell.name) is None


# Library contains


@given(library=library_strategy(), cell=cell_strategy())
def test_library_contains(library: Library, cell: Cell):
    library.add(cell)
    assert cell in library
    assert library.contains(cell)


@given(library=library_strategy(), cell=cell_strategy())
def test_library_contains_returns_false_when_cell_not_in_library(
    library: Library, cell: Cell
):
    library.add(cell)
    library.remove(cell)
    assert cell not in library
    assert not library.contains(cell)


# Library copy


@given(library=library_strategy(), cell=cell_strategy())
def test_library_copy(library: Library, cell: Cell):
    library.add(cell)
    library_copy = library.copy()
    assert library == library_copy
    assert library is not library_copy
    assert library.cells[cell.name] is library_copy.cells[cell.name]


@given(library=library_strategy(), cell=cell_strategy())
def test_library_copy_deep(library: Library, cell: Cell):
    library.add(cell)
    library_copy = library.copy(deep=True)
    assert library == library_copy
    assert library is not library_copy
    assert library.cells[cell.name] is not library_copy.cells[cell.name]


# Library read write


@settings(deadline=None, max_examples=10)
@given(
    library=library_strategy(), cell=cell_strategy(), element=element_param_strategy()
)
def test_library_read_write(library: Library, cell: Cell, element: Element):
    if isinstance(element, Reference):
        instance_cell = get_cell_from_recursive_reference(element)
        if instance_cell is not None:
            assume(instance_cell.name != cell.name)
            library.add(instance_cell)
    cell.add(element)
    library.add(cell)
    new_library = Library.from_gds(library.to_gds())

    assert library.name == new_library.name
    assert library.cells == new_library.cells
    assert library == new_library


# Library eq


@given(library=library_strategy())
def test_library_eq_self(library: Library):
    assert library == library


@given(library1=library_strategy(), library2=library_strategy())
def test_library_with_different_names_not_equal(library1: Library, library2: Library):
    assume(library1.name != library2.name)
    assert library1 != library2


@given(library=library_strategy(), cell=cell_strategy())
def test_library_eq_when_cells_are_the_same(library: Library, cell: Cell):
    library.add(cell)
    library2 = Library(library.name)
    library2.add(cell)
    assert library == library2


@given(library=library_strategy(), cell1=cell_strategy(), cell2=cell_strategy())
def test_library_eq_when_cells_are_different(
    library: Library, cell1: Cell, cell2: Cell
):
    assume(cell1.name != cell2.name)
    library.add(cell1)
    library2 = Library(library.name)
    library2.add(cell2)
    assert library != library2


@given(library=library_strategy(), cell=cell_strategy())
def test_library_eq_when_cells_length_is_different(library: Library, cell: Cell):
    library.add(cell)
    library2 = Library(library.name)
    assert library != library2


# Library str


@given(library=library_strategy())
def test_empty_library_str(library: Library):
    assert str(library) == f"Library '{library.name}' with 0 cells"


@given(library=library_strategy(), cell1=cell_strategy(), cell2=cell_strategy())
def test_library_str(library: Library, cell1: Cell, cell2: Cell):
    assume(cell1.name != cell2.name)
    library.add(cell1)
    library.add(cell2)
    assert str(library) == f"Library '{library.name}' with 2 cells"


# Library repr


@given(library=library_strategy())
def test_library_repr(library: Library):
    assert repr(library) == f"Library({library.name})"
