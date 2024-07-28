from gdsr import ElementReference, Polygon


def test_move_to_returns_self():
    element = ElementReference(Polygon([(0, 0), (1, 0), (1, 1), (0, 1)]))
    new_element = element.move_to((1, 1))
    assert element is not new_element
    assert element == new_element
    assert element.grid.origin == (1, 1)
