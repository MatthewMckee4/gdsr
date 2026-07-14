use std::f64::consts::{FRAC_PI_4, TAU};
use std::io::Write;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use gdsr::{
    Cell, DataType, Element, GdsBox, GdsFileWriter, GdsStreamWriter, Grid, HorizontalPresentation,
    Layer, Library, Node, Path, PathType, Point, Polygon, Property, Radians, Reference, Text, Unit,
    VerticalPresentation,
};
use tempfile::NamedTempFile;

const DATABASE_UNITS: f64 = 1e-9;
const USER_UNITS: f64 = 1e-6;

const PATH_TYPES: [PathType; 3] = [PathType::Square, PathType::Round, PathType::Overlap];
const VERTICAL_PRESENTATIONS: [VerticalPresentation; 3] = [
    VerticalPresentation::Top,
    VerticalPresentation::Middle,
    VerticalPresentation::Bottom,
];
const HORIZONTAL_PRESENTATIONS: [HorizontalPresentation; 3] = [
    HorizontalPresentation::Left,
    HorizontalPresentation::Centre,
    HorizontalPresentation::Right,
];

struct Fixture {
    name: &'static str,
    library: Library,
    encoded_size: u64,
    input_file: NamedTempFile,
    output_file: NamedTempFile,
}

impl Fixture {
    fn new(name: &'static str, library: Library) -> Self {
        let encoded = library
            .write(&GdsFileWriter, USER_UNITS, DATABASE_UNITS)
            .expect("benchmark fixture should serialize");
        let encoded_size = encoded.len() as u64;

        let mut input_file = NamedTempFile::new().expect("benchmark input file should be created");
        input_file
            .write_all(&encoded)
            .expect("benchmark input should be written");
        input_file
            .flush()
            .expect("benchmark input should be flushed");

        let output_file = NamedTempFile::new().expect("benchmark output file should be created");

        Self {
            name,
            library,
            encoded_size,
            input_file,
            output_file,
        }
    }
}

fn point(x: i32, y: i32) -> Point {
    Point::integer(x, y, DATABASE_UNITS)
}

fn polygon(index: i32) -> Polygon {
    let point_count = 4 + index as usize % 5;
    let centre_x = index * 100;
    let radius = 20 + index % 40;
    let points = (0..point_count).map(|point_index| {
        let angle = TAU * point_index as f64 / point_count as f64;
        point(
            centre_x + (f64::from(radius) * angle.cos()) as i32,
            (f64::from(radius) * angle.sin()) as i32,
        )
    });
    Polygon::new(
        points,
        Layer::new((index % 64) as u16),
        DataType::new((index % 8) as u16),
    )
}

fn path(index: i32) -> Path {
    let origin = index * 100;
    Path::new(
        [
            point(origin, 0),
            point(origin + 50, 60),
            point(origin + 100, -20),
            point(origin + 150, 80),
            point(origin + 200, 0),
        ],
        Layer::new((index % 64) as u16),
        DataType::new((index % 8) as u16),
        Some(PATH_TYPES[index as usize % PATH_TYPES.len()]),
        Some(Unit::integer(10 + index % 100, DATABASE_UNITS)),
        None,
        None,
    )
}

fn text(index: i32) -> Text {
    Text::new(
        &format!("label_{index:06}"),
        point(index * 100, index % 500),
        Layer::new((index % 64) as u16),
        DataType::new((index % 8) as u16),
        1.0 + f64::from(index % 5) * 0.25,
        Radians::new(f64::from(index % 8) * FRAC_PI_4),
        index % 2 == 0,
        VERTICAL_PRESENTATIONS[index as usize % VERTICAL_PRESENTATIONS.len()],
        HORIZONTAL_PRESENTATIONS[index as usize % HORIZONTAL_PRESENTATIONS.len()],
    )
}

fn gds_box(index: i32) -> GdsBox {
    let origin = index * 100;
    GdsBox::new(
        point(origin, 0),
        point(origin + 80, 60 + index % 40),
        Layer::new((index % 64) as u16),
        DataType::new((index % 8) as u16),
    )
}

fn node(index: i32) -> Node {
    let origin = index * 100;
    Node::new(
        vec![
            point(origin, 0),
            point(origin + 10, 20),
            point(origin + 30, 10),
            point(origin + 50, 40),
        ],
        Layer::new((index % 64) as u16),
        DataType::new((index % 8) as u16),
    )
}

fn add_properties(element: &mut Element, index: i32) {
    if index % 10 == 0 {
        element.properties_mut().push(Property::new(
            (index % 64) as u16,
            format!("net_{index:06}"),
        ));
    }
}

fn shape(index: i32) -> Element {
    let mut element = match index % 5 {
        0 => polygon(index).into(),
        1 => path(index).into(),
        2 => text(index).into(),
        3 => gds_box(index).into(),
        _ => node(index).into(),
    };
    add_properties(&mut element, index);
    element
}

fn reference(index: i32, target: &str) -> Reference {
    let array = index % 2 == 0;
    Reference::new(target).with_grid(
        Grid::default()
            .with_origin(point(index * 100, index * 20))
            .with_columns(if array { 4 } else { 1 })
            .with_rows(if array { 3 } else { 1 })
            .with_spacing_x(array.then(|| point(1_000, 0)))
            .with_spacing_y(array.then(|| point(0, 800)))
            .with_magnification(1.0 + f64::from(index % 3) * 0.1)
            .with_angle(Radians::new(f64::from(index % 8) * FRAC_PI_4))
            .with_x_reflection(index % 3 == 0),
    )
}

fn mixed_library(element_count: usize) -> Library {
    let mut library = Library::new("bench_mixed");
    let mut leaf = Cell::new("leaf");
    leaf.add(shape(0));
    library.add_cell(leaf);

    let mut top = Cell::new("top");
    for index in 0..element_count {
        let index = index as i32;
        let element = if index % 6 == 5 {
            let mut element = reference(index, "leaf").into();
            add_properties(&mut element, index);
            element
        } else {
            shape(index)
        };
        top.add(element);
    }
    library.add_cell(top);
    library
}

fn hierarchy_library() -> Library {
    const LEAF_CELLS: i32 = 50;
    const SHAPES_PER_LEAF: i32 = 200;

    let mut library = Library::new("bench_hierarchy");
    for cell_index in 0..LEAF_CELLS {
        let mut cell = Cell::new(&format!("leaf_{cell_index:02}"));
        for element_index in 0..SHAPES_PER_LEAF {
            cell.add(shape(cell_index * SHAPES_PER_LEAF + element_index));
        }
        library.add_cell(cell);
    }

    for cell_index in 0..10 {
        let mut cell = Cell::new(&format!("middle_{cell_index:02}"));
        for reference_index in 0..5 {
            let target = (cell_index * 5 + reference_index) % LEAF_CELLS;
            cell.add(reference(reference_index, &format!("leaf_{target:02}")));
        }
        library.add_cell(cell);
    }

    let mut top = Cell::new("top");
    for cell_index in 0..10 {
        top.add(reference(cell_index, &format!("middle_{cell_index:02}")));
    }
    library.add_cell(top);
    library
}

fn fixtures() -> [Fixture; 3] {
    [
        Fixture::new("mixed_10k", mixed_library(10_000)),
        Fixture::new("mixed_50k", mixed_library(50_000)),
        Fixture::new("hierarchy_10k", hierarchy_library()),
    ]
}

fn bench_read_file(c: &mut Criterion, fixtures: &[Fixture]) {
    let mut group = c.benchmark_group("read_file");
    for fixture in fixtures {
        group.throughput(Throughput::Bytes(fixture.encoded_size));
        group.bench_with_input(
            BenchmarkId::from_parameter(fixture.name),
            fixture,
            |b, fixture| {
                b.iter(|| {
                    black_box(
                        Library::read_file(
                            black_box(fixture.input_file.path()),
                            Some(DATABASE_UNITS),
                        )
                        .expect("benchmark fixture should parse"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_read_file_filtered(c: &mut Criterion, fixture: &Fixture) {
    let mut group = c.benchmark_group("read_file_filtered");
    group.throughput(Throughput::Bytes(fixture.encoded_size));
    group.bench_with_input(
        BenchmarkId::from_parameter("mixed_50k_10_percent"),
        fixture,
        |b, fixture| {
            b.iter(|| {
                black_box(
                    Library::read_file_filtered(
                        black_box(fixture.input_file.path()),
                        Some(DATABASE_UNITS),
                        |layer, _| layer.value() % 10 == 0,
                    )
                    .expect("filtered benchmark fixture should parse"),
                )
            });
        },
    );
    group.finish();
}

fn bench_write_bytes(c: &mut Criterion, fixtures: &[Fixture]) {
    let mut group = c.benchmark_group("write_bytes");
    for fixture in fixtures {
        group.throughput(Throughput::Bytes(fixture.encoded_size));
        group.bench_with_input(
            BenchmarkId::from_parameter(fixture.name),
            fixture,
            |b, fixture| {
                b.iter(|| {
                    black_box(
                        fixture
                            .library
                            .write(&GdsFileWriter, USER_UNITS, DATABASE_UNITS)
                            .expect("benchmark library should serialize"),
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_write_stream(c: &mut Criterion, fixture: &Fixture) {
    let mut group = c.benchmark_group("write_stream");
    group.throughput(Throughput::Bytes(fixture.encoded_size));
    group.bench_with_input(
        BenchmarkId::from_parameter(fixture.name),
        fixture,
        |b, fixture| {
            b.iter(|| {
                let output = Vec::with_capacity(fixture.encoded_size as usize);
                let mut writer = GdsStreamWriter::new(
                    output,
                    fixture.library.name(),
                    USER_UNITS,
                    DATABASE_UNITS,
                )
                .expect("stream benchmark should write its header");
                for cell in fixture.library.cells().values() {
                    writer
                        .write_cell(cell)
                        .expect("stream benchmark should write each cell");
                }
                black_box(
                    writer
                        .finish()
                        .expect("stream benchmark should write its footer"),
                )
            });
        },
    );
    group.finish();
}

fn bench_write_file(c: &mut Criterion, fixture: &Fixture) {
    let mut group = c.benchmark_group("write_file");
    group.throughput(Throughput::Bytes(fixture.encoded_size));
    group.bench_with_input(
        BenchmarkId::from_parameter(fixture.name),
        fixture,
        |b, fixture| {
            b.iter(|| {
                fixture
                    .library
                    .write_file(
                        black_box(fixture.output_file.path()),
                        USER_UNITS,
                        DATABASE_UNITS,
                    )
                    .expect("benchmark library should be written");
            });
        },
    );
    group.finish();
}

fn bench_io(c: &mut Criterion) {
    let fixtures = fixtures();
    let mixed_50k = &fixtures[1];
    let hierarchy_10k = &fixtures[2];

    bench_read_file(c, &fixtures);
    bench_read_file_filtered(c, mixed_50k);
    bench_write_bytes(c, &fixtures);
    bench_write_stream(c, hierarchy_10k);
    bench_write_file(c, mixed_50k);
}

criterion_group!(benches, bench_io);
criterion_main!(benches);
