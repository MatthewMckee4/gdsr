use gdsr_core::*;
use rstest::rstest;
use tempfile::tempdir;

#[test]
fn test_library_roundtrip_mixed_elements() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("main_mixed.gds");

    let mut library = Library::new("mixed_elements");

    let mut cell = Cell::new("mixed_cell");

    let polygon = Polygon::new([(0, 0), (10, 0), (10, 10), (0, 10)], 1, 0);
    cell.add(polygon);

    let text = Text::new(
        "Test Label".to_string(),
        Point::new(5, 5),
        1,
        1.0,
        0.0,
        false,
        gdsr_core::VerticalPresentation::default(),
        gdsr_core::HorizontalPresentation::default(),
    );
    cell.add(text);

    let path = Path::new(
        vec![Point::new(0, 0), Point::new(5, 5), Point::new(10, 0)],
        1,
        0,
        Some(PathType::Square),
        Some(2.0),
    );
    cell.add(path);

    let ref_polygon = Polygon::new([(15, 15), (20, 15), (20, 20), (15, 20)], 2, 0);
    let reference = Reference::new(
        ref_polygon,
        Grid::new((0, 25), 2, 2, (25, 0), (0, 25), 1.0, 0.0, false),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-10);

    let new_library: Library<DatabaseIntegerUnit> =
        Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library);
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
        Grid::new((0, 0), 3, 3, (150, 0), (0, 150), 0.0, 0.0, false),
    );

    cell.add(reference2);

    library.add(cell);
    library.add(cell2);

    let _res = library.to_gds(gds_path.to_str().unwrap(), user_units, database_units);

    let new_library: Library<DatabaseIntegerUnit> =
        Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_empty_library_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("empty.gds");

    let library = Library::new("empty_lib");

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-10);

    let new_library: Library<DatabaseIntegerUnit> =
        Library::from_gds(gds_path.to_str().unwrap()).unwrap();

    assert_eq!(library, new_library);
}
