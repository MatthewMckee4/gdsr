use gdsr_core::{
    Cell, DatabaseIntegerUnit, Grid, Library,
    elements::{Polygon, Reference},
};

fn main() {
    let mut library = Library::new("main");

    let mut cell = Cell::new("main_cell");

    let polygon = Polygon::new(
        [(0, 0).into(), (1, 0).into(), (1, 1).into(), (0, 1).into()].to_vec(),
        1,
        0,
    );

    let reference = Reference::new(
        polygon.into(),
        Grid::new((0, 0), 5, 5, (2, 0), (0, 2), 2.0, 0.0, false),
    );

    cell.add(reference);

    library.add(cell);

    let _res = library.to_gds("main.gds", 1e-9, 1e-9);

    let new_library: Library<DatabaseIntegerUnit> = Library::from_gds("main.gds").unwrap();

    println!("{:?}", new_library);
}
