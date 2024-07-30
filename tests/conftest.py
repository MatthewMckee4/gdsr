import pytest

from gdsr import (
    Cell,
    Grid,
    HorizontalPresentation,
    Instance,
    Path,
    PathType,
    Point,
    Polygon,
    Reference,
    Text,
    VerticalPresentation,
)

BASIC_CELL = Cell("test_cell")
BASIC_GRID = Grid(Point(10, 10), 2, 3, (10, 0), (0, 10), 45, 2, True)
BASIC_POLYGON = Polygon([(0, 0), (1, 0), (1, 1), (0, 1)], 1, 10)
BASIC_PATH = Path([Point(0, 0), Point(1, 0), Point(1, 1)], 2, 3, PathType.Round, 5)
BASIC_TEXT = Text(
    "test_text",
    (0, 0),
    1,
    5,
    45,
    False,
    VerticalPresentation.Top,
    HorizontalPresentation.Left,
)
BASIC_REFERENCE = Reference(BASIC_CELL)
BASIC_DEEP_REFERENCE = Reference(Reference(Reference(BASIC_REFERENCE)))


@pytest.fixture
def sample_cell() -> Cell:
    return BASIC_CELL


@pytest.fixture
def sample_grid() -> Grid:
    return BASIC_GRID


@pytest.fixture
def sample_polygon() -> Polygon:
    return BASIC_POLYGON


@pytest.fixture
def sample_path() -> Path:
    return BASIC_PATH


@pytest.fixture
def sample_text() -> Text:
    return BASIC_TEXT


@pytest.fixture
def sample_reference() -> Reference:
    return BASIC_REFERENCE


@pytest.fixture
def all_instances(
    sample_cell: Cell,
    sample_polygon: Polygon,
    sample_path: Path,
    sample_text: Text,
    sample_reference: Reference,
) -> list[Instance]:
    return [sample_cell, sample_polygon, sample_path, sample_text, sample_reference]


@pytest.fixture
def unique_instance_pairs(
    all_instances: list[Instance],
) -> list[tuple[Instance, Instance]]:
    return [
        (instance, other_instance)
        for instance in all_instances
        for other_instance in all_instances
        if instance != other_instance
    ]


instance_param = pytest.mark.parametrize(
    "instance",
    [
        BASIC_CELL,
        BASIC_POLYGON,
        BASIC_PATH,
        BASIC_TEXT,
        BASIC_REFERENCE,
        BASIC_DEEP_REFERENCE,
    ],
)


unique_instance_pairs_param = pytest.mark.parametrize(
    "instance,other_instance",
    [
        (inst1, inst2)
        for inst1 in [
            BASIC_CELL,
            BASIC_POLYGON,
            BASIC_PATH,
            BASIC_TEXT,
            BASIC_REFERENCE,
            BASIC_DEEP_REFERENCE,
        ]
        for inst2 in [
            BASIC_CELL,
            BASIC_POLYGON,
            BASIC_PATH,
            BASIC_TEXT,
            BASIC_REFERENCE,
            BASIC_DEEP_REFERENCE,
        ]
        if inst1 != inst2
    ],
)
