Units are used in every data structure, a `Unit` represents a value with a unit of measurement.

For integer units, the default unit is `nm` (nanometer).

For floating-point units, the default unit is `um` (micrometer).

When using units, you can construct them in the following way:

```rs
use gdsr::Unit;

let nm_value = Unit::integer(10, 1e-9);
let um_value = Unit::float(10.0, 1e-6);
```

Or if you want to use the default units, you can simply write:

```rs
use gdsr::Unit;

let nm_value = Unit::default_integer(10);
let um_value = Unit::default_float(10.0);
```

You may not be using `Unit` directly in your code, but you will be using `Point`.

We can similarly construct points in the following way:

```rs
use gdsr::Point;

let nm_point = Point::integer(10, 20, 1e-9);
let um_point = Point::float(10.0, 20.0, 1e-6);
```

Or if you want to use the default units, you can simply write:

```rs
use gdsr::Point;

let nm_point = Point::default_integer(10, 20);
let um_point = Point::default_float(10.0, 20.0);
```

We model units this way so that there is no confusion about what unit is being used.

When writing a `Library`, with `Library::write_file`, you must specify two units, the user units, and the database units.
This may be slightly confusing, but there is a good reason for it.

The user units are used simply for your GDSII editor, these let you see values in a more human-readable format.
For most users I would imagine that they would want to use micrometers (`um`) for their user units and nanometers (`nm`) for their database units.

When reading a GDSII file into a `Library`, with `Library::read_file`, you can only specify the "user" units.
These units are what is used when creating `Point`s and `Unit`s. This allows you to work with these values in a consistent way. If you do not provide this, then values will be with units of 1e-9, which is not recommended, though it is fine to work with these values.

## Important

I'd like to first show a scenario, and explain why it may result in unexpected behavior.

```rs
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

    cell.add(polygon);

    library.add(cell);

    library.write_file("main.gds", 1e-6, 1e-6).unwrap();
}
```

Note here we are writing with user units `1e-3` and database units `1e-6`.

But `1e-6` is greater than `1e-9`, which is the units we are using for our `Point`s and `Unit`s.

When we set database units, this is setting the minimal value that we can see in our GDSII,
since all values are less than that, all values in the polygon will be scaled to 0.
