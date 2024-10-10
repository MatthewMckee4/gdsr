from gdsr import Path, PathType, Polygon, boolean


def test_or():
    square = Polygon([(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)])
    circle = Polygon.ellipse((0.5, 0.5), 0.5, 0.5)
    result = boolean([square], [circle], operation="or")
    assert len(result) == 1
    assert result[0].looks_like(square)


def test_or_with_polygon_and_path():
    path = Path([(0, 0), (0, 1)], width=1)
    polygon = Polygon([(-0.5, 1), (0.5, 1), (0.5, 2), (-0.5, 2)])
    result = boolean([polygon], [path], operation="or")
    assert len(result) == 1
    assert result[0].looks_like(Polygon([(-0.5, 0), (0.5, 0), (0.5, 2), (-0.5, 2)]))


def test_path_intersection_creates_circle():
    path1 = Path([(0, -1), (0, 0)], width=1, path_type=PathType.Round)

    path2 = Path([(0, 0), (0, 1)], width=1, path_type=PathType.Round)
    result = path1 & path2
    expected_polygon = Polygon.ellipse(
        (0, 0), 1 - 0.49760, 1 - 0.49760, n_sides=32
    ).rotate(360 / 32 / 2)

    assert result[0].looks_like(expected_polygon)
