use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;
use tempfile::tempdir;

use super::arbitrary::{
    arb_gds_box, arb_gds_path, arb_gds_polygon, arb_gds_text, arb_integer_point, arb_structure_name,
};
use crate::*;

fn assert_roundtrip(library: &Library) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.gds");
    library.write_file(&path, 1e-9, 1e-9).unwrap();
    let read = Library::read_file(&path, Some(DEFAULT_INTEGER_UNITS)).unwrap();
    assert_eq!(*library, read, "{library:#?}\n{read:#?}");
}

fn make_library(cell_name: &str, elements: Vec<Element>) -> Library {
    let mut library = Library::new("roundtrip_lib");
    let mut cell = Cell::new(cell_name);
    for e in elements {
        cell.add(e);
    }
    library.add_cell(cell);
    library
}

#[quickcheck]
fn polygon_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let polygon = arb_gds_polygon(&mut g);
    let library = make_library("cell", vec![polygon.into()]);
    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn path_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let path = arb_gds_path(&mut g);
    let library = make_library("cell", vec![path.into()]);
    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn text_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let text = arb_gds_text(&mut g);
    let library = make_library("cell", vec![text.into()]);
    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn box_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let gds_box = arb_gds_box(&mut g);
    let library = make_library("cell", vec![gds_box.into()]);
    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn mixed_elements_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let elements: Vec<Element> = vec![
        arb_gds_polygon(&mut g).into(),
        arb_gds_path(&mut g).into(),
        arb_gds_box(&mut g).into(),
        arb_gds_text(&mut g).into(),
    ];
    let library = make_library("cell", elements);
    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn cell_reference_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let mut library = Library::new("roundtrip_lib");

    let mut base_cell = Cell::new("base");
    base_cell.add(arb_gds_polygon(&mut g));
    library.add_cell(base_cell);

    let cols = 2 + (u32::arbitrary(&mut g) % 4);
    let rows = 2 + (u32::arbitrary(&mut g) % 4);
    let mut cell = Cell::new("top");
    cell.add(
        Reference::new("base".to_string()).with_grid(
            Grid::default()
                .with_origin(arb_integer_point(&mut g))
                .with_columns(cols)
                .with_rows(rows)
                .with_spacing_x(Some(Point::integer(20, 0, 1e-9)))
                .with_spacing_y(Some(Point::integer(0, 20, 1e-9))),
        ),
    );
    library.add_cell(cell);

    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn multi_cell_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let mut library = Library::new("roundtrip_lib");

    let count = 2 + (usize::arbitrary(&mut g) % 4);
    for i in 0..count {
        let name = format!("cell_{i}");
        let mut cell = Cell::new(&name);
        cell.add(arb_gds_polygon(&mut g));
        if bool::arbitrary(&mut g) {
            cell.add(arb_gds_text(&mut g));
        }
        library.add_cell(cell);
    }

    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn nested_reference_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let mut library = Library::new("roundtrip_lib");

    let mut cell_a = Cell::new("cell_a");
    cell_a.add(arb_gds_polygon(&mut g));
    library.add_cell(cell_a);

    let mut cell_b = Cell::new("cell_b");
    cell_b.add(
        Reference::new("cell_a".to_string())
            .with_grid(Grid::default().with_origin(arb_integer_point(&mut g))),
    );
    library.add_cell(cell_b);

    let mut cell_c = Cell::new("cell_c");
    cell_c.add(
        Reference::new("cell_b".to_string())
            .with_grid(Grid::default().with_origin(arb_integer_point(&mut g))),
    );
    library.add_cell(cell_c);

    assert_roundtrip(&library);
    true
}

#[quickcheck]
fn cell_name_roundtrip(_seed: u8) -> bool {
    let mut g = Gen::new(30);
    let name = arb_structure_name(&mut g);
    let mut library = Library::new("roundtrip_lib");
    let mut cell = Cell::new(&name);
    cell.add(arb_gds_polygon(&mut g));
    library.add_cell(cell);
    assert_roundtrip(&library);
    true
}
