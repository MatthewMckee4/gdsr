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

fn build_library() -> Library {
    let mut library = Library::new("bench_complex");

    for c in 0_i32..20 {
        let mut cell = Cell::new(&format!("leaf_{c}"));

        for i in 0_i32..500 {
            let offset = i * 50;
            let layer = ((c * 3 + i) % 64) as u16;
            let npoints = 4 + (i as usize % 5);
            let points: Vec<Point> = (0..npoints)
                .map(|p| {
                    let angle = std::f64::consts::TAU * p as f64 / npoints as f64;
                    let r = 20 + (i % 30);
                    point(
                        offset + (f64::from(r) * angle.cos()) as i32,
                        (f64::from(r) * angle.sin()) as i32,
                    )
                })
                .collect();
            cell.add(Polygon::new(points, layer, (i % 8) as u16));
        }

        for i in 0_i32..250 {
            let y_base = c * 5000 + i * 80;
            let nsegs = 3 + (i as usize % 8);
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

        for i in 0_i32..100 {
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

    for c in 0_i32..20 {
        let mut cell = Cell::new(&format!("mid_{c}"));
        for r in 0_i32..3 {
            let leaf = (c * 3 + r) % 20;
            cell.add(
                Reference::new(format!("leaf_{leaf}")).with_grid(
                    Grid::default()
                        .with_origin(point(r * 80_000, c * 40_000))
                        .with_columns(10)
                        .with_rows(8)
                        .with_spacing_x(Some(point(15_000, 0)))
                        .with_spacing_y(Some(point(0, 12_000)))
                        .with_magnification(1.0 + f64::from(c % 5) * 0.1)
                        .with_angle(f64::from(c % 8) * 0.05)
                        .with_x_reflection(c % 3 == 0),
                ),
            );
        }

        for i in 0_i32..50 {
            cell.add(Polygon::new(
                vec![
                    point(i * 2000, 0),
                    point(i * 2000 + 1000, 0),
                    point(i * 2000 + 1000, 1000),
                    point(i * 2000, 1000),
                ],
                (i % 16) as u16,
                0,
            ));
        }

        library.add_cell(cell);
    }

    for c in 0_i32..10 {
        let mut cell = Cell::new(&format!("top_{c}"));

        for i in 0_i32..100 {
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
            let mid = (c * 3 + r) % 20;
            cell.add(
                Reference::new(format!("mid_{mid}")).with_grid(
                    Grid::default()
                        .with_origin(point(r * 300_000, 0))
                        .with_columns(5)
                        .with_rows(4)
                        .with_spacing_x(Some(point(150_000, 0)))
                        .with_spacing_y(Some(point(0, 120_000)))
                        .with_magnification(0.9 + f64::from(r) * 0.1)
                        .with_angle(f64::from(r) * 0.03)
                        .with_x_reflection(r % 2 == 1),
                ),
            );
        }

        library.add_cell(cell);
    }

    library
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: benchmark <write|read>");
        std::process::exit(1);
    }

    let tmp_dir = std::env::temp_dir();
    let path = tmp_dir.join("gdsr_bench.gds");

    let library = build_library();

    match args[1].as_str() {
        "write" => {
            library.write_file(&path, DB_UNITS, USER_UNITS).unwrap();
        }
        "read" => {
            if !path.exists() {
                library.write_file(&path, DB_UNITS, USER_UNITS).unwrap();
            }
            Library::read_file(&path, Some(DB_UNITS)).unwrap();
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}
