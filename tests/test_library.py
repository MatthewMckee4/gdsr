from gdsr import (
    Cell,
    Grid,
    HorizontalPresentation,
    Library,
    Path,
    PathType,
    Polygon,
    Reference,
    Text,
    VerticalPresentation,
)

lib = Library("LIBRARY")
child_cell = Cell("CHILD_Cell")
cell = Cell("MAIN_Cell")
cell.add(
    Polygon(
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        layer=1,
        data_type=2,
    )
)
cell.add(
    Path(
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        layer=3,
        data_type=4,
        path_type=PathType.Round,
        width=5,
    )
)
cell.add(
    Text(
        "Hello World",
        (0.5, 0.5),
        layer=6,
        magnification=7,
        angle=8,
        x_reflection=True,
        vertical_presentation=VerticalPresentation.Bottom,
        horizontal_presentation=HorizontalPresentation.Left,
    )
)
cell.add(
    Reference(
        child_cell,
        Grid(
            (0.33, 0.33),
            9,
            10,
            (0.12, 0.34),
            (0.56, 0.78),
            11,
            12,
        ),
    )
)
lib.add(cell)
lib.to_gds("test.gds")

lib = Library.from_gds("test.gds")
