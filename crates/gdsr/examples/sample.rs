use gdsr::{
    Cell, DataType, Grid, HorizontalPresentation, Layer, Library, Path, PathType, Point, Polygon,
    Reference, Text, Unit, VerticalPresentation, cell_to_svg,
};

fn main() -> Result<(), gdsr::GdsError> {
    let mut library = Library::new("sample");

    // Cell with polygons on different layers
    let mut polygons_cell = Cell::new("polygons");
    polygons_cell.add(Polygon::new(
        vec![
            Point::default_integer(0, 0),
            Point::default_integer(1000, 0),
            Point::default_integer(1000, 1000),
            Point::default_integer(0, 1000),
        ],
        Layer::new(1),
        DataType::new(0),
    ));
    polygons_cell.add(Polygon::new(
        vec![
            Point::default_integer(1500, 0),
            Point::default_integer(2500, 500),
            Point::default_integer(2000, 1000),
            Point::default_integer(1500, 1000),
        ],
        Layer::new(2),
        DataType::new(0),
    ));
    polygons_cell.add(Polygon::new(
        vec![
            Point::default_integer(3000, 0),
            Point::default_integer(4000, 0),
            Point::default_integer(3500, 1000),
        ],
        Layer::new(3),
        DataType::new(0),
    ));

    // Cell with paths
    let mut paths_cell = Cell::new("paths");
    paths_cell.add(Path::new(
        vec![
            Point::default_integer(0, 0),
            Point::default_integer(1000, 0),
            Point::default_integer(1000, 1000),
        ],
        Layer::new(4),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::default_integer(50)),
        None,
        None,
    ));
    paths_cell.add(Path::new(
        vec![
            Point::default_integer(1500, 0),
            Point::default_integer(2500, 500),
            Point::default_integer(2500, 1000),
        ],
        Layer::new(5),
        DataType::new(0),
        Some(PathType::Round),
        Some(Unit::default_integer(100)),
        None,
        None,
    ));
    paths_cell.add(Path::new(
        vec![
            Point::default_integer(3000, 0),
            Point::default_integer(3000, 1000),
        ],
        Layer::new(4),
        DataType::new(0),
        Some(PathType::Overlap),
        Some(Unit::default_integer(75)),
        None,
        None,
    ));

    // Cell with text
    let mut text_cell = Cell::new("text");
    text_cell.add(Text::new(
        "Hello",
        Point::default_integer(0, 0),
        Layer::new(6),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::Bottom,
        HorizontalPresentation::Left,
    ));
    text_cell.add(Text::new(
        "World",
        Point::default_integer(0, 500),
        Layer::new(7),
        DataType::new(0),
        2.0,
        0.0,
        false,
        VerticalPresentation::Middle,
        HorizontalPresentation::Centre,
    ));

    // Top-level cell that references the others
    let mut top = Cell::new("top");
    top.add(Reference::new("polygons").with_grid(Grid::new(
        Point::default_integer(0, 0),
        1,
        1,
        None,
        None,
        1.0,
        0.0,
        false,
    )));
    top.add(Reference::new("paths").with_grid(Grid::new(
        Point::default_integer(0, 2000),
        1,
        1,
        None,
        None,
        1.0,
        0.0,
        false,
    )));
    top.add(Reference::new("text").with_grid(Grid::new(
        Point::default_integer(0, 4000),
        1,
        1,
        None,
        None,
        1.0,
        0.0,
        false,
    )));
    // Array reference: 3x2 grid of the polygons cell
    top.add(Reference::new("polygons").with_grid(Grid::new(
        Point::default_integer(5000, 0),
        3,
        2,
        Some(Point::default_integer(5000, 0)),
        Some(Point::default_integer(0, 2000)),
        1.0,
        0.0,
        false,
    )));

    library.add_cell(polygons_cell);
    library.add_cell(paths_cell);
    library.add_cell(text_cell);
    library.add_cell(top);

    library.write_file("sample.gds", 1e-6, 1e-9)?;

    let svg = cell_to_svg(
        library.get_cell("top").expect("top cell exists"),
        &library,
        1e-9,
    );
    std::fs::write("sample.svg", &svg).expect("failed to write SVG");

    Ok(())
}
