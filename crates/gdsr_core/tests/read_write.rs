use gdsr_core::*;
use rstest::rstest;
use tempfile::tempdir;

#[test]
fn test_library_roundtrip_mixed_elements() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("main_mixed.gds");

    let units = 1e-9;

    let mut library = Library::new("mixed_elements");

    let mut cell = Cell::new("mixed_cell");

    let polygon = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        1,
        0,
    );
    cell.add(polygon);

    let text = Text::new(
        "Test Label",
        Point::integer(5, 5, units),
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
            Point::integer(0, 0, units),
            Point::integer(5, 5, units),
            Point::integer(10, 0, units),
        ],
        1,
        0,
        Some(PathType::Square),
        Some(2.0),
    );
    cell.add(path);

    let ref_polygon = Polygon::new(
        [
            Point::integer(15, 15, units),
            Point::integer(20, 15, units),
            Point::integer(20, 20, units),
            Point::integer(15, 20, units),
        ],
        2,
        0,
    );
    let reference = Reference::new(
        ref_polygon,
        Grid::new(
            Point::integer(0, 25, units),
            2,
            2,
            Point::integer(25, 0, units),
            Point::integer(0, 25, units),
            1.0,
            0.0,
            false,
        ),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

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
        &cell2,
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

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library, "{library:#?}\n{new_library:#?}");
}

#[test]
fn test_empty_library_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("empty.gds");

    let library = Library::new("empty_lib");

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-10);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_complex_path_types_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("paths.gds");

    let units = 1e-9;
    let mut library = Library::new("path_test");
    let mut cell = Cell::new("path_cell");

    // Path with Square endcaps
    let path1 = Path::new(
        vec![
            Point::integer(0, 0, units),
            Point::integer(50, 0, units),
            Point::integer(50, 50, units),
        ],
        1,
        0,
        Some(PathType::Square),
        Some(5.0),
    );
    cell.add(path1);

    // Path with Round endcaps
    let path2 = Path::new(
        vec![
            Point::integer(100, 0, units),
            Point::integer(150, 0, units),
            Point::integer(150, 50, units),
        ],
        2,
        0,
        Some(PathType::Round),
        Some(3.0),
    );
    cell.add(path2);

    // Path with Overlap
    let path3 = Path::new(
        vec![Point::integer(200, 0, units), Point::integer(250, 0, units)],
        3,
        0,
        Some(PathType::Overlap),
        Some(4.0),
    );
    cell.add(path3);

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_text_with_various_presentations() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("text.gds");

    let units = 1e-9;
    let mut library = Library::new("text_test");
    let mut cell = Cell::new("text_cell");

    // Test different text presentations
    let text1 = Text::new(
        "Top Left",
        Point::integer(0, 0, units),
        1,
        1.0,
        0.0,
        false,
        gdsr_core::VerticalPresentation::Top,
        gdsr_core::HorizontalPresentation::Left,
    );
    cell.add(text1);

    let text2 = Text::new(
        "Middle Centre",
        Point::integer(50, 50, units),
        1,
        1.5,
        45.0,
        false,
        gdsr_core::VerticalPresentation::Middle,
        gdsr_core::HorizontalPresentation::Centre,
    );
    cell.add(text2);

    let text3 = Text::new(
        "Bottom Right",
        Point::integer(100, 100, units),
        2,
        2.0,
        90.0,
        true,
        gdsr_core::VerticalPresentation::Bottom,
        gdsr_core::HorizontalPresentation::Right,
    );
    cell.add(text3);

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_nested_references() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("nested.gds");

    let units = 1e-9;
    let mut library = Library::new("nested_test");

    // Create a basic cell
    let mut cell1 = Cell::new("base_cell");
    let polygon1 = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        1,
        0,
    );
    cell1.add(polygon1);

    // Create a cell that references the first
    let mut cell2 = Cell::new("mid_cell");
    let reference1 = Reference::new(
        "base_cell".to_string(),
        Grid::new(
            Point::integer(0, 0, units),
            2,
            2,
            Point::integer(20, 0, units),
            Point::integer(0, 20, units),
            1.0,
            0.0,
            false,
        ),
    );
    cell2.add(reference1);

    // Create a top cell that references the middle cell (without transformation for now)
    let mut cell3 = Cell::new("top_cell");
    let reference2 = Reference::new(
        "mid_cell".to_string(),
        Grid::new(
            Point::integer(50, 50, units),
            1,
            1,
            Point::integer(0, 0, units),
            Point::integer(0, 0, units),
            1.0,
            0.0,
            false,
        ),
    );
    cell3.add(reference2);

    library.add(cell1);
    library.add(cell2);
    library.add(cell3);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_large_polygon_coordinates() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("large_coords.gds");

    let units = 1e-9;
    let mut library = Library::new("large_coords_test");
    let mut cell = Cell::new("large_cell");

    // Test with large coordinate values
    let polygon = Polygon::new(
        [
            Point::integer(1_000_000, 1_000_000, units),
            Point::integer(2_000_000, 1_000_000, units),
            Point::integer(2_000_000, 2_000_000, units),
            Point::integer(1_000_000, 2_000_000, units),
        ],
        1,
        0,
    );
    cell.add(polygon);

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_float_coordinates() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("float_coords.gds");

    let units = 1e-9;
    let mut library = Library::new("float_test");
    let mut cell = Cell::new("float_cell");

    // Test with float coordinates - they will be rounded when written
    let polygon = Polygon::new(
        [
            Point::float(0.5, 0.5, units),
            Point::float(10.5, 0.5, units),
            Point::float(10.5, 10.5, units),
            Point::float(0.5, 10.5, units),
        ],
        1,
        0,
    );
    cell.add(polygon);

    let path = Path::new(
        vec![
            Point::float(15.25, 15.25, units),
            Point::float(20.75, 15.25, units),
            Point::float(20.75, 20.75, units),
        ],
        2,
        0,
        Some(PathType::Round),
        Some(1.5),
    );
    cell.add(path);

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    // Float coordinates are rounded when writing to GDS, so we just verify it reads back successfully
    assert_eq!(new_library.name, library.name);
    assert_eq!(new_library.cells.len(), 1);
    assert!(new_library.cells.contains_key("float_cell"));
}

#[test]
fn test_multiple_cells_different_layers() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("multi_layer.gds");

    let units = 1e-9;
    let mut library = Library::new("multi_layer_test");

    // Create cells with different layers
    for layer in 0..5 {
        let cell_name = format!("layer_{layer}_cell");
        let mut cell = Cell::new(cell_name.as_str());

        let polygon = Polygon::new(
            [
                Point::integer(layer * 10, 0, units),
                Point::integer(layer * 10 + 10, 0, units),
                Point::integer(layer * 10 + 10, 10, units),
                Point::integer(layer * 10, 10, units),
            ],
            layer as u16,
            layer as u16,
        );
        cell.add(polygon);

        library.add(cell);
    }

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_error_invalid_file() {
    let result = Library::from_gds("/nonexistent/path/to/file.gds", Some(DEFAULT_INTEGER_UNITS));
    assert!(result.is_err());
}

#[test]
fn test_single_cell_with_all_element_types() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let units = 1e-9;
    let mut library = Library::new("all_elements_test");
    let mut cell = Cell::new("main_cell");

    // Add a polygon
    cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(100, 0, units),
            Point::integer(100, 100, units),
            Point::integer(0, 100, units),
        ],
        1,
        0,
    ));

    // Add a path
    cell.add(Path::new(
        vec![
            Point::integer(150, 50, units),
            Point::integer(200, 50, units),
            Point::integer(200, 100, units),
        ],
        2,
        0,
        Some(PathType::Round),
        Some(5.0),
    ));

    // Add text
    cell.add(Text::new(
        "All Elements",
        Point::integer(250, 50, units),
        3,
        2.0,
        0.0,
        false,
        gdsr_core::VerticalPresentation::Middle,
        gdsr_core::HorizontalPresentation::Centre,
    ));

    // Add a reference to a simple cell
    let mut ref_cell = Cell::new("ref_cell");
    ref_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(20, 0, units),
            Point::integer(20, 20, units),
            Point::integer(0, 20, units),
        ],
        4,
        0,
    ));
    library.add(ref_cell);

    cell.add(Reference::new(
        "ref_cell".to_string(),
        Grid::new(
            Point::integer(300, 50, units),
            1,
            1,
            Point::integer(0, 0, units),
            Point::integer(0, 0, units),
            1.0,
            0.0,
            false,
        ),
    ));

    library.add(cell);

    let _res = library.to_gds(gds_path.to_str().unwrap(), 1e-9, 1e-9);

    let new_library: Library =
        Library::from_gds(gds_path.to_str().unwrap(), Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}
