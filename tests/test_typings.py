from gdsr.typings import PointLike


def test_point_dict():
    point = {0: 1, 1: 2}

    assert isinstance(point, PointLike)
    assert point[0] == 1
    assert point[1] == 2


def test_point_tuple():
    point = (1, 2)

    assert isinstance(point, PointLike)
    assert point[0] == 1
    assert point[1] == 2


def test_point_list():
    point = [1, 2]

    assert isinstance(point, PointLike)
    assert point[0] == 1
    assert point[1] == 2
