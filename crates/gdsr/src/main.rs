use gdsr::{Cell, Grid, Library, Point, Polygon, Reference};

fn main() {
    let units = 1e-9;

    let mut library = Library::new("main");

    let mut cell = Cell::new("main_cell");

    let polygon = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(1, 0, units),
            Point::integer(1, 1, units),
            Point::integer(0, 1, units),
        ],
        1,
        0,
    );

    let reference = Reference::new(
        polygon,
        Grid::new(
            Point::integer(0, 0, units),
            5,
            5,
            Point::integer(2, 0, units),
            Point::integer(0, 2, units),
            1.0,
            0.0,
            false,
        ),
    );

    cell.add(reference);

    library.add(cell);

    library.write_file("main.gds", 1e-6, 1e-9).unwrap();
}
