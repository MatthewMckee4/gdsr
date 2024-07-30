from gdsr import Polygon

polygon = Polygon([(0, 0), (1, 0), (1, 1), (0, 1)])

contains = polygon.contains((0.5, 0.5))
