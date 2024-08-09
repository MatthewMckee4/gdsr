from typing import TYPE_CHECKING

from hypothesis import strategies as st

from gdsr import (
    Cell,
    Grid,
    HorizontalPresentation,
    Library,
    Path,
    PathType,
    Point,
    Polygon,
    Reference,
    Text,
    VerticalPresentation,
)

if TYPE_CHECKING:
    from gdsr import Element, Instance


@st.composite
def float_strategy(draw: st.DrawFn) -> float:
    return float(
        draw(
            st.decimals(
                min_value=-214748.3648,
                max_value=214748.3647,
                allow_nan=False,
                allow_infinity=False,
                places=4,
            )
        )
    )


@st.composite
def point_strategy(draw: st.DrawFn) -> Point:
    return Point(draw(float_strategy()), draw(float_strategy()))


@st.composite
def layer_strategy(draw: st.DrawFn) -> int:
    return draw(st.integers(min_value=0, max_value=255))


@st.composite
def data_type_strategy(draw: st.DrawFn) -> int:
    return draw(st.integers(min_value=0, max_value=32767))


@st.composite
def cell_strategy(draw: st.DrawFn, *, cell_name: str | None = None) -> Cell:
    if cell_name is not None:
        return Cell(cell_name)
    return Cell(draw(st.text(min_size=1)))


@st.composite
def library_strategy(draw: st.DrawFn) -> Library:
    return Library(draw(st.text(min_size=1)))


@st.composite
def grid_strategy(draw: st.DrawFn) -> Grid:
    return Grid(
        draw(point_strategy()),
        draw(st.integers(min_value=1, max_value=200)),
        draw(st.integers(min_value=1, max_value=200)),
        draw(point_strategy()),
        draw(point_strategy()),
        draw(st.integers()),
        draw(st.integers(min_value=1)),
        draw(st.booleans()),
    )


@st.composite
def polygon_strategy(draw: st.DrawFn) -> Polygon:
    points = draw(st.lists(point_strategy(), min_size=4, max_size=8191))
    if points[0] != points[-1]:
        points.append(points[0])
    return Polygon(points, draw(layer_strategy()), draw(data_type_strategy()))


@st.composite
def path_strategy(draw: st.DrawFn) -> Path:
    width_from_draw = draw(
        st.one_of(
            st.decimals(
                min_value=1,
                max_value=65535,
                allow_infinity=False,
                allow_nan=False,
                places=4,
            ),
            st.none(),
        )
    )

    if width_from_draw is not None:
        width = float(width_from_draw)
    else:
        width = None

    return Path(
        draw(st.lists(point_strategy(), min_size=2)),
        draw(layer_strategy()),
        draw(data_type_strategy()),
        draw(st.sampled_from(PathType.values()) | st.none()),
        width,
    )


@st.composite
def text_strategy(draw: st.DrawFn) -> Text:
    return Text(
        draw(st.text(min_size=1)),
        draw(point_strategy()),
        draw(layer_strategy()),
        draw(st.integers(min_value=1)),
        draw(st.integers()),
        draw(st.booleans()),
        draw(st.sampled_from(VerticalPresentation.values())),
        draw(st.sampled_from(HorizontalPresentation.values())),
    )


@st.composite
def reference_strategy(draw: st.DrawFn) -> "Reference[Instance]":
    return Reference(draw(cell_strategy()))


@st.composite
def instance_param_strategy(draw: st.DrawFn) -> "Instance":
    return draw(
        st.one_of(
            cell_strategy(),
            polygon_strategy(),
            path_strategy(),
            text_strategy(),
            reference_strategy(),
            reference_strategy().map(lambda r: Reference(Reference(r))),
        )
    )


@st.composite
def element_param_strategy(draw: st.DrawFn) -> "Element":
    return draw(
        st.one_of(
            polygon_strategy(),
            path_strategy(),
            text_strategy(),
            reference_strategy(),
        )
    )


def check_references(library: Library, instance: "Instance", new_cell: Cell):
    if isinstance(instance, Cell):
        assert library.cells[instance.name] == instance
    elif isinstance(instance, Polygon):
        assert instance == new_cell.polygons[0]
    elif isinstance(instance, Path):
        assert instance == new_cell.paths[0]
    elif isinstance(instance, Text):
        assert instance == new_cell.texts[0]
    else:
        check_references(library, instance.instance, new_cell)
