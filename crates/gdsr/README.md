# GDSR

[![Crates.io](https://img.shields.io/crates/v/gdsr)](https://crates.io/crates/gdsr)
[![Documentation](https://img.shields.io/docsrs/gdsr)](https://docs.rs/gdsr)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/MatthewMckee4/gdsr/blob/main/LICENSE)

GDSII manipulation, written in Rust.

> **Warning:** This is a work in progress and is not yet ready for production use.

## Getting Started

```rust
use gdsr::{Cell, Grid, Library, Point, Polygon, Reference};

fn main() {
    let units = 1e-9;

    let mut library = Library::new("main");

    let mut cell = Cell::new("main_cell");

    let polygon = Polygon::new(
        [
            Point::integer(0, 0, units),
            Point::integer(1, 0, units),
            Point::integer(1, 1, units),
            Point::integer(0, 1, units),
        ],
        1,
        0,
    );

    let reference = Reference::new(polygon).with_grid(
        Grid::default()
            .with_columns(5)
            .with_rows(5)
            .with_spacing_x(Some(Point::integer(2, 0, units)))
            .with_spacing_y(Some(Point::integer(0, 2, units))),
    );

    cell.add(reference);

    library.add_cell(cell);

    library.write_file("main.gds", 1e-9, 1e-9).unwrap();
}
```

## Documentation

Documentation is available at [matthewmckee4.github.io/gdsr](https://matthewmckee4.github.io/gdsr/)

## License

gdsr is licensed under the MIT License.
