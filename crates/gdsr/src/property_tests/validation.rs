use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use tempfile::tempdir;

use crate::*;

fn assert_roundtrip(library: &Library) {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("test.gds");
    library.write_file(&gds_path, 1e-9, 1e-9).unwrap();
    let read_library = Library::read_file(&gds_path, Some(DEFAULT_INTEGER_UNITS)).unwrap();
    assert_eq!(*library, read_library, "{library:#?}\n{read_library:#?}");
}

fn assert_write_validation_error(library: &Library) {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("validation.gds");
    let err = library.write_file(&gds_path, 1e-9, 1e-9).unwrap_err();
    assert!(
        matches!(err, GdsError::ValidationError { .. }),
        "expected ValidationError, got: {err}"
    );
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
            Layer::new(4),
            DataType::new(0),
        )
        .into(),
        Text::new(
            "All Elements",
            Point::integer(250, 50, units),
            Layer::new(3),
            DataType::new(0),
            2.0,
            0.0,
            false,
            VerticalPresentation::Middle,
            HorizontalPresentation::Centre,
        )
        .into(),
        Path::new(
            vec![
                Point::integer(150, 50, units),
                Point::integer(200, 50, units),
                Point::integer(200, 100, units),
            ],
            Layer::new(2),
            DataType::new(0),
            Some(PathType::Round),
            Some(Unit::float(5.0, units)),
            None,
            None,
        )
        .into(),
        Path::new(
            vec![
                Point::integer(150, 50, units),
                Point::integer(200, 50, units),
                Point::integer(200, 100, units),
            ],
            Layer::new(2),
            DataType::new(0),
            None,
            None,
            None,
            None,
        )
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            2,
            2,
            Some(Point::integer(0, 10, units)),
            Some(Point::integer(10, 0, units)),
            1.0,
            FRAC_PI_4,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            1,
            1,
            None,
            None,
            2.0,
            FRAC_PI_4,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            1,
            1,
            None,
            None,
            2.0,
            -FRAC_PI_4,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            1,
            1,
            None,
            None,
            1.0,
            0.0,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            1,
            2,
            None,
            Some(Point::integer(10, 10, units)),
            1.0,
            0.0,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            2,
            1,
            Some(Point::integer(10, 10, units)),
            None,
            1.0,
            0.0,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            1,
            2,
            None,
            Some(Point::integer(10, 10, units)),
            1.0,
            0.0,
            false,
        ))
        .into(),
        Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(20, 0, units),
                Point::integer(20, 20, units),
                Point::integer(0, 20, units),
            ],
            Layer::new(4),
            DataType::new(0),
        ))
        .with_grid(Grid::new(
            Point::integer(300, 50, units),
            2,
            1,
            Some(Point::integer(10, 10, units)),
            None,
            1.0,
            0.0,
            false,
        ))
        .into(),
    ]
}

#[test]
fn test_library_roundtrip_mixed_elements() {
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
        Layer::new(1),
        DataType::new(0),
    );
    cell.add(polygon);

    let text = Text::new(
        "Test Label",
        Point::integer(5, 5, units),
        Layer::new(1),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::default(),
        HorizontalPresentation::default(),
    );
    cell.add(text);

    let path = Path::new(
        vec![
            Point::integer(0, 0, units),
            Point::integer(5, 5, units),
            Point::integer(10, 0, units),
        ],
        Layer::new(1),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::float(2.0, units)),
        None,
        None,
    );
    cell.add(path);

    let ref_polygon = Polygon::new(
        [
            Point::integer(15, 15, units),
            Point::integer(20, 15, units),
            Point::integer(20, 20, units),
            Point::integer(15, 20, units),
        ],
        Layer::new(2),
        DataType::new(0),
    );

    let reference = Reference::new(ref_polygon.clone()).with_grid(
        Grid::default()
            .with_origin(Point::integer(0, 25, units))
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(25, 0, units)))
            .with_spacing_y(Some(Point::integer(0, 25, units))),
    );

    let elements = reference.flatten(None, &library);

    for element in elements {
        cell.add(element);
    }

    library.add_cell(cell);

    assert_roundtrip(&library);
}

#[test]
fn test_library_roundtrip_different_units() {
    let cases = [
        (1e-9, 1e-10),
        (1e-6, 1e-10),
        (1e-7, 1e-9),
        (1e-8, 1e-9),
        (1e-9, 1e-9),
    ];

    for (user_units, database_units) in cases {
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
            Layer::new(1),
            DataType::new(0),
        );

        let reference = Reference::new(polygon.clone()).with_grid(Grid::new(
            Point::integer(0, 0, units),
            3,
            3,
            Some(Point::integer(150, 0, units)),
            Some(Point::integer(0, 150, units)),
            1.5,
            45.0,
            true,
        ));

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
            Layer::new(1),
            DataType::new(0),
        );

        cell2.add(polygon2);

        let reference2 = Reference::new(&cell2).with_grid(
            Grid::default()
                .with_columns(3)
                .with_rows(3)
                .with_spacing_x(Some(Point::integer(150, 0, units)))
                .with_spacing_y(Some(Point::integer(0, 150, units))),
        );

        cell.add(reference2);

        library.add_cell(cell2);

        library.add_cell(cell);

        let temp_dir = tempdir().unwrap();
        let gds_path = temp_dir.path().join("main.gds");
        library
            .write_file(&gds_path, user_units, database_units)
            .unwrap();
        let new_library = Library::read_file(&gds_path, Some(units)).unwrap();
        assert_eq!(library, new_library);
    }
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
    let units = 1e-9;
    let mut library = Library::new("path_test");
    let mut cell = Cell::new("path_cell");

    cell.add(Path::new(
        vec![
            Point::integer(0, 0, units),
            Point::integer(50, 0, units),
            Point::integer(50, 50, units),
        ],
        Layer::new(1),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::float(5.0, units)),
        None,
        None,
    ));

    cell.add(Path::new(
        vec![
            Point::integer(100, 0, units),
            Point::integer(150, 0, units),
            Point::integer(150, 50, units),
        ],
        Layer::new(2),
        DataType::new(0),
        Some(PathType::Round),
        Some(Unit::float(3.0, units)),
        None,
        None,
    ));

    cell.add(Path::new(
        vec![Point::integer(200, 0, units), Point::integer(250, 0, units)],
        Layer::new(3),
        DataType::new(0),
        Some(PathType::Overlap),
        Some(Unit::float(4.0, units)),
        None,
        None,
    ));

    library.add_cell(cell);

    assert_roundtrip(&library);
}

#[test]
fn test_text_with_various_presentations() {
    let units = 1e-9;
    let mut library = Library::new("text_test");
    let mut cell = Cell::new("text_cell");

    cell.add(Text::new(
        "Top Left",
        Point::integer(0, 0, units),
        Layer::new(1),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::Top,
        HorizontalPresentation::Left,
    ));

    cell.add(Text::new(
        "Middle Centre",
        Point::integer(50, 50, units),
        Layer::new(1),
        DataType::new(0),
        1.5,
        45.0,
        false,
        VerticalPresentation::Middle,
        HorizontalPresentation::Centre,
    ));

    cell.add(Text::new(
        "Bottom Right",
        Point::integer(100, 100, units),
        Layer::new(2),
        DataType::new(0),
        2.0,
        90.0,
        true,
        VerticalPresentation::Bottom,
        HorizontalPresentation::Right,
    ));

    library.add_cell(cell);

    assert_roundtrip(&library);
}

#[test]
fn test_nested_references() {
    let units = 1e-9;
    let mut library = Library::new("nested_test");

    let mut cell1 = Cell::new("base_cell");
    cell1.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        Layer::new(1),
        DataType::new(0),
    ));

    let mut cell2 = Cell::new("mid_cell");
    cell2.add(
        Reference::new("base_cell".to_string()).with_grid(
            Grid::default()
                .with_columns(2)
                .with_rows(2)
                .with_spacing_x(Some(Point::integer(20, 0, units)))
                .with_spacing_y(Some(Point::integer(0, 20, units))),
        ),
    );

    let mut cell3 = Cell::new("top_cell");
    cell3.add(
        Reference::new("mid_cell".to_string())
            .with_grid(Grid::default().with_origin(Point::integer(50, 50, units))),
    );

    library.add_cell(cell1);
    library.add_cell(cell2);
    library.add_cell(cell3);

    assert_roundtrip(&library);
}

#[test]
fn test_large_polygon_coordinates() {
    let units = 1e-9;
    let mut library = Library::new("large_coords_test");
    let mut cell = Cell::new("large_cell");

    cell.add(Polygon::new(
        [
            Point::integer(1_000_000, 1_000_000, units),
            Point::integer(2_000_000, 1_000_000, units),
            Point::integer(2_000_000, 2_000_000, units),
            Point::integer(1_000_000, 2_000_000, units),
        ],
        Layer::new(1),
        DataType::new(0),
    ));

    library.add_cell(cell);

    assert_roundtrip(&library);
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
        Layer::new(1),
        DataType::new(0),
    );
    cell.add(polygon);

    let path = Path::new(
        vec![
            Point::float(15.25, 15.25, units),
            Point::float(20.75, 15.25, units),
            Point::float(20.75, 20.75, units),
        ],
        Layer::new(2),
        DataType::new(0),
        Some(PathType::Round),
        Some(Unit::float(1.5, units)),
        None,
        None,
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
    let units = 1e-9;
    let mut library = Library::new("multi_layer_test");

    for layer in 0..5 {
        let cell_name = format!("layer_{layer}_cell");
        let mut cell = Cell::new(cell_name.as_str());

        cell.add(Polygon::new(
            [
                Point::integer(layer * 10, 0, units),
                Point::integer(layer * 10 + 10, 0, units),
                Point::integer(layer * 10 + 10, 10, units),
                Point::integer(layer * 10, 10, units),
            ],
            Layer::new(layer as u16),
            DataType::new(0),
        ));

        library.add_cell(cell);
    }

    assert_roundtrip(&library);
}

#[test]
fn test_error_invalid_file() {
    let result = Library::read_file("/nonexistent/path/to/file.gds", None);
    assert!(result.is_err());
}

#[test]
fn test_single_cell_with_all_element_types() {
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
        Layer::new(1),
        DataType::new(0),
    ));

    cell.add(Path::new(
        vec![
            Point::integer(150, 50, units),
            Point::integer(200, 50, units),
            Point::integer(200, 100, units),
        ],
        Layer::new(2),
        DataType::new(0),
        Some(PathType::Round),
        Some(Unit::float(5.0, units)),
        None,
        None,
    ));

    cell.add(Text::new(
        "All Elements",
        Point::integer(250, 50, units),
        Layer::new(3),
        DataType::new(0),
        2.0,
        0.0,
        false,
        VerticalPresentation::Middle,
        HorizontalPresentation::Centre,
    ));

    let mut ref_cell = Cell::new("ref_cell");
    ref_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(20, 0, units),
            Point::integer(20, 20, units),
            Point::integer(0, 20, units),
        ],
        Layer::new(4),
        DataType::new(0),
    ));
    library.add_cell(ref_cell);

    cell.add(
        Reference::new("ref_cell".to_string())
            .with_grid(Grid::default().with_origin(Point::integer(300, 50, units))),
    );

    library.add_cell(cell);

    assert_roundtrip(&library);
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

        let reference = Reference::new(element.clone()).with_grid(Grid::new(
            Point::integer(10, 0, units),
            1,
            1,
            Some(Point::integer(10, 10, units)),
            Some(Point::integer(10, 10, units)),
            1.0,
            0.0,
            false,
        ));

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

        let reference = Reference::new("ref_cell").with_grid(Grid::new(
            Point::integer(0, 0, units),
            2,
            2,
            Some(Point::integer(10, 10, units)),
            Some(Point::integer(10, 10, units)),
            2.0,
            FRAC_PI_2,
            true,
        ));

        cell.add(reference.clone());

        let reference = Reference::new("ref_cell").with_grid(Grid::new(
            Point::integer(0, 0, units),
            2,
            1,
            Some(Point::integer(10, 10, units)),
            None,
            2.0,
            FRAC_PI_2,
            true,
        ));

        cell.add(reference.clone());

        let reference = Reference::new("ref_cell").with_grid(Grid::new(
            Point::integer(0, 0, units),
            1,
            2,
            None,
            Some(Point::integer(10, 10, units)),
            2.0,
            FRAC_PI_2,
            true,
        ));

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
        Layer::new(2),
        DataType::new(0),
        Some(PathType::Round),
        Some(Unit::float(5.0, units)),
        None,
        None,
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

    let polygon = Polygon::new(polygon_points, Layer::new(1), DataType::new(0));

    cell.add(polygon);

    library.add_cell(cell);

    let res = library.write_file(&gds_path, 1e-9, 1e-9);

    assert!(res.is_err());
}

#[test]
fn test_invalid_no_cell() {
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("all_elements.gds");

    let mut library = Library::new("reference_test");

    let mut cell = Cell::new("cell");

    let elements = Reference::new("random_cell").flatten(None, &library);

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

#[test]
fn test_empty_cell_roundtrip() {
    let mut library = Library::new("empty_cell_lib");
    library.add_cell(Cell::new("empty"));

    assert_roundtrip(&library);
}

#[test]
fn test_max_coordinate_values_roundtrip() {
    let units = 1e-9;
    let mut library = Library::new("max_coords_lib");
    let mut cell = Cell::new("max_cell");

    cell.add(Polygon::new(
        [
            Point::integer(i32::MAX, i32::MAX, units),
            Point::integer(i32::MIN, i32::MAX, units),
            Point::integer(i32::MIN, i32::MIN, units),
            Point::integer(i32::MAX, i32::MIN, units),
        ],
        Layer::new(1),
        DataType::new(0),
    ));

    library.add_cell(cell);

    assert_roundtrip(&library);
}

#[test]
fn test_special_characters_in_cell_names_roundtrip() {
    let units = 1e-9;
    let mut library = Library::new("special_names_lib");

    for name in [
        "CELL_WITH_UNDERSCORES",
        "Cell123",
        "A_1_B_2",
        "X",
        "cell_00",
    ] {
        let mut cell = Cell::new(name);
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
                Point::integer(0, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(cell);
    }

    assert_roundtrip(&library);
}

#[test]
fn test_all_presentation_combinations_roundtrip() {
    let units = 1e-9;
    let mut library = Library::new("presentations_lib");
    let mut cell = Cell::new("text_cell");

    let vertical = [
        VerticalPresentation::Top,
        VerticalPresentation::Middle,
        VerticalPresentation::Bottom,
    ];
    let horizontal = [
        HorizontalPresentation::Left,
        HorizontalPresentation::Centre,
        HorizontalPresentation::Right,
    ];

    let mut y_offset = 0;
    for vp in &vertical {
        for hp in &horizontal {
            cell.add(Text::new(
                &format!("{vp}_{hp}"),
                Point::integer(0, y_offset, units),
                Layer::new(1),
                DataType::new(0),
                1.0,
                0.0,
                false,
                *vp,
                *hp,
            ));
            y_offset += 100;
        }
    }

    library.add_cell(cell);

    assert_roundtrip(&library);
}

#[test]
fn test_multiple_cells_referencing_same_cell() {
    let units = 1e-9;
    let mut library = Library::new("multi_ref_lib");

    let mut shared_cell = Cell::new("shared");
    shared_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(5, 0, units),
            Point::integer(5, 5, units),
            Point::integer(0, 5, units),
        ],
        Layer::new(1),
        DataType::new(0),
    ));
    library.add_cell(shared_cell);

    let mut cell_a = Cell::new("cell_a");
    cell_a.add(
        Reference::new("shared".to_string())
            .with_grid(Grid::default().with_origin(Point::integer(0, 0, units))),
    );
    library.add_cell(cell_a);

    let mut cell_b = Cell::new("cell_b");
    cell_b.add(
        Reference::new("shared".to_string())
            .with_grid(Grid::default().with_origin(Point::integer(100, 100, units))),
    );
    library.add_cell(cell_b);

    assert_roundtrip(&library);
}

#[test]
fn test_polygon_invalid_layer() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
        ],
        Layer::new(256),
        DataType::new(0),
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_polygon_invalid_data_type() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
        ],
        Layer::new(0),
        DataType::new(256),
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_polygon_too_few_points() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Polygon::new(
        [Point::integer(0, 0, units), Point::integer(10, 0, units)],
        Layer::new(0),
        DataType::new(0),
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_path_invalid_layer() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Path::new(
        vec![Point::integer(0, 0, units), Point::integer(10, 0, units)],
        Layer::new(256),
        DataType::new(0),
        None,
        None,
        None,
        None,
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_path_invalid_data_type() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Path::new(
        vec![Point::integer(0, 0, units), Point::integer(10, 0, units)],
        Layer::new(0),
        DataType::new(256),
        None,
        None,
        None,
        None,
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_text_invalid_layer() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Text::new(
        "hello",
        Point::integer(0, 0, units),
        Layer::new(256),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::default(),
        HorizontalPresentation::default(),
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_text_string_too_long() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    let long_string = "a".repeat(513);
    cell.add(Text::new(
        &long_string,
        Point::integer(0, 0, units),
        Layer::new(0),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::default(),
        HorizontalPresentation::default(),
    ));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_text_string_at_max_length() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    let max_string = "a".repeat(512);
    cell.add(Text::new(
        &max_string,
        Point::integer(0, 0, units),
        Layer::new(0),
        DataType::new(0),
        1.0,
        0.0,
        false,
        VerticalPresentation::default(),
        HorizontalPresentation::default(),
    ));
    library.add_cell(cell);
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("valid.gds");
    assert!(library.write_file(&gds_path, 1e-9, 1e-9).is_ok());
}

#[test]
fn test_cell_name_too_long() {
    let mut library = Library::new("lib");
    let long_name = "a".repeat(33);
    let cell = Cell::new(&long_name);
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_cell_name_invalid_characters() {
    let mut library = Library::new("lib");
    let cell = Cell::new("cell with spaces");
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_cell_name_valid_special_characters() {
    let mut library = Library::new("lib");
    let cell = Cell::new("CELL_with$valid?chars");
    library.add_cell(cell);
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("valid.gds");
    assert!(library.write_file(&gds_path, 1e-9, 1e-9).is_ok());
}

#[test]
fn test_cell_name_at_max_length() {
    let mut library = Library::new("lib");
    let max_name = "a".repeat(32);
    let cell = Cell::new(&max_name);
    library.add_cell(cell);
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("valid.gds");
    assert!(library.write_file(&gds_path, 1e-9, 1e-9).is_ok());
}

#[test]
fn test_reference_columns_exceed_max() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut base_cell = Cell::new("base");
    base_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        Layer::new(0),
        DataType::new(0),
    ));
    library.add_cell(base_cell);

    let mut cell = Cell::new("cell");
    cell.add(Reference::new("base").with_grid(Grid::default().with_columns(32768).with_rows(1)));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_reference_rows_exceed_max() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut base_cell = Cell::new("base");
    base_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        Layer::new(0),
        DataType::new(0),
    ));
    library.add_cell(base_cell);

    let mut cell = Cell::new("cell");
    cell.add(Reference::new("base").with_grid(Grid::default().with_columns(1).with_rows(32768)));
    library.add_cell(cell);
    assert_write_validation_error(&library);
}

#[test]
fn test_reference_col_row_at_max() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut base_cell = Cell::new("base");
    base_cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        Layer::new(0),
        DataType::new(0),
    ));
    library.add_cell(base_cell);

    let mut cell = Cell::new("cell");
    cell.add(
        Reference::new("base").with_grid(Grid::default().with_columns(32767).with_rows(32767)),
    );
    library.add_cell(cell);
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("valid.gds");
    assert!(library.write_file(&gds_path, 1e-9, 1e-9).is_ok());
}

#[test]
fn test_polygon_layer_and_data_type_at_boundary() {
    let units = 1e-9;
    let mut library = Library::new("lib");
    let mut cell = Cell::new("cell");
    cell.add(Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(10, 0, units),
            Point::integer(10, 10, units),
            Point::integer(0, 10, units),
        ],
        Layer::new(255),
        DataType::new(255),
    ));
    library.add_cell(cell);
    let temp_dir = tempdir().unwrap();
    let gds_path = temp_dir.path().join("valid.gds");
    assert!(library.write_file(&gds_path, 1e-9, 1e-9).is_ok());
}
