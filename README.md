# GDSR
GDSII manipulation, written in rust.

> [!WARNING]
> This is a work in progress and is not yet ready for production use.

gdsr is currently being repurposed to being a rust crate at the core. Python bindings will be added back in the future.

## Inspiration

My main inspiration comes from [gdstk](https://github.com/heitzmann/gdstk). If you are looking for an extremely fast gds manipulation python package then i would strongly recommend heading over and having a look at his work.

Other inspirations include:
- [gdsfactory](https://github.com/gdsfactory/gdsfactory)
- [klayout](https://www.klayout.org/klayout-pypi/)

## Getting Started

A simple program below shows the easy to use interface.

```rust
use gdsr_core::{Cell, Grid, Library, Polygon, Reference};

fn main() {
    let mut library = Library::new("main");

    let mut cell = Cell::new("main_cell");

    let polygon = Polygon::new([(0, 0), (1, 0), (1, 1), (0, 1)], 1, 0);

    let reference = Reference::new(
        polygon,
        Grid::new((0, 0), 5, 5, (2, 0), (0, 2), 2.0, 0.0, false),
    );

    cell.add(reference);

    library.add(cell);

    let _res = library.to_gds("main.gds", 1e-9, 1e-9);
}
```

## Need help?

Head over to the [discussions page](https://github.com/MatthewMckee4/gdsr/discussions) and create a new discussion there or have a look at the [issues page](https://github.com/MatthewMckee4/gdsr/issues) to see if anyone has had the same issue as you.
