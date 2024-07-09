from gdsr import Cell, Polygon

cell = Cell("main")

polygon = Polygon(
    [(1.0, 2.0), [3.0, 4.0], {0: 5.0, 1: 6.0}, [7.0, 8.0]],
    1,
)

print(polygon)

cell.add(*[p for p in [polygon] * 3])

print(cell.polygons)

polygon.points = [(1, 1)]

print(polygon.points)
