import pytest
from hypothesis import assume, given
from hypothesis import strategies as st

from gdsr import Point

from .conftest import point_strategy

# Point init

float_strategy = st.floats(allow_nan=False)


@given(x=float_strategy, y=float_strategy)
def test_point_creation(x: float, y: float):
    p = Point(x, y)
    assert p.x == x
    assert p.y == y
    assert p[0] == x
    assert p[1] == y


def test_point_invalid_initialization():
    with pytest.raises(TypeError):
        Point("string", "string")  # type: ignore
    with pytest.raises(TypeError):
        Point(None, None)  # type: ignore


# Point bool
def test_point_bool():
    p = Point(0, 0)
    assert not p


@given(x=float_strategy, y=float_strategy)
def test_point_bool_true(x: float, y: float):
    assume(x != 0 or y != 0)
    p = Point(x, y)
    assert p


# Point eq


@given(p=point_strategy())
def test_point_equal_to_point(p: Point):
    assert p == p


@given(x=st.floats(allow_nan=False), y=float_strategy)
def test_point_equal_to_tuple(x: float, y: float):
    p = Point(x, y)
    assert p == (x, y)


@given(x=float_strategy, y=float_strategy)
def test_point_equal_to_list(x: float, y: float):
    p = Point(x, y)
    assert p == [x, y]


@given(x=float_strategy, y=float_strategy)
def test_point_not_equal_to_point(x: float, y: float):
    p = Point(x, y)
    assert p != Point(x + 1, y)
    assert p != Point(x, y + 1)


@given(x=float_strategy, y=float_strategy)
def test_point_not_equal_to_tuple(x: float, y: float):
    p = Point(x, y)
    assert p != (x + 1, y)
    assert p != (x, y + 1)


@given(x=float_strategy, y=float_strategy)
def test_point_not_equal_to_list(x: float, y: float):
    p = Point(x, y)
    assert p != [x + 1, y]
    assert p != [x, y + 1]


# Point comparison


def test_point_less_than_point():
    p1 = Point(1, 2)
    p2 = Point(2, 1)
    assert p1 < p2


def test_point_less_than_tuple():
    p = Point(1, 2)
    assert p < (2, 1)


def test_point_less_than_list():
    p = Point(1, 2)
    assert p < [2, 1]


def test_point_less_than_or_equal_to_point():
    p1 = Point(1, 2)
    p2 = Point(2, 1)
    assert p1 <= p2
    assert p1 <= p1


def test_point_less_than_or_equal_to_tuple():
    p = Point(1, 2)
    assert p <= (2, 1)
    assert p <= (1, 2)


def test_point_less_than_or_equal_to_list():
    p = Point(1, 2)
    assert p <= [2, 1]
    assert p <= [1, 2]


def test_point_greater_than_point():
    p1 = Point(2, 1)
    p2 = Point(1, 2)
    assert p1 > p2


def test_point_greater_than_tuple():
    p = Point(2, 1)
    assert p > (1, 2)


def test_point_greater_than_list():
    p = Point(2, 1)
    assert p > [1, 2]


def test_point_greater_than_or_equal_to_point():
    p1 = Point(2, 1)
    p2 = Point(1, 2)
    assert p1 >= p2
    assert p1 >= p1


def test_point_greater_than_or_equal_to_tuple():
    p = Point(2, 1)
    assert p >= (1, 2)
    assert p >= (2, 1)


def test_point_greater_than_or_equal_to_list():
    p = Point(2, 1)
    assert p >= [1, 2]
    assert p >= [2, 1]


# Point add
def test_point_add_point():
    p1 = Point(1, 2)
    p2 = Point(3, 4)
    p3 = p1 + p2
    assert p3 == (4, 6)


def test_point_add_tuple():
    p = Point(1, 2)
    t = (3, 4)
    p3 = p + t
    assert p3 == (4, 6)


def test_point_add_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p3 = p + p2
    assert p3 == (4, 6)


def test_point_radd_tuple():
    p = Point(1, 2)
    t = (3, 4)
    p3 = t + p
    assert p3 == (4, 6)


def test_point_radd_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p3 = p2 + p
    assert p3 == (4, 6)


def test_point_iadd_point():
    p = Point(1, 2)
    p += Point(3, 4)
    assert p == (4, 6)


def test_point_iadd_tuple():
    p = Point(1, 2)
    t = (3, 4)
    p += t
    assert p == (4, 6)


def test_point_iadd_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p += p2
    assert p == (4, 6)


def test_point_add_invalid_operand():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p + 3  # type: ignore


# Point sub
def test_point_sub_point():
    p1 = Point(1, 2)
    p2 = Point(3, 4)
    p3 = p1 - p2
    assert p3 == (-2, -2)


def test_point_sub_tuple():
    p = Point(1, 2)
    p2 = (3, 4)
    p3 = p - p2
    assert p3 == (-2, -2)


def test_point_sub_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p3 = p - p2
    assert p3 == (-2, -2)


def test_point_rsub_tuple():
    p = Point(1, 2)
    p2 = (3, 4)
    p3 = p2 - p
    assert p3 == (2, 2)


def test_point_rsub_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p3 = p2 - p
    assert p3 == (2, 2)


def test_point_isub_point():
    p = Point(1, 2)
    p -= Point(3, 4)
    assert p == (-2, -2)


def test_point_isub_tuple():
    p = Point(1, 2)
    p2 = (3, 4)
    p -= p2
    assert p == (-2, -2)


def test_point_isub_list():
    p = Point(1, 2)
    p2 = [3, 4]
    p -= p2
    assert p == (-2, -2)


def test_tuple_isub_point():
    p2 = (3, 4)
    p2 -= Point(1, 2)
    assert p2 == (2, 2)


def test_list_isub_point():
    p2 = [3, 4]
    p2 -= Point(1, 2)
    assert p2 == (2, 2)


# Point mul
def test_point_mul():
    p = Point(1, 2)
    p3 = p * 3
    assert p3 == (3, 6)


def test_point_rmul():
    p = Point(1, 2)
    p3 = 3 * p
    assert p3 == (3, 6)


def test_point_imul():
    p = Point(1, 2)
    p *= 3
    assert p == (3, 6)


# Point truediv
def test_point_truediv():
    p = Point(3, 6)
    p3 = p / 3
    assert p3 == (1, 2)


def test_point_itruediv():
    p = Point(3, 6)
    p /= 3
    assert p == (1, 2)


def test_point_truediv_by_zero():
    p = Point(3, 6)
    with pytest.raises(ZeroDivisionError):
        _ = p / 0


def test_point_floor_div():
    p = Point(4, 7)
    p3 = p // 3
    assert p3 == (1, 2)


def test_point_ifloor_div():
    p = Point(4, 7)
    p //= 3
    assert p == (1, 2)


def test_point_floor_div_by_zero():
    p = Point(4, 7)
    with pytest.raises(ZeroDivisionError):
        _ = p // 0


# Point str
def test_point_str():
    p = Point(1, 2)
    assert str(p) == "Point(1, 2)"


# Point repr


def test_point_repr():
    p = Point(1, 2)
    assert repr(p) == "(1, 2)"


# Point round
def test_point_round():
    p = Point(1.12, 2.28)
    p3 = round(p, 1)
    assert p3 == (1.1, 2.3)


# Point neg
def test_point_neg():
    p = Point(1, 2)
    p3 = -p
    assert p3 == (-1, -2)


def test_negative_point_neg():
    p = Point(-1, -2)
    p3 = -p
    assert p3 == (1, 2)


# Point distance_to
def test_point_distance_to():
    p1 = Point(1, 1)
    p2 = Point(4, 5)
    assert p1.distance_to(p2) == 5


def test_point_distance_to_():
    p = Point(1, 1)
    assert p.distance_to(p) == 0


def test_point_distance_to_tuple():
    p = Point(1, 1)
    assert p.distance_to((4, 5)) == 5


def test_point_large_values():
    p1 = Point(1e10, 1e10)
    p2 = Point(1e10, 1e10)
    assert p1.distance_to(p2) == 0


# Point copy
def test_point_copy():
    p = Point(1, 2)
    p2 = p.copy()
    assert p == p2
    assert p is not p2


# Point frozen
def test_point_is_immutable():
    p = Point(1, 2)
    with pytest.raises(AttributeError):
        p.x = 3  # type: ignore
    with pytest.raises(AttributeError):
        p.y = 3  # type: ignore
    with pytest.raises(TypeError):
        p[0] = 3  # type: ignore
    with pytest.raises(TypeError):
        p[1] = 3  # type: ignore


# Point hash
def test_hash():
    point_dict = {Point(3.0, 4.0): "test"}
    assert point_dict[Point(3.0, 4.0)] == "test"


# Point init with negative values
def test_point_init_negative():
    p = Point(-1, -2)
    assert p.x == -1
    assert p.y == -2


def test_point_init_zero():
    p = Point(0, 0)
    assert p.x == 0
    assert p.y == 0


def test_point_init_large():
    p = Point(1e6, 2e6)
    assert p.x == 1e6
    assert p.y == 2e6


def test_point_repr_negative():
    p = Point(-1, -2)
    assert repr(p) == "(-1, -2)"


# Subtraction with Negative Result
def test_point_sub_negative_result():
    p1 = Point(1, 2)
    p2 = Point(3, 4)
    p3 = p2 - p1
    assert p3 == (2, 2)


# Division by Zero Tests
def test_point_div_by_zero():
    p = Point(1, 2)
    with pytest.raises(ZeroDivisionError):
        _ = p / 0


# Multiplication by Negative
def test_point_mul_by_negative():
    p = Point(1, 2)
    p3 = p * -1
    assert p3 == (-1, -2)


# Comparison with Negative Coordinates
def test_point_compare_negative_coordinates():
    p1 = Point(-1, -2)
    p2 = Point(1, 2)
    assert p1 < p2
    assert p1 <= p2
    assert not (p1 > p2)
    assert not (p1 >= p2)


# Comparison with Other Types
def test_point_compare_with_other_types():
    p = Point(1, 2)
    assert not (p == "not a point")
    assert p != "not a point"
    with pytest.raises(TypeError):
        p < "not a point"  # type: ignore
    with pytest.raises(TypeError):
        p <= "not a point"  # type: ignore
    with pytest.raises(TypeError):
        p > "not a point"  # type: ignore
    with pytest.raises(TypeError):
        p >= "not a point"  # type: ignore


# Invalid Addition Operations
def test_point_add_invalid_operand_string():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p + "invalid operand"  # type: ignore


def test_point_iadd_invalid_operand():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p += "invalid operand"  # type: ignore


# Division by Non-Numeric
def test_point_div_by_non_numeric():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p / "non-numeric"  # type: ignore


# Multiplication by Non-Numeric
def test_point_mul_by_non_numeric():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p * "non-numeric"  # type: ignore


# Truediv with Large Coordinates
def test_point_truediv_large_coordinates():
    p = Point(1e6, 2e6)
    p3 = p / 2
    assert p3 == (5e5, 1e6)


# Truediv with Large Result
def test_point_truediv_large_result():
    p = Point(1, 1)
    p3 = p / 0.5
    assert p3 == (2, 2)


# Iteration
def test_point_iter():
    p = Point(1, 2)
    assert list(p) == [1, 2]


# Destructuring
def test_destructure():
    p = Point(1, 2)
    x, y = p
    assert x == 1
    assert y == 2


# Cross Product
def test_cross():
    p1 = Point(1, 2)
    p2 = Point(3, 4)
    assert p1.cross(p2) == -2


def test_cross_zero():
    p1 = Point(1, 0)
    p2 = Point(0, 1)
    assert p1.cross(p2) == 1


def test_cross_negative():
    p1 = Point(1, 2)
    p2 = Point(4, 3)
    assert p1.cross(p2) == -5


def test_cross_negative_zero():
    p1 = Point(1, 0)
    p2 = Point(0, -1)
    assert p1.cross(p2) == -1


def test_cross_zero_negative():
    p1 = Point(0, 1)
    p2 = Point(-1, 0)
    assert p1.cross(p2) == 1


def test_cross_invalid_operand():
    p1 = Point(1, 2)
    with pytest.raises(TypeError):
        p1.cross("invalid operand")  # type: ignore


def test_cross_tuple():
    p1 = Point(1, 2)
    assert p1.cross((3, 4)) == -2


#  rotate


def test_point_rotate():
    p = Point(1, 0)
    p2 = p.rotate(90)
    assert p2 == (0, 1)


def test_point_rotate_180():
    p = Point(1, 0)
    p2 = p.rotate(180)
    assert p2 == (-1, 0)


def test_point_rotate_270():
    p = Point(1, 0)
    p2 = p.rotate(270)
    assert p2 == (0, -1)


def test_point_rotate_360():
    p = Point(1, 0)
    p2 = p.rotate(360)
    assert p2 == (1, 0)


def test_point_rotate_negative():
    p = Point(1, 0)
    p2 = p.rotate(-90)
    assert p2 == (0, -1)


def test_point_rotate_large():
    p = Point(1, 0)
    p2 = p.rotate(720)
    assert p2 == (1, 0)


def test_point_rotate_large_negative():
    p = Point(1, 0)
    p2 = p.rotate(-720)
    assert p2 == (1, 0)


# Scale


def test_point_scale():
    p = Point(1, 2)
    p2 = p.scale(2)
    assert p2 == (2, 4)


def test_point_scale_negative():
    p = Point(1, 2)
    p2 = p.scale(-2)
    assert p2 == (-2, -4)


def test_point_scale_zero():
    p = Point(1, 2)
    p2 = p.scale(0)
    assert p2 == (0, 0)


def test_point_scale_centre():
    p = Point(1, 2)
    p2 = p.scale(2, Point(1, 1))
    assert p2 == (1, 3)


def test_point_scale_centre_negative():
    p = Point(1, 2)
    p2 = p.scale(-2, Point(1, 1))
    assert p2 == (1, -1)


def test_point_scale_centre_zero():
    p = Point(1, 2)
    p2 = p.scale(0, Point(1, 1))
    assert p2 == (1, 1)


def test_point_scale_invalid_operand():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p.scale("invalid operand", Point(1, 1))  # type: ignore


def test_point_scale_invalid_centre():
    p = Point(1, 2)
    with pytest.raises(TypeError):
        p.scale(2, "invalid centre")  # type: ignore
