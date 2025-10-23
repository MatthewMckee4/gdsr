use gdsr_core::*;

fn main() {
    let mut library = Library::new("mixed_elements");

    let mut cell = Cell::new("precision_cell");

    let polygon = Polygon::new([(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);

    let reference = Reference::new(
        polygon,
        Grid::new((0, 0), 3, 3, (150, 0), (0, 150), 1.5, 45.0, true),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    let mut cell2 = Cell::new("precision_cell2");

    let polygon2 = Polygon::new([(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);

    cell2.add(polygon2);

    let reference2 = Reference::new(
        cell2.name(),
        Grid::new((0, 0), 3, 3, (150, 0), (0, 150), 1.0, 0.0, false),
    );

    cell.add(reference2);

    library.add(cell);
    library.add(cell2);

    let gds_path = "test.gds";

    let _res = library.to_gds(gds_path, 1e-6, 1e-9);

    let new_library: Library = Library::from_gds(gds_path).unwrap();

    println!("Are equal {}", library == new_library);
}
