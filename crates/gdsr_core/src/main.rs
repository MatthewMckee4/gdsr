use gdsr_core::{Cell, DatabaseIntegerUnit, Library, elements::Polygon};

fn main() {
    let mut library: Library<DatabaseIntegerUnit> = Library::new("main");

    let mut cell: Cell<DatabaseIntegerUnit> = Cell::new("main_cell");

    let polygon: Polygon<DatabaseIntegerUnit> = Polygon::new(
        [(0, 0).into(), (1, 0).into(), (1, 1).into(), (0, 1).into()].to_vec(),
        1,
        0,
    );

    cell.add_polygon(polygon);

    library.add(cell);

    let _res = library.to_gds("main.gds", 1e10, 1e4);
}
