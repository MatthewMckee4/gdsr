#![expect(clippy::print_stdout, reason = "example binary that prints output")]

use gdsr::{Cell, DataType, Grid, Layer, Library, Point, Polygon, Reference};

fn compare_libraries(lib1: &Library, lib2: &Library) -> bool {
    let mut has_differences = false;

    if lib1.cells().len() != lib2.cells().len() {
        println!(
            "MISMATCH: cell count differs: {} vs {}",
            lib1.cells().len(),
            lib2.cells().len()
        );
        has_differences = true;
    }

    let mut cell_names: Vec<&str> = lib1.cells().keys().map(String::as_str).collect();
    cell_names.sort_unstable();

    for name in &cell_names {
        let Some(cell1) = lib1.get_cell(name) else {
            continue;
        };
        let Some(cell2) = lib2.get_cell(name) else {
            println!("MISMATCH: cell '{name}' missing from re-read library");
            has_differences = true;
            continue;
        };

        if cell1 != cell2 {
            has_differences = true;
            println!("\nMISMATCH in cell '{name}':");
            print_reference_diffs(cell1, cell2);
            print_element_diffs(cell1, cell2);
        }
    }

    has_differences
}

fn print_reference_diffs(cell1: &gdsr::Cell, cell2: &gdsr::Cell) {
    let refs1: Vec<_> = cell1.references().collect();
    let refs2: Vec<_> = cell2.references().collect();

    if refs1.len() != refs2.len() {
        println!("  reference count: {} vs {}", refs1.len(), refs2.len());
    }

    for (i, (r1, r2)) in refs1.iter().zip(refs2.iter()).enumerate() {
        if r1 != r2 {
            println!("  Reference {i} differs:");
            println!("    original: {r1}");
            println!("    re-read:  {r2}");
            print_grid_diffs(r1.grid(), r2.grid());
        }
    }
}

fn print_grid_diffs(g1: &gdsr::Grid, g2: &gdsr::Grid) {
    if g1.origin() != g2.origin() {
        println!("    origin:    {:?} vs {:?}", g1.origin(), g2.origin());
    }
    if g1.columns() != g2.columns() || g1.rows() != g2.rows() {
        println!(
            "    cols/rows: {}x{} vs {}x{}",
            g1.columns(),
            g1.rows(),
            g2.columns(),
            g2.rows()
        );
    }
    if g1.spacing_x() != g2.spacing_x() {
        println!(
            "    spacing_x: {:?} vs {:?}",
            g1.spacing_x(),
            g2.spacing_x()
        );
    }
    if g1.spacing_y() != g2.spacing_y() {
        println!(
            "    spacing_y: {:?} vs {:?}",
            g1.spacing_y(),
            g2.spacing_y()
        );
    }
    if g1.angle() != g2.angle() {
        println!("    angle:     {} vs {}", g1.angle(), g2.angle());
    }
    if g1.magnification() != g2.magnification() {
        println!(
            "    mag:       {} vs {}",
            g1.magnification(),
            g2.magnification()
        );
    }
    if g1.x_reflection() != g2.x_reflection() {
        println!(
            "    x_reflect: {} vs {}",
            g1.x_reflection(),
            g2.x_reflection()
        );
    }
}

fn print_element_diffs(cell1: &gdsr::Cell, cell2: &gdsr::Cell) {
    let polys1: Vec<_> = cell1.polygons().collect();
    let polys2: Vec<_> = cell2.polygons().collect();
    if polys1.len() != polys2.len() {
        println!("  polygon count: {} vs {}", polys1.len(), polys2.len());
    }
    for (i, (p1, p2)) in polys1.iter().zip(polys2.iter()).enumerate() {
        if p1 != p2 {
            println!("  Polygon {i} differs:");
            println!("    original: {p1:?}");
            println!("    re-read:  {p2:?}");
        }
    }

    let paths1: Vec<_> = cell1.paths().collect();
    let paths2: Vec<_> = cell2.paths().collect();
    if paths1.len() != paths2.len() {
        println!("  path count: {} vs {}", paths1.len(), paths2.len());
    }
    for (i, (p1, p2)) in paths1.iter().zip(paths2.iter()).enumerate() {
        if p1 != p2 {
            println!("  Path {i} differs:");
            println!("    original: {p1:?}");
            println!("    re-read:  {p2:?}");
        }
    }

    let texts1: Vec<_> = cell1.texts().collect();
    let texts2: Vec<_> = cell2.texts().collect();
    if texts1.len() != texts2.len() {
        println!("  text count: {} vs {}", texts1.len(), texts2.len());
    }
    for (i, (t1, t2)) in texts1.iter().zip(texts2.iter()).enumerate() {
        if t1 != t2 {
            println!("  Text {i} differs:");
            println!("    original: {t1:?}");
            println!("    re-read:  {t2:?}");
        }
    }
}

/// Dump all array references found in a library read from an external file.
fn dump_arefs(lib: &Library) {
    let mut cell_names: Vec<&str> = lib.cells().keys().map(String::as_str).collect();
    cell_names.sort_unstable();

    let mut count = 0;
    for name in &cell_names {
        if let Some(cell) = lib.get_cell(name) {
            for r in cell.references() {
                let g = r.grid();
                if g.columns() > 1 || g.rows() > 1 {
                    count += 1;
                    if count <= 20 {
                        println!("  Cell '{name}': {r}");
                    }
                }
            }
        }
    }
    if count > 20 {
        println!("  ... and {} more ARef references", count - 20);
    }
    println!("  Total ARef references: {count}");
}

fn roundtrip_file(input_path: &str) {
    let output_path = "roundtrip_output.gds";
    let user_units = 1e-6;
    let db_units = 1e-9;

    println!("=== File roundtrip: {input_path} ===");
    let lib1 = match Library::read_file(input_path, Some(db_units)) {
        Ok(lib) => lib,
        Err(e) => {
            println!("  Error reading: {e}");
            return;
        }
    };
    println!(
        "  Library '{}' with {} cells",
        lib1.name(),
        lib1.cells().len()
    );

    println!("\n  Array references found:");
    dump_arefs(&lib1);

    if let Err(e) = lib1.write_file(output_path, user_units, db_units) {
        println!("\n  Error writing: {e}");
        println!("  (skipping re-read comparison)");
        return;
    }

    let lib2 = match Library::read_file(output_path, Some(db_units)) {
        Ok(lib) => lib,
        Err(e) => {
            println!("  Error re-reading: {e}");
            return;
        }
    };

    if !compare_libraries(&lib1, &lib2) {
        println!("  Libraries are identical after roundtrip!");
    }

    std::fs::remove_file(output_path).ok();
}

/// Creates a library with arrayed references at non-zero origins to test `ARef` roundtrip.
fn roundtrip_constructed() {
    let output_path = "roundtrip_constructed.gds";
    let user_units = 1e-6;
    let db_units = 1e-9;

    println!("\n=== Constructed library roundtrip (ARef at non-zero origin) ===");

    let mut library = Library::new("test");

    let mut base = Cell::new("base");
    base.add(Polygon::new(
        vec![
            Point::default_integer(0, 0),
            Point::default_integer(100, 0),
            Point::default_integer(100, 100),
            Point::default_integer(0, 100),
        ],
        Layer::new(1),
        DataType::new(0),
    ));

    let mut top = Cell::new("top");

    top.add(Reference::new("base").with_grid(Grid::new(
        Point::default_integer(500, 500),
        1,
        1,
        None,
        None,
        1.0,
        0.0,
        false,
    )));

    top.add(Reference::new("base").with_grid(Grid::new(
        Point::default_integer(0, 0),
        3,
        2,
        Some(Point::default_integer(200, 0)),
        Some(Point::default_integer(0, 200)),
        1.0,
        0.0,
        false,
    )));

    top.add(Reference::new("base").with_grid(Grid::new(
        Point::default_integer(1000, 1000),
        3,
        2,
        Some(Point::default_integer(200, 0)),
        Some(Point::default_integer(0, 200)),
        1.0,
        0.0,
        false,
    )));

    top.add(Reference::new("base").with_grid(Grid::new(
        Point::default_integer(2000, 0),
        4,
        1,
        Some(Point::default_integer(300, 0)),
        None,
        1.0,
        0.0,
        false,
    )));

    top.add(Reference::new("base").with_grid(Grid::new(
        Point::default_integer(0, 2000),
        1,
        3,
        None,
        Some(Point::default_integer(0, 300)),
        1.0,
        0.0,
        false,
    )));

    library.add_cell(base);
    library.add_cell(top);

    println!("  Original library references:");
    for r in library.get_cell("top").iter().flat_map(|c| c.references()) {
        println!("    {r}");
    }

    if let Err(e) = library.write_file(output_path, user_units, db_units) {
        println!("  Error writing: {e}");
        return;
    }

    let lib2 = match Library::read_file(output_path, Some(db_units)) {
        Ok(lib) => lib,
        Err(e) => {
            println!("  Error re-reading: {e}");
            return;
        }
    };

    println!("\n  Re-read library references:");
    for r in lib2.get_cell("top").iter().flat_map(|c| c.references()) {
        println!("    {r}");
    }

    println!();
    if !compare_libraries(&library, &lib2) {
        println!("  Libraries are identical after roundtrip!");
    }

    std::fs::remove_file(output_path).ok();
}

fn main() {
    let input_path = std::env::args().nth(1);

    if let Some(ref path) = input_path {
        roundtrip_file(path);
    }

    roundtrip_constructed();
}
