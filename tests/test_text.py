import pytest

from gdsr import HorizontalPresentation, Text, VerticalPresentation

# Text init


def test_text_init():
    text = Text("Hello, World!")
    assert text.text == "Hello, World!"
    assert text.origin == (0, 0)
    assert text.vertical_presentation == VerticalPresentation.Middle
    assert text.horizontal_presentation == HorizontalPresentation.Centre


def test_text_init_with_presentation():
    text = Text(
        "Hello, World!",
        origin=(10, 20),
        vertical_presentation=VerticalPresentation.Bottom,
        horizontal_presentation=HorizontalPresentation.Left,
    )
    assert text.text == "Hello, World!"
    assert text.origin == (10, 20)
    assert text.vertical_presentation == VerticalPresentation.Bottom
    assert text.horizontal_presentation == HorizontalPresentation.Left


# Angle and Magnification


def test_text_init_with_angle():
    text = Text("Hello, World!", angle=45.0)
    assert text.angle == 45.0


def test_text_init_with_magnification():
    text = Text("Hello, World!", magnification=2.0)
    assert text.magnification == 2.0


# X Reflection Tests


def test_text_init_with_x_reflection():
    text = Text("Hello, World!", x_reflection=True)
    assert text.x_reflection is True


# Full Parameter Initialization Test


def test_text_init_with_all_parameters():
    text = Text(
        "Hello, World!",
        origin=(10, 20),
        layer=1,
        magnification=1.5,
        angle=90.0,
        x_reflection=True,
        vertical_presentation=VerticalPresentation.Top,
        horizontal_presentation=HorizontalPresentation.Right,
    )
    assert text.origin == (10, 20)
    assert text.layer == 1
    assert text.magnification == 1.5
    assert text.angle == 90.0
    assert text.x_reflection is True
    assert text.vertical_presentation == VerticalPresentation.Top
    assert text.horizontal_presentation == HorizontalPresentation.Right


# Origin Setter Tests


def test_text_origin_setter():
    text = Text("Hello, World!")
    text.origin = (5, 5)
    assert text.origin == (5, 5)


def test_text_origin_setter_invalid():
    text = Text("Hello, World!")
    with pytest.raises(TypeError):
        text.origin = None  # type: ignore


# String Representation Tests


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


def test_text_repr_with_different_states():
    text = Text("Hello, World!", magnification=2.0)
    assert repr(text) == "T(Hello, World!)"


# Equality Tests


def test_text_equality():
    text1 = Text("Hello, World!")
    text2 = Text("Hello, World!")
    assert text1 == text2
    assert text1 is not text2


def test_text_inequality():
    text1 = Text("Hello, World!")
    text2 = Text("Goodbye, World!")
    assert text1 != text2


# Copy Functionality Tests


def test_text_copy():
    text = Text("Hello, World!")
    text_copy = text.copy()
    assert text == text_copy
    assert text is not text_copy
    assert text.text == text_copy.text
    assert text.vertical_presentation == text_copy.vertical_presentation
    assert text.horizontal_presentation == text_copy.horizontal_presentation


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


# test vertical presentation


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
