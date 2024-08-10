import pytest
from hypothesis import assume, given, settings

from gdsr import Cell, Element

from .conftest import element_param_strategy

# Cell init


def test_cell_initialization():
    cell = Cell("test_cell")
    assert cell.name == "test_cell"
    assert isinstance(cell.polygons, list)
    assert cell.polygons == []
    assert isinstance(cell.references, list)
    assert cell.references == []
    assert isinstance(cell.paths, list)
    assert cell.paths == []
    assert isinstance(cell.texts, list)
    assert cell.texts == []


# Cell setters


def test_set_name():
    cell = Cell("test_cell")
    assert cell.name == "test_cell"
    cell.name = "new_name"
    assert cell.name == "new_name"


def test_set_polygons_raises_error():
    cell = Cell("test_cell")
    with pytest.raises(AttributeError):
        cell.polygons = []  # type: ignore


def test_set_references_raises_error():
    cell = Cell("test_cell")
    with pytest.raises(AttributeError):
        cell.references = []  # type: ignore


def test_set_paths_raises_error():
    cell = Cell("test_cell")
    with pytest.raises(AttributeError):
        cell.paths = []  # type: ignore


def test_set_texts_raises_error():
    cell = Cell("test_cell")
    with pytest.raises(AttributeError):
        cell.texts = []  # type: ignore


# Cell add


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_add_polygon(element: Element):
    cell = Cell("test_cell")
    assert not cell.contains(element)
    cell.add(element)
    assert cell.contains(element)


# Cell remove


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_remove_polygon(element: Element):
    cell = Cell("test_cell")
    assert not cell.contains(element)
    cell.add(element)
    assert cell.contains(element)
    cell.remove(element)
    assert not cell.contains(element)


# Cell contains


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_contains(element: Element):
    cell = Cell("test_cell")
    assert not cell.contains(element)
    cell.add(element)
    assert cell.contains(element)
    cell.remove(element)
    assert not cell.contains(element)
    cell.add(element)
    assert cell.contains(element)


# Cell is_empty


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_is_empty(element: Element):
    cell = Cell("test_cell")
    assert cell.is_empty()
    cell.add(element)
    assert not cell.is_empty()
    cell.remove(element)
    assert cell.is_empty()


# Cell copy


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_copy(element: Element):
    cell = Cell("test_cell")
    cell.add(element)
    new_cell = cell.copy()
    assert cell == new_cell
    assert cell is not new_cell


# Cell str


def test_str_empty_cell():
    cell = Cell("test_cell")
    assert (
        str(cell)
        == "Cell: test_cell with 0 polygons, 0 paths, 0 references, and 0 texts"
    )


# Cell repr


def test_repr():
    cell = Cell("test_cell")
    assert repr(cell) == "Cell(test_cell)"


# Cell eq


def test_cell_equal():
    cell = Cell("test_cell")
    new_cell = Cell("test_cell")
    assert cell == new_cell


@settings(max_examples=3)
@given(element=element_param_strategy())
def test_cell_with_element_equal(element: Element):
    cell = Cell("test_cell")
    new_cell = Cell("test_cell")
    cell.add(element)
    new_cell.add(element)
    assert cell == new_cell


@settings(max_examples=3)
@given(element=element_param_strategy(), other_element=element_param_strategy())
def test_cells_with_different_elements_not_equal(
    element: Element, other_element: Element
):
    assume(element != other_element)
    cell = Cell("test_cell")
    new_cell = Cell("test_cell")
    cell.add(element)
    new_cell.add(other_element)
    assert cell != new_cell


def test_cell_not_equal():
    cell = Cell("test_cell")
    new_cell = Cell("new_cell")
    assert cell != new_cell
