from gdsr import Cell, Grid, Library, Path, Reference

element = Reference(Cell("0"))

print(element.is_on())
grid = Grid((0, 0), 1, 1, (0, 0), (0, 0), 1.0, 0.0, False)
cell_name = ""

reference = Reference(element, grid)
elements = reference.flatten(depth=1)
assert len(elements) == grid.columns * grid.rows
assert all(isinstance(new_element, element.__class__) for new_element in elements)

cell = Cell(repr(cell_name))
cell.add(reference)

library = Library.from_gds(cell.to_gds())

output_cell = library.cells[repr(cell_name)]

assert all(element in output_cell for element in elements)

print(Path([(0, 0), (1, 0), (1, 1), (0, 1), (0, 0)]).is_on())
