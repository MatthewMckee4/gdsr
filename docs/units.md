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
These units are what is used when creating `Point`s and `Unit`s. This allows you to work with these values in a consistent way. If you do not provide this, then values will be with units of 1, which is not recommended, though it is fine to work with these values.
