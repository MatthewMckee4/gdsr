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

const PATH_TYPES: [PathType; 3] = [PathType::Square, PathType::Round, PathType::Overlap];
const VERT_PRES: [VerticalPresentation; 3] = [
    VerticalPresentation::Top,
    VerticalPresentation::Middle,
    VerticalPresentation::Bottom,
];
const HORIZ_PRES: [HorizontalPresentation; 3] = [
    HorizontalPresentation::Left,
    HorizontalPresentation::Centre,
    HorizontalPresentation::Right,
];

/// 800 elements across polygons, paths, and texts on multiple layers.
fn medium_library() -> Library {
    let mut library = Library::new("bench_medium");
    let mut cell = Cell::new("cell");

    for i in 0_i32..500 {
        let offset = i * 100;
        cell.add(Polygon::new(
            vec![
                point(offset, 0),
                point(offset + 80, 0),
                point(offset + 90, 40),
                point(offset + 50, 70),
                point(offset + 10, 40),
            ],
            (i % 16) as u16,
            (i % 4) as u16,
        ));
    }

    for i in 0_i32..200 {
        let y = i * 100;
        cell.add(Path::new(
            vec![
                point(0, y),
                point(200, y + 50),
                point(400, y),
                point(600, y + 50),
                point(800, y),
            ],
            (i % 16) as u16,
            (i % 4) as u16,
            Some(PATH_TYPES[i as usize % 3]),
            Some(Unit::integer((i % 50 + 1) * 10, DB_UNITS)),
        ));
    }

    for i in 0_i32..100 {
        cell.add(Text::new(
            &format!("medium_label_{i:04}"),
            point(i * 200, -100),
            (i % 16) as u16,
            0,
            1.0 + f64::from(i % 5) * 0.5,
            f64::from(i % 4) * 0.785,
            i % 2 == 0,
            VERT_PRES[i as usize % 3],
            HORIZ_PRES[i as usize % 3],
        ));
    }

    library.add_cell(cell);
    library
}

/// 30 cells with 3 hierarchy levels exercising all GDS element types.
fn complex_library() -> Library {
    let mut library = Library::new("bench_complex");

    for c in 0_i32..10 {
        let mut cell = Cell::new(&format!("leaf_{c}"));

        for i in 0_i32..200 {
            let offset = i * 50;
            let layer = ((c * 3 + i) % 64) as u16;
            let npoints = 4 + (i as usize % 5);
            let points: Vec<Point> = (0..npoints)
                .map(|p| {
                    let angle = std::f64::consts::TAU * p as f64 / npoints as f64;
                    let r = 20 + (i % 10);
                    point(
                        offset + (f64::from(r) * angle.cos()) as i32,
                        (f64::from(r) * angle.sin()) as i32,
                    )
                })
                .collect();
            cell.add(Polygon::new(points, layer, (i % 8) as u16));
        }

        for i in 0_i32..100 {
            let y_base = c * 5000 + i * 80;
            let nsegs = 3 + (i as usize % 6);
            let points: Vec<Point> = (0..nsegs)
                .map(|s| point(s as i32 * 150, y_base + if s % 2 == 0 { 0 } else { 60 }))
                .collect();
            cell.add(Path::new(
                points,
                ((c * 2 + i) % 64) as u16,
                (i % 4) as u16,
                Some(PATH_TYPES[i as usize % 3]),
                Some(Unit::integer((i % 30 + 1) * 10, DB_UNITS)),
            ));
        }

        for i in 0_i32..50 {
            cell.add(Text::new(
                &format!("L{c}_text_{i:03}"),
                point(i * 400, c * 1000),
                ((c + i) % 64) as u16,
                (i % 4) as u16,
                1.0 + f64::from(i % 3) * 0.5,
                f64::from(i % 8) * std::f64::consts::FRAC_PI_4,
                i % 3 == 0,
                VERT_PRES[i as usize % 3],
                HORIZ_PRES[i as usize % 3],
            ));
        }

        library.add_cell(cell);
    }

    for c in 0_i32..10 {
        let mut cell = Cell::new(&format!("mid_{c}"));
        for r in 0_i32..2 {
            let leaf = (c * 2 + r) % 10;
            cell.add(
                Reference::new(format!("leaf_{leaf}")).with_grid(
                    Grid::default()
                        .with_origin(point(r * 50_000, c * 30_000))
                        .with_columns(8)
                        .with_rows(6)
                        .with_spacing_x(Some(point(12_000, 0)))
                        .with_spacing_y(Some(point(0, 10_000)))
                        .with_magnification(1.0 + f64::from(c) * 0.1)
                        .with_angle(f64::from(c) * 0.1)
                        .with_x_reflection(c % 2 == 0),
                ),
            );
        }
        library.add_cell(cell);
    }

    for c in 0_i32..10 {
        let mut cell = Cell::new(&format!("top_{c}"));

        for i in 0_i32..20 {
            cell.add(Polygon::new(
                vec![
                    point(i * 1000, 0),
                    point(i * 1000 + 500, 0),
                    point(i * 1000 + 500, 500),
                    point(i * 1000, 500),
                ],
                (i % 16) as u16,
                0,
            ));
        }

        for r in 0_i32..3 {
            let mid = (c * 3 + r) % 10;
            cell.add(
                Reference::new(format!("mid_{mid}")).with_grid(
                    Grid::default()
                        .with_origin(point(r * 200_000, 0))
                        .with_columns(4)
                        .with_rows(3)
                        .with_spacing_x(Some(point(100_000, 0)))
                        .with_spacing_y(Some(point(0, 80_000)))
                        .with_magnification(0.9 + f64::from(r) * 0.1)
                        .with_angle(f64::from(r) * 0.05)
                        .with_x_reflection(r % 2 == 1),
                ),
            );
        }

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
