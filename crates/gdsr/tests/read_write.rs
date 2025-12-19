use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use gdsr::*;
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
        0,
        1.0,
        0.0,
        false,
        gdsr::VerticalPresentation::default(),
        gdsr::HorizontalPresentation::default(),
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
        Some(Unit::float(2.0, units)),
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
            Some(Point::integer(25, 0, units)),
            Some(Point::integer(0, 25, units)),
            1.0,
            0.0,
            false,
        ),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library, "{library:#?}\n{new_library:#?}");
}

#[rstest]
#[case(1e-9, 1e-10)]
#[case(1e-6, 1e-10)]
#[case(1e-7, 1e-9)]
#[case(1e-8, 1e-9)]
#[case(1e-9, 1e-9)]
fn test_library_roundtrip_different_units(#[case] user_units: f64, #[case] database_units: f64) {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("main.gds");

    let mut library = Library::new("main");

    let units = 1e-9;

    let mut cell = Cell::new("1");

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
            Some(Point::integer(150, 0, units)),
            Some(Point::integer(0, 150, units)),
            1.5,
            45.0,
            true,
        ),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    let mut cell2 = Cell::new("2");

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
            Some(Point::integer(150, 0, units)),
            Some(Point::integer(0, 150, units)),
            1.0,
            0.0,
            false,
        ),
    );

    cell.add(reference2);

    library.add_cell(cell2);

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, user_units, database_units);

    let new_library = Library::read_file(&gds_path, Some(units)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_empty_library_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("empty.gds");

    let library = Library::new("empty");

    let _res = library.write_file(&gds_path, 1e-9, 1e-10);

    let new_library = Library::read_file(&gds_path, None).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_complex_path_types_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("paths.gds");

    let units = 1e-9;
    let mut library = Library::new("path_test");
    let mut cell = Cell::new("path_cell");

    let path1 = Path::new(
        vec![
            Point::integer(0, 0, units),
            Point::integer(50, 0, units),
            Point::integer(50, 50, units),
        ],
        1,
        0,
        Some(PathType::Square),
        Some(Unit::float(5.0, units)),
    );
    cell.add(path1);

    let path2 = Path::new(
        vec![
            Point::integer(100, 0, units),
            Point::integer(150, 0, units),
            Point::integer(150, 50, units),
        ],
        2,
        0,
        Some(PathType::Round),
        Some(Unit::float(3.0, units)),
    );
    cell.add(path2);

    let path3 = Path::new(
        vec![Point::integer(200, 0, units), Point::integer(250, 0, units)],
        3,
        0,
        Some(PathType::Overlap),
        Some(Unit::float(4.0, units)),
    );

    cell.add(path3);

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, None).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_text_with_various_presentations() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("text.gds");

    let units = 1e-9;
    let mut library = Library::new("text_test");
    let mut cell = Cell::new("text_cell");

    let text1 = Text::new(
        "Top Left",
        Point::integer(0, 0, units),
        1,
        0,
        1.0,
        0.0,
        false,
        gdsr::VerticalPresentation::Top,
        gdsr::HorizontalPresentation::Left,
    );
    cell.add(text1);

    let text2 = Text::new(
        "Middle Centre",
        Point::integer(50, 50, units),
        1,
        0,
        1.5,
        45.0,
        false,
        gdsr::VerticalPresentation::Middle,
        gdsr::HorizontalPresentation::Centre,
    );
    cell.add(text2);

    let text3 = Text::new(
        "Bottom Right",
        Point::integer(100, 100, units),
        2,
        0,
        2.0,
        90.0,
        true,
        gdsr::VerticalPresentation::Bottom,
        gdsr::HorizontalPresentation::Right,
    );
    cell.add(text3);

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, None).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_nested_references() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("nested.gds");

    let units = 1e-9;
    let mut library = Library::new("nested_test");

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

    let mut cell2 = Cell::new("mid_cell");
    let reference1 = Reference::new(
        "base_cell".to_string(),
        Grid::new(
            Point::integer(0, 0, units),
            2,
            2,
            Some(Point::integer(20, 0, units)),
            Some(Point::integer(0, 20, units)),
            1.0,
            0.0,
            false,
        ),
    );
    cell2.add(reference1);

    let mut cell3 = Cell::new("top_cell");
    let reference2 = Reference::new(
        "mid_cell".to_string(),
        Grid::new(
            Point::integer(50, 50, units),
            1,
            1,
            Some(Point::integer(0, 0, units)),
            Some(Point::integer(0, 0, units)),
            1.0,
            0.0,
            false,
        ),
    );
    cell3.add(reference2);

    library.add_cell(cell1);
    library.add_cell(cell2);
    library.add_cell(cell3);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_large_polygon_coordinates() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("large_coords.gds");

    let units = 1e-9;
    let mut library = Library::new("large_coords_test");
    let mut cell = Cell::new("large_cell");

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

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_float_coordinates() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("float_coords.gds");

    let units = 1e-9;
    let mut library = Library::new("float_test");
    let mut cell = Cell::new("float_cell");

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
        Some(Unit::float(1.5, units)),
    );

    cell.add(path);

    library.add_cell(cell.clone());

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(new_library.name(), library.name());
    assert_eq!(new_library.cells().len(), 1);
    assert!(new_library.contains_cell(&cell));
}

#[test]
fn test_multiple_cells_different_layers() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("multi_layer.gds");

    let units = 1e-9;
    let mut library = Library::new("multi_layer_test");

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
            0,
        );
        cell.add(polygon);

        library.add_cell(cell);
    }

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

#[test]
fn test_error_invalid_file() {
    let result = Library::read_file("/nonexistent/path/to/file.gds", None);
    assert!(result.is_err());
}

#[test]
fn test_single_cell_with_all_element_types() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let units = 1e-9;
    let mut library = Library::new("all_elements_test");
    let mut cell = Cell::new("main_cell");

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

    cell.add(Path::new(
        vec![
            Point::integer(150, 50, units),
            Point::integer(200, 50, units),
            Point::integer(200, 100, units),
        ],
        2,
        0,
        Some(PathType::Round),
        Some(Unit::float(5.0, units)),
    ));

    cell.add(Text::new(
        "All Elements",
        Point::integer(250, 50, units),
        3,
        0,
        2.0,
        0.0,
        false,
        gdsr::VerticalPresentation::Middle,
        gdsr::HorizontalPresentation::Centre,
    ));

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
    library.add_cell(ref_cell);

    cell.add(Reference::new(
        "ref_cell".to_string(),
        Grid::new(
            Point::integer(300, 50, units),
            1,
            1,
            Some(Point::integer(0, 0, units)),
            Some(Point::integer(0, 0, units)),
            1.0,
            0.0,
            false,
        ),
    ));

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    assert_eq!(library, new_library);
}

fn get_elements(units: f64) -> Vec<Element> {
    vec![
        Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            4,
            0,
        )
        .into(),
        Text::new(
            "All Elements",
            Point::integer(250, 50, units),
            3,
            0,
            2.0,
            0.0,
            false,
            gdsr::VerticalPresentation::Middle,
            gdsr::HorizontalPresentation::Centre,
        )
        .into(),
        Path::new(
            vec![
                Point::integer(150, 50, units),
                Point::integer(200, 50, units),
                Point::integer(200, 100, units),
            ],
            2,
            0,
            Some(PathType::Round),
            Some(Unit::float(5.0, units)),
        )
        .into(),
        Path::new(
            vec![
                Point::integer(150, 50, units),
                Point::integer(200, 50, units),
                Point::integer(200, 100, units),
            ],
            2,
            0,
            None,
            None,
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(10, 0, units),
                    Point::integer(10, 10, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                2,
                2,
                Some(Point::integer(0, 10, units)),
                Some(Point::integer(10, 0, units)),
                1.0,
                FRAC_PI_4,
                false,
            ),
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(10, 0, units),
                    Point::integer(10, 10, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                1,
                1,
                None,
                None,
                2.0,
                FRAC_PI_4,
                false,
            ),
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(10, 0, units),
                    Point::integer(10, 10, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                1,
                1,
                None,
                None,
                2.0,
                -FRAC_PI_4,
                false,
            ),
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(20, 0, units),
                    Point::integer(20, 20, units),
                    Point::integer(0, 20, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                1,
                1,
                None,
                None,
                1.0,
                0.0,
                false,
            ),
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(20, 0, units),
                    Point::integer(20, 20, units),
                    Point::integer(0, 20, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                1,
                2,
                None,
                Some(Point::integer(10, 10, units)),
                1.0,
                0.0,
                false,
            ),
        )
        .into(),
        Reference::new(
            Polygon::new(
                [
                    Point::integer(0, 0, units),
                    Point::integer(20, 0, units),
                    Point::integer(20, 20, units),
                    Point::integer(0, 20, units),
                ],
                4,
                0,
            ),
            Grid::new(
                Point::integer(300, 50, units),
                2,
                1,
                Some(Point::integer(10, 10, units)),
                None,
                1.0,
                0.0,
                false,
            ),
        )
        .into(),
    ]
}

#[test]
fn test_element_reference() {
    let units = DEFAULT_INTEGER_UNITS;

    for element in get_elements(units) {
        let temp_dir = tempdir().unwrap();
        let gds_path = temp_dir.path().join("all_elements.gds");

        let mut library = Library::new("reference_test");

        let mut cell = Cell::new("cell");

        let element = element.move_to(Point::integer(100, 100, units));

        let reference = Reference::new(
            element.clone(),
            Grid::new(
                Point::integer(10, 0, units),
                1,
                1,
                Some(Point::integer(10, 10, units)),
                Some(Point::integer(10, 10, units)),
                1.0,
                0.0,
                false,
            ),
        );

        cell.add(reference.clone());

        library.add_cell(cell);

        let _res = library.write_file(&gds_path, 1e-9, 1e-9);

        let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

        let mut expected_library = Library::new("reference_test");
        let mut cell = Cell::new("cell");

        for element in reference.flatten(None, &expected_library) {
            cell.add(element);
        }

        expected_library.add_cell(cell);

        assert_eq!(expected_library, new_library);
    }
}

#[test]
fn test_cell_reference() {
    let units = DEFAULT_INTEGER_UNITS;

    for element in get_elements(units) {
        let temp_dir = tempdir().unwrap();
        let gds_path = temp_dir.path().join("all_elements.gds");

        let mut library = Library::new("reference_test");

        let mut cell = Cell::new("cell");

        let element = element.move_to(Point::integer(100, 100, units));

        let mut ref_cell = Cell::new("ref_cell");

        ref_cell.add(element.clone());

        let reference = Reference::new(
            "ref_cell",
            Grid::new(
                Point::integer(00, 0, units),
                2,
                2,
                Some(Point::integer(10, 10, units)),
                Some(Point::integer(10, 10, units)),
                2.0,
                FRAC_PI_2,
                true,
            ),
        );

        cell.add(reference.clone());

        library.add_cell(cell);

        let _res = library.write_file(&gds_path, 1e-9, 1e-9);

        let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

        assert_eq!(library, new_library);
    }
}

#[test]
fn test_invalid_path() {
    let units = DEFAULT_INTEGER_UNITS;

    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let mut library = Library::new("reference_test");

    let mut cell = Cell::new("cell");

    let path = Path::new(
        vec![Point::integer(150, 50, units)],
        2,
        0,
        Some(PathType::Round),
        Some(Unit::float(5.0, units)),
    );

    cell.add(path);

    library.add_cell(cell);

    let res = library.write_file(&gds_path, 1e-9, 1e-9);

    assert!(res.is_err());
}

#[test]
fn test_invalid_polygon() {
    let units = DEFAULT_INTEGER_UNITS;

    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let mut library = Library::new("reference_test");

    let mut cell = Cell::new("cell");

    let mut polygon_points = Vec::new();
    for i in 0..8192 {
        polygon_points.push(Point::integer(i, 0, units));
    }

    let polygon = Polygon::new(polygon_points, 1, 0);

    cell.add(polygon);

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    let mut expected_library = Library::new("reference_test");

    let cell = Cell::new("cell");

    expected_library.add_cell(cell);

    assert_eq!(expected_library, new_library);
}

#[test]
fn test_invalid_no_cell() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let mut library = Library::new("reference_test");

    let mut cell = Cell::new("cell");

    let elements = Reference::new("random_cell", Grid::default()).flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add_cell(cell);

    let _res = library.write_file(&gds_path, 1e-9, 1e-9);

    let new_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();

    let mut expected_library = Library::new("reference_test");

    let cell = Cell::new("cell");

    expected_library.add_cell(cell);

    assert_eq!(expected_library, new_library);
}
