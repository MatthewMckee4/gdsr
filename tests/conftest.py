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
def float_strategy(
    draw: st.DrawFn,
    min_value: float = -10000,
    max_value: float = 10000,
    places: int = 4,
) -> float:
    return float(
        draw(
            st.decimals(
                min_value=min_value,
                max_value=max_value,
                allow_nan=False,
                allow_infinity=False,
                places=places,
            )
        )
    )


@st.composite
def row_col_strategy(draw: st.DrawFn, min: int = 1, max: int = 32767) -> int:
    return draw(st.integers(min_value=min, max_value=max))


@st.composite
def string_strategy(draw: st.DrawFn, min_size: int = 1, max_size: int = 100) -> str:
    return rf"{
        draw(
            st.text(
                alphabet=st.characters(codec="ascii"),
                min_size=min_size,
                max_size=max_size,
            )
        )!r
    }"


@st.composite
def point_strategy(
    draw: st.DrawFn,
    min_value: float = -10000,
    max_value: float = 10000,
    places: int = 4,
) -> Point:
    return Point(
        draw(float_strategy(min_value=min_value, max_value=max_value, places=places)),
        draw(float_strategy(min_value=min_value, max_value=max_value, places=places)),
    )


@st.composite
def layer_strategy(draw: st.DrawFn) -> int:
    return draw(st.integers(min_value=0, max_value=255))


@st.composite
def data_type_strategy(draw: st.DrawFn) -> int:
    return draw(st.integers(min_value=0, max_value=32767))


@st.composite
def cell_strategy(draw: st.DrawFn, *, cell_name: str | None = None) -> Cell:
    if cell_name is not None:
        return Cell(rf"{cell_name}")
    return Cell(rf"{draw(string_strategy())}")


@st.composite
def randomly_populated_cell_strategy(
    draw: st.DrawFn, *, cell_name: str | None = None
) -> Cell:
    cell = draw(cell_strategy(cell_name=cell_name))
    num_elements = draw(st.integers(min_value=1, max_value=100))

    for _ in range(num_elements):
        element = draw(
            st.one_of(
                polygon_strategy(),
                path_strategy(),
                text_strategy(),
                reference_strategy(),
            )
        )
        cell.add(element)
    return cell


@st.composite
def library_strategy(draw: st.DrawFn) -> Library:
    return Library(rf"{draw(string_strategy())}")


@st.composite
def angle_strategy(draw: st.DrawFn) -> float:
    return draw(st.sampled_from([0, 45, 90, 180, 270, 359]))


@st.composite
def grid_strategy(
    draw: st.DrawFn, columns_max: int | None = None, rows_max: int | None = None
) -> Grid:
    magnification = float(draw(st.decimals(min_value=1, max_value=10, places=0)))

    if columns_max is not None:
        columns = draw(row_col_strategy(max=columns_max))
    else:
        columns = draw(row_col_strategy())
    if rows_max is not None:
        rows = draw(row_col_strategy(max=rows_max))
    else:
        rows = draw(row_col_strategy())
    return Grid(
        draw(point_strategy(max_value=1000.0, min_value=-1000.0)),
        columns,
        rows,
        draw(point_strategy(max_value=1000.0, min_value=-1000.0)).round(2),
        draw(point_strategy(max_value=1000.0, min_value=-1000.0)).round(2),
        magnification,
        draw(angle_strategy()),
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
        rf"{draw(string_strategy())}",
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


def get_cell_from_recursive_reference(reference: "Reference[Instance]") -> Cell | None:
    if isinstance(reference.instance, Cell):
        return reference.instance
    elif isinstance(reference.instance, Reference):
        return get_cell_from_recursive_reference(reference.instance)
    return None
