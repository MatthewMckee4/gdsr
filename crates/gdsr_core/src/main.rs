use gdsr_core::*;

fn main() {
    let mut library = Library::new("mixed_elements");

    let units = 1e-9;

    let mut cell = Cell::new("precision_cell");

    let polygon = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(100, 0, units),
            Point::integer(100, 100, units),
            Point::integer(0, 100, units),
        ],
        1,
        0,
    );

    let reference = Reference::new(
        polygon,
        Grid::new(
            Point::integer(0, 0, units),
            3,
            3,
            Point::integer(150, 0, units),
            Point::integer(0, 150, units),
            1.5,
            45.0,
            true,
        ),
    );

    let elements = reference.flatten(None, &library);

    // for element in elements {
    //     cell.add(element);
    // }

    let mut cell2 = Cell::new("precision_cell2");

    let polygon2 = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(100, 0, units),
            Point::integer(100, 100, units),
            Point::integer(0, 100, units),
        ],
        1,
        0,
    );

    cell2.add(polygon2);

    let reference2 = Reference::new(
        cell2.name(),
        Grid::new(
            Point::integer(0, 0, units),
            3,
            3,
            Point::integer(150, 0, units),
            Point::integer(0, 150, units),
            1.0,
            0.0,
            false,
        ),
    );

    cell.add(reference2);

    library.add(cell2);

    library.add(cell);

    let gds_path = "test.gds";

    let _res = library.to_gds(gds_path, 1e-3, 1e-6);

    let new_library: Library = Library::from_gds(gds_path).unwrap();

    println!("Library: {:#?}", library);
    println!("new_library: {:#?}", new_library);
    println!("Are equal {}", library == new_library);
}
