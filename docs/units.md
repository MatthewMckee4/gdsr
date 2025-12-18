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

When writing a `Library`, you must specify two units, the user units, and the database units.
This may be slightly confusing, but there is a good reason for it.
