use gdsr_core::*;
use rstest::rstest;
use tempfile::tempdir;

#[test]
fn test_library_roundtrip_mixed_elements() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("main_mixed.gds");

    let mut library = Library::new("mixed_elements");

    let mut cell = Cell::new("mixed_cell");

    let polygon = Polygon::new(
        [
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ],
        1,
        0,
    );
    cell.add(polygon);

    let text = Text::new(
        "Test Label".to_string(),
        Point::integer(5, 5, 1e-9),
        1,
        1.0,
        0.0,
        false,
        gdsr_core::VerticalPresentation::default(),
        gdsr_core::HorizontalPresentation::default(),
    );
    cell.add(text);

    let path = Path::new(
        vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(5, 5, 1e-9),
            Point::integer(10, 0, 1e-9),
        ],
        1,
        0,
        Some(PathType::Square),
        Some(2.0),
    );
    cell.add(path);

    let ref_polygon = Polygon::new(
        [
            Point::integer(15, 15, 1e-9),
            Point::integer(20, 15, 1e-9),
            Point::integer(20, 20, 1e-9),
            Point::integer(15, 20, 1e-9),
        ],
        2,
        0,
    );
    let reference = Reference::new(
        ref_polygon,
        Grid::new((0, 25), 2, 2, (25, 0), (0, 25), 1.0, 0.0, false),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library = Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library, "{library:#?}\n{new_library:#?}");
}

#[rstest]
#[case(1e-9, 1e-9)]
#[case(1e-9, 1e-10)]
#[case(1e-6, 1e-9)]
#[case(1e-3, 1e-6)]
fn test_library_roundtrip_different_precision(
    #[case] user_units: f64,
    #[case] database_units: f64,
) {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("precision_test.gds");

    let mut library = Library::new("precision_test");

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

    for element in elements {
        cell.add(element);
    }

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

    let _res = library.to_gds(gds_path.to_str().unwrap(), user_units, database_units);

    let new_library: Library = Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library, "{library:#?}\n{new_library:#?}");
}

#[test]
fn test_empty_library_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("empty.gds");

    let library = Library::new("empty_lib");

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-10);

    let new_library: Library = Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library);
}
