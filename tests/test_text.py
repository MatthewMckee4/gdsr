import hypothesis.strategies as st
import pytest
from hypothesis import assume, given

from gdsr import HorizontalPresentation, Point, Text, VerticalPresentation

from .conftest import float_strategy, layer_strategy, point_strategy, text_strategy

# Text init


@given(
    text=st.text(),
    origin=point_strategy(),
    layer=layer_strategy(),
    magnification=float_strategy(),
    angle=float_strategy(),
    x_reflection=st.booleans(),
    vertical_presentation=st.sampled_from(VerticalPresentation.values()),
    horizontal_presentation=st.sampled_from(HorizontalPresentation.values()),
)
def test_text_init_with_all_parameters(
    text: str,
    origin: Point,
    layer: int,
    magnification: float,
    angle: float,
    x_reflection: bool,
    vertical_presentation: VerticalPresentation,
    horizontal_presentation: HorizontalPresentation,
):
    text_obj = Text(
        text,
        origin=origin,
        layer=layer,
        magnification=magnification,
        angle=angle,
        x_reflection=x_reflection,
        vertical_presentation=vertical_presentation,
        horizontal_presentation=horizontal_presentation,
    )
    assert text_obj.text == text
    assert text_obj.origin == origin
    assert text_obj.layer == layer
    assert text_obj.magnification == magnification
    assert text_obj.angle == angle
    assert text_obj.x_reflection == x_reflection
    assert text_obj.vertical_presentation == vertical_presentation
    assert text_obj.horizontal_presentation == horizontal_presentation


# Text setters


@given(text=st.text())
def test_text_text_setter(text: str):
    text_obj = Text("Hello, World!")
    text_obj.text = text
    assert text_obj.text == text


@given(text=st.text())
def test_text_text_setter_method(text: str):
    text_obj = Text("Hello, World!")
    new_text_obj = text_obj.set_text(text)
    assert text_obj.text == text
    assert new_text_obj is text_obj


@given(origin=point_strategy())
def test_text_origin_setter(origin: Point):
    text = Text("Hello, World!")
    text.origin = origin
    assert text.origin == origin


@given(origin=point_strategy())
def test_text_origin_setter_method(origin: Point):
    text = Text("Hello, World!")
    new_text = text.set_origin(origin)
    assert text.origin == origin
    assert new_text is text


def test_text_origin_setter_invalid():
    text = Text("Hello, World!")
    with pytest.raises(TypeError):
        text.origin = None  # type: ignore


@given(layer=layer_strategy())
def test_text_layer_setter(layer: int):
    text = Text("Hello, World!")
    text.layer = layer
    assert text.layer == layer


@given(layer=layer_strategy())
def test_text_layer_setter_method(layer: int):
    text = Text("Hello, World!")
    new_text = text.set_layer(layer)
    assert text.layer == layer
    assert new_text is text


@given(magnification=float_strategy())
def test_text_magnification_setter(magnification: float):
    text = Text("Hello, World!")
    text.magnification = magnification
    assert text.magnification == magnification


@given(magnification=float_strategy())
def test_text_magnification_setter_method(magnification: float):
    text = Text("Hello, World!")
    new_text = text.set_magnification(magnification)
    assert text.magnification == magnification
    assert new_text is text


@given(angle=float_strategy())
def test_text_angle_setter(angle: float):
    text = Text("Hello, World!")
    text.angle = angle
    assert text.angle == angle


@given(angle=float_strategy())
def test_text_angle_setter_method(angle: float):
    text = Text("Hello, World!")
    new_text = text.set_angle(angle)
    assert text.angle == angle
    assert new_text is text


@given(x_reflection=st.booleans())
def test_text_x_reflection_setter(x_reflection: bool):
    text = Text("Hello, World!")
    text.x_reflection = x_reflection
    assert text.x_reflection == x_reflection


@given(x_reflection=st.booleans())
def test_text_x_reflection_setter_method(x_reflection: bool):
    text = Text("Hello, World!")
    new_text = text.set_x_reflection(x_reflection)
    assert text.x_reflection == x_reflection
    assert new_text is text


@given(vertical_presentation=st.sampled_from(VerticalPresentation.values()))
def test_text_vertical_presentation_setter(vertical_presentation: VerticalPresentation):
    text = Text("Hello, World!")
    text.vertical_presentation = vertical_presentation
    assert text.vertical_presentation == vertical_presentation


@given(vertical_presentation=st.sampled_from(VerticalPresentation.values()))
def test_text_vertical_presentation_setter_method(
    vertical_presentation: VerticalPresentation,
):
    text = Text("Hello, World!")
    new_text = text.set_vertical_presentation(vertical_presentation)
    assert text.vertical_presentation == vertical_presentation
    assert new_text is text


@given(horizontal_presentation=st.sampled_from(HorizontalPresentation.values()))
def test_text_horizontal_presentation_setter(
    horizontal_presentation: HorizontalPresentation,
):
    text = Text("Hello, World!")
    text.horizontal_presentation = horizontal_presentation
    assert text.horizontal_presentation == horizontal_presentation


@given(horizontal_presentation=st.sampled_from(HorizontalPresentation.values()))
def test_text_horizontal_presentation_setter_method(
    horizontal_presentation: HorizontalPresentation,
):
    text = Text("Hello, World!")
    new_text = text.set_horizontal_presentation(horizontal_presentation)
    assert text.horizontal_presentation == horizontal_presentation
    assert new_text is text


# Text str


def test_text_str_with_different_states():
    text = Text("Hello, World!", angle=30.0, magnification=1.2)
    assert (
        str(text)
        == "Text 'Hello, World!' vertical: Middle, horizontal: Centre at (0, 0)"
    )
    text.angle = 45.0
    assert (
        str(text)
        == "Text 'Hello, World!' vertical: Middle, horizontal: Centre at (0, 0)"
    )


# Text repr


def test_text_repr_with_different_states():
    text = Text("Hello, World!", magnification=2)
    assert repr(text) == "Text(Hello, World!, (0, 0), 0, 2.0, 0, false, Middle, Centre)"


# Text eq


@given(text=text_strategy())
def test_text_equality(text: Text):
    assert text == text


@given(text=text_strategy())
def test_text_equality_with_same_values(text: Text):
    text_copy = text.copy()
    assert text == text_copy


@given(text=text_strategy())
def test_text_equality_with_different_values(text: Text):
    assume(text.text != "Hello, World!")
    text_copy = text.copy()
    text_copy.text = "Hello, World!"
    assert text != text_copy


# Text copy


def test_text_copy():
    text = Text("Hello, World!")
    text_copy = text.copy()
    assert text == text_copy
    assert text is not text_copy


def test_text_copy_with_different_states():
    text = Text("Hello, World!", magnification=1.0)
    text_copy = text.copy()
    text.magnification = 2.0
    assert text_copy.magnification == 1.0
    assert text.magnification == 2.0


def test_text_copy_immutability():
    text = Text("Hello, World!", vertical_presentation=VerticalPresentation.Top)
    text_copy = text.copy()
    text_copy.vertical_presentation = VerticalPresentation.Bottom
    assert text.vertical_presentation == VerticalPresentation.Top
    assert text_copy.vertical_presentation == VerticalPresentation.Bottom


# Text vertical presentation


def test_vertical_presentation_init():
    vertical_presentation = VerticalPresentation.Top
    assert vertical_presentation == VerticalPresentation.Top


def test_vertical_presentation_str():
    vertical_presentation = VerticalPresentation.Top
    assert str(vertical_presentation) == "Vertical Top"


def test_vertical_presentation_repr():
    vertical_presentation = VerticalPresentation.Top
    assert repr(vertical_presentation) == "Top"


def test_vertical_presentation_equality():
    vertical_presentation1 = VerticalPresentation.Top
    vertical_presentation2 = VerticalPresentation.Top
    assert vertical_presentation1 == vertical_presentation2


# Text move to


def test_move_to_returns_self():
    text = Text("Hello, World!")
    new_text = text.move_to((1, 1))
    assert text is new_text
    assert text == new_text
    assert text.origin == (1, 1)


# Text move by


def test_move_by_returns_self():
    text = Text("Hello, World!")
    new_text = text.move_by((1, 1))
    assert text is new_text
    assert text == new_text
    assert text.origin == (1, 1)
