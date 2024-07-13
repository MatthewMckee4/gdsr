from gdsr.typing import PointLike


def test_point_dict():
    point = {0: 1.0, 1: 2.0}

    assert isinstance(point, PointLike)
    assert point[0] == 1.0
    assert point[1] == 2.0


def test_point_tuple():
    point = (1.0, 2.0)

    assert isinstance(point, PointLike)
    assert point[0] == 1.0
    assert point[1] == 2.0


def test_point_list():
    point = [1.0, 2.0]

    assert isinstance(point, PointLike)
    assert point[0] == 1.0
    assert point[1] == 2.0
