use criterion::{Criterion, criterion_group, criterion_main};
use gdsr::{
    Cell, Grid, HorizontalPresentation, Library, Path, PathType, Point, Polygon, Reference, Text,
    Unit, VerticalPresentation,
};

const DB_UNITS: f64 = 1e-9;
const USER_UNITS: f64 = 1e-9;

fn point(x: i32, y: i32) -> Point {
    Point::integer(x, y, DB_UNITS)
}

fn medium_library() -> Library {
    let mut library = Library::new("bench_medium");
    let mut cell = Cell::new("cell");

    for i in 0..100 {
        let offset = i * 100;
        cell.add(Polygon::new(
            vec![
                point(offset, 0),
                point(offset + 50, 0),
                point(offset + 50, 50),
                point(offset, 50),
            ],
            (i % 4) as u16,
            0,
        ));
    }

    for i in 0..50 {
        let y = i * 100;
        cell.add(Path::new(
            vec![point(0, y), point(500, y), point(500, y + 50)],
            (i % 4) as u16,
            0,
            Some(PathType::Square),
            Some(Unit::integer(10, DB_UNITS)),
        ));
    }

    for i in 0..20 {
        cell.add(Text::new(
            &format!("label_{i}"),
            point(i * 200, 0),
            (i % 4) as u16,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::Top,
            HorizontalPresentation::Left,
        ));
    }

    library.add_cell(cell);
    library
}

fn complex_library() -> Library {
    let mut library = Library::new("bench_complex");

    for c in 0..5 {
        let mut cell = Cell::new(&format!("base_{c}"));
        for i in 0..100 {
            let offset = i * 50;
            cell.add(Polygon::new(
                vec![
                    point(offset, 0),
                    point(offset + 30, 0),
                    point(offset + 30, 30),
                    point(offset + 15, 40),
                    point(offset, 30),
                ],
                (i % 8) as u16,
                0,
            ));
        }
        for i in 0..30 {
            cell.add(Path::new(
                vec![
                    point(i * 100, 0),
                    point(i * 100 + 50, 25),
                    point(i * 100 + 100, 0),
                ],
                (i % 4) as u16,
                0,
                Some(PathType::Round),
                Some(Unit::integer(5, DB_UNITS)),
            ));
        }
        for i in 0..10 {
            cell.add(Text::new(
                &format!("text_{c}_{i}"),
                point(i * 300, -50),
                0,
                0,
                1.0,
                0.0,
                false,
                VerticalPresentation::Middle,
                HorizontalPresentation::Centre,
            ));
        }
        library.add_cell(cell);
    }

    for c in 0..5 {
        let mut cell = Cell::new(&format!("array_{c}"));
        cell.add(
            Reference::new(format!("base_{c}")).with_grid(
                Grid::default()
                    .with_origin(point(c * 10000, 0))
                    .with_columns(5)
                    .with_rows(5)
                    .with_spacing_x(Some(point(6000, 0)))
                    .with_spacing_y(Some(point(0, 5000))),
            ),
        );
        library.add_cell(cell);
    }

    library
}

fn bench_write(c: &mut Criterion) {
    let medium = medium_library();
    let complex = complex_library();

    c.bench_function("write_medium", |b| {
        b.iter_with_setup(tempfile::NamedTempFile::new, |f| {
            medium
                .write_file(f.unwrap().path(), DB_UNITS, USER_UNITS)
                .unwrap();
        });
    });

    c.bench_function("write_complex", |b| {
        b.iter_with_setup(tempfile::NamedTempFile::new, |f| {
            complex
                .write_file(f.unwrap().path(), DB_UNITS, USER_UNITS)
                .unwrap();
        });
    });
}

fn bench_read(c: &mut Criterion) {
    let medium = medium_library();
    let complex = complex_library();

    let medium_file = tempfile::NamedTempFile::new().unwrap();
    medium
        .write_file(medium_file.path(), DB_UNITS, USER_UNITS)
        .unwrap();

    let complex_file = tempfile::NamedTempFile::new().unwrap();
    complex
        .write_file(complex_file.path(), DB_UNITS, USER_UNITS)
        .unwrap();

    c.bench_function("read_medium", |b| {
        b.iter(|| Library::read_file(medium_file.path(), Some(DB_UNITS)).unwrap());
    });

    c.bench_function("read_complex", |b| {
        b.iter(|| Library::read_file(complex_file.path(), Some(DB_UNITS)).unwrap());
    });
}

criterion_group!(benches, bench_write, bench_read);
criterion_main!(benches);
