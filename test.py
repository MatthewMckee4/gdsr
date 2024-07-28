from gdsr import Cell, Library, Path

lib = Library()

cell = Cell("cell")

path = Path(
    [(0, 0), (1, 0), (2, 1), (3, 1), (4, 0), (5, 0), (6, 1), (7, 1), (8, 0), (9, 0)],
    width=1,
)

cell.add(path)

cell.to_gds("test.gds")

lib.add(cell)

lib.to_gds("test2.gds")
