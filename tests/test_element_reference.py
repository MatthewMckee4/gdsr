from gdsr import ElementReference, Polygon


def test_move_to_returns_self():
    element = ElementReference(Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))
    new_element = element.move_to((1, 1))
    assert element is new_element
    assert element == new_element
    assert element.grid.origin == (1, 1)


def test_move_by_returns_self():
    element = ElementReference(Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))
    new_element = element.move_by((1, 1))
    assert element is new_element
    assert element == new_element
    assert element.grid.origin == (1, 1)
