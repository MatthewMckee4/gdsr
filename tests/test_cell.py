from gdsr import Cell, Polygon


def test_cell_initialization():
    cell = Cell("test_cell")
    assert cell.name == "test_cell"
    assert isinstance(cell.array_references, list)
    assert cell.array_references == []
    assert isinstance(cell.polygons, list)
    assert cell.polygons == []
    assert isinstance(cell.boxes, list)
    assert cell.boxes == []
    assert isinstance(cell.nodes, list)
    assert cell.nodes == []
    assert isinstance(cell.paths, list)
    assert cell.paths == []
    assert isinstance(cell.references, list)
    assert cell.references == []
    assert isinstance(cell.texts, list)
    assert cell.texts == []


# Add element
def test_add_polygon():
    cell = Cell("test_cell")
    polygon = Polygon([(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)])
    cell.add(polygon)
    assert cell.polygons == [polygon]


# Remove element


def test_remove_polygon():
    cell = Cell("test_cell")
    polygon = Polygon([(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)])
    cell.add(polygon)
    cell.remove(polygon)
    assert cell.polygons == []


# String representation


def test_str():
    cell = Cell("test_cell")
    assert str(cell) == "Cell: test_cell"


def test_repr():
    cell = Cell("test_cell")
    assert repr(cell) == "test_cell"
