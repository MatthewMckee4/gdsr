from gdsr import Polygon, boolean


def test_or():
    square = Polygon([(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)])
    circle = Polygon.ellipse((0.5, 0.5), 0.5, 0.5)
    result = boolean([square], [circle], operation="or")
    assert len(result) == 1
    assert result[0].looks_like(square)
