use crate::units::Unit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: Unit,
    y: Unit,
}

impl Point {
    #[must_use]
    pub const fn new(x: Unit, y: Unit) -> Self {
        Self { x, y }
    }

    /// Gets the x coordinate of the point.
    #[must_use]
    pub const fn x(&self) -> Unit {
        self.x
    }

    /// Gets the y coordinate of the point.
    #[must_use]
    pub const fn y(&self) -> Unit {
        self.y
    }

    /// Gets a mutable reference to the x coordinate of the point.
    pub fn x_mut(&mut self) -> &mut Unit {
        &mut self.x
    }

    /// Gets a mutable reference to the y coordinate of the point.
    pub fn y_mut(&mut self) -> &mut Unit {
        &mut self.y
    }

    /// Sets the x coordinate of the point.
    pub const fn set_x(&mut self, x: Unit) {
        self.x = x;
    }

    /// Sets the y coordinate of the point.
    pub const fn set_y(&mut self, y: Unit) {
        self.y = y;
    }

    /// Converts both coordinates to integer units.
    ///
    /// # Returns
    /// A new `Point` with both x and y coordinates converted to `Unit::Integer`
    #[must_use]
    pub fn to_integer_unit(&self) -> Self {
        Self {
            x: self.x.to_integer_unit(),
            y: self.y.to_integer_unit(),
        }
    }

    /// Converts both coordinates to float units.
    ///
    /// # Returns
    /// A new `Point` with both x and y coordinates converted to `Unit::Float`
    #[must_use]
    pub fn to_float_unit(&self) -> Self {
        Self {
            x: self.x.to_float_unit(),
            y: self.y.to_float_unit(),
        }
    }

    /// Rotates the point around the origin (0, 0) by the given angle in radians.
    ///
    /// # Arguments
    /// * `angle` - The rotation angle in radians (positive = counter-clockwise)
    ///
    /// # Returns
    /// A new `Point` representing the rotated position
    #[must_use]
    pub fn rotate(&self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Convert to float units and extract values
        let x_float = self.x.to_float_unit();
        let y_float = self.y.to_float_unit();

        let Unit::Float {
            value: x_val,
            user_unit: x_user_unit,
            db_unit: x_db_unit,
        } = x_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: y_val,
            user_unit: y_user_unit,
            db_unit: y_db_unit,
        } = y_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        // Calculate real world values
        let x_real = x_val * x_user_unit;
        let y_real = y_val * y_user_unit;

        // Apply rotation transformation
        let new_x_real = x_real.mul_add(cos_a, -(y_real * sin_a));
        let new_y_real = x_real.mul_add(sin_a, y_real * cos_a);

        Self {
            x: Unit::float(new_x_real, 1.0, x_db_unit),
            y: Unit::float(new_y_real, 1.0, y_db_unit),
        }
    }
}

// From implementations for common types
impl From<[i32; 2]> for Point {
    fn from(arr: [i32; 2]) -> Self {
        Self {
            x: Unit::default_integer(arr[0]),
            y: Unit::default_integer(arr[1]),
        }
    }
}

impl From<(i32, i32)> for Point {
    fn from(tuple: (i32, i32)) -> Self {
        Self {
            x: Unit::default_integer(tuple.0),
            y: Unit::default_integer(tuple.1),
        }
    }
}

impl From<[f64; 2]> for Point {
    fn from(arr: [f64; 2]) -> Self {
        Self {
            x: Unit::default_float(arr[0]),
            y: Unit::default_float(arr[1]),
        }
    }
}

impl From<(f64, f64)> for Point {
    fn from(tuple: (f64, f64)) -> Self {
        Self {
            x: Unit::default_float(tuple.0),
            y: Unit::default_float(tuple.1),
        }
    }
}

impl From<[Unit; 2]> for Point {
    fn from(arr: [Unit; 2]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
        }
    }
}

impl From<(Unit, Unit)> for Point {
    fn from(tuple: (Unit, Unit)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod creation {
        use super::*;

        #[test]
        fn with_integers() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn with_floats() {
            let point = Point::new(Unit::float(1.5, 1e-6, 1e-9), Unit::float(2.5, 1e-6, 1e-9));
            assert_eq!(point.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn with_mixed_units() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_i32_array() {
            let point = Point::from([100, 200]);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn from_i32_tuple() {
            let point = Point::from((100, 200));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn from_f64_array() {
            let point = Point::from([1.5, 2.5]);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_tuple() {
            let point = Point::from((1.5, 2.5));
            assert_eq!(point.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_unit_array() {
            let units = [Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9)];
            let point = Point::from(units);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_unit_tuple() {
            let point = Point::from((Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9)));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn into_works_with_type_inference() {
            let point: Point = [100, 200].into();
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            let point: Point = (1.5, 2.5).into();
            assert_eq!(point.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_i32_array_with_negative_values() {
            let point = Point::from([-50, -100]);
            assert_eq!(point.x(), Unit::integer(-50, 1e-9));
            assert_eq!(point.y(), Unit::integer(-100, 1e-9));
        }

        #[test]
        fn from_i32_tuple_with_zero() {
            let point = Point::from((0, 0));
            assert_eq!(point.x(), Unit::integer(0, 1e-9));
            assert_eq!(point.y(), Unit::integer(0, 1e-9));
        }

        #[test]
        fn from_f64_array_with_negative_values() {
            let point = Point::from([-3.5, -7.2]);
            assert_eq!(point.x(), Unit::float(-3.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(-7.2, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_tuple_with_zero() {
            let point = Point::from((0.0, 0.0));
            assert_eq!(point.x(), Unit::float(0.0, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(0.0, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_with_large_values() {
            let point = Point::from([1000.0, 2000.5]);
            assert_eq!(point.x(), Unit::float(1000.0, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2000.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_unit_array_with_different_scales() {
            let units = [Unit::integer(1000, 1e-9), Unit::integer(1, 1e-6)];
            let point = Point::from(units);
            assert_eq!(point.x(), Unit::integer(1000, 1e-9));
            assert_eq!(point.y(), Unit::integer(1, 1e-6));
            // They should be equal in real-world value
            assert_eq!(point.x(), point.y());
        }

        #[test]
        fn chaining_from_and_methods() {
            let point = Point::from([100, 200]);
            let rotated = point.rotate(std::f64::consts::PI);

            // Original point should be unchanged
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            // Rotated point should be approximately (-100, -200) in real units
            let x_real = match rotated.x().to_float_unit() {
                Unit::Float {
                    value, user_unit, ..
                } => value * user_unit,
                Unit::Integer { .. } => unreachable!(),
            };
            let y_real = match rotated.y().to_float_unit() {
                Unit::Float {
                    value, user_unit, ..
                } => value * user_unit,
                Unit::Integer { .. } => unreachable!(),
            };

            assert!((x_real - (-100e-9)).abs() < 1e-15);
            assert!((y_real - (-200e-9)).abs() < 1e-15);
        }

        #[test]
        fn function_accepting_into_point() {
            fn double_coords(p: impl Into<Point>) -> Point {
                let point = p.into();
                Point::new(point.x() * 2, point.y() * 2)
            }

            let p1 = double_coords([10, 20]);
            assert_eq!(p1.x(), Unit::integer(20, 1e-9));
            assert_eq!(p1.y(), Unit::integer(40, 1e-9));

            let p2 = double_coords((5.0, 7.5));
            assert_eq!(p2.x(), Unit::float(10.0, 1e-6, 1e-9));
            assert_eq!(p2.y(), Unit::float(15.0, 1e-6, 1e-9));
        }

        #[test]
        fn getter_and_setter() {
            let mut point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));

            // Test getters
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            // Test setters
            point.set_x(Unit::integer(300, 1e-9));
            point.set_y(Unit::integer(400, 1e-9));

            assert_eq!(point.x(), Unit::integer(300, 1e-9));
            assert_eq!(point.y(), Unit::integer(400, 1e-9));
        }

        #[test]
        fn mutable_references() {
            let mut point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));

            // Test getting mutable references
            let x_ref = point.x_mut();
            *x_ref = Unit::integer(300, 1e-9);

            let y_ref = point.y_mut();
            *y_ref = Unit::integer(400, 1e-9);

            assert_eq!(point.x(), Unit::integer(300, 1e-9));
            assert_eq!(point.y(), Unit::integer(400, 1e-9));
        }

        #[test]
        fn modify_via_mutable_reference() {
            let mut point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));

            // Modify using Unit's set_db_units method
            point.x_mut().set_db_units(1e-6);
            point.y_mut().set_user_units(1e-3);

            match point.x() {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }

            match point.y() {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 200);
                    assert_eq!(db_unit, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn chained_mutable_operations() {
            let mut point = Point::new(Unit::float(1.5, 1e-6, 1e-9), Unit::float(2.5, 1e-6, 1e-9));

            // Chain operations on mutable references
            *point.x_mut() = point.x().with_user_units(1e-3);
            *point.y_mut() = point.y().with_db_units(1e-12);

            match point.x() {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(user_unit, 1e-3);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }

            match point.y() {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 2.5);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn to_integer_unit_from_integers() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));
            let converted = point.to_integer_unit();

            assert_eq!(converted.x(), Unit::integer(100, 1e-9));
            assert_eq!(converted.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn to_integer_unit_from_floats() {
            let point = Point::new(Unit::float(1.007, 1.0, 1e-3), Unit::float(2.015, 1.0, 1e-3));
            let converted = point.to_integer_unit();

            match converted.x() {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 1007);
                    assert_eq!(db_unit, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }

            match converted.y() {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 2015);
                    assert_eq!(db_unit, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_integer_unit_from_mixed() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9));
            let converted = point.to_integer_unit();

            assert_eq!(converted.x(), Unit::integer(100, 1e-9));

            match converted.y() {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 2500);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_float_unit_from_floats() {
            let point = Point::new(Unit::float(1.5, 1e-6, 1e-9), Unit::float(2.5, 1e-6, 1e-9));
            let converted = point.to_float_unit();

            assert_eq!(converted.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(converted.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn to_float_unit_from_integers() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));
            let converted = point.to_float_unit();

            match converted.x() {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 1e-7).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }

            match converted.y() {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 2e-7).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn to_float_unit_from_mixed() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9));
            let converted = point.to_float_unit();

            match converted.x() {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 1e-7).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }

            assert_eq!(converted.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn roundtrip_integer_to_float_to_integer() {
            let original = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9));
            let as_float = original.to_float_unit();
            let back_to_int = as_float.to_integer_unit();

            assert_eq!(back_to_int.x(), original.x());
            assert_eq!(back_to_int.y(), original.y());
        }

        #[test]
        fn conversion_preserves_equality() {
            let point1 = Point::new(Unit::integer(1000, 1e-9), Unit::integer(2000, 1e-9));
            let point2 = point1.to_float_unit();

            // The coordinates should be equal in real-world value
            assert_eq!(point1.x(), point2.x());
            assert_eq!(point1.y(), point2.y());
        }
    }

    mod rotation {
        use std::f64::consts::PI;

        use super::*;

        fn extract_real_value(unit: Unit) -> f64 {
            match unit.to_float_unit() {
                Unit::Float {
                    value, user_unit, ..
                } => value * user_unit,
                Unit::Integer { .. } => unreachable!(),
            }
        }

        #[test]
        fn rotate_90_degrees() {
            let point = Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9));
            let rotated = point.rotate(PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 1e-6).abs() < 1e-15);
        }

        #[test]
        fn rotate_180_degrees() {
            let point = Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9));
            let rotated = point.rotate(PI);

            assert!((extract_real_value(rotated.x()) - (-1e-6)).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_270_degrees() {
            let point = Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9));
            let rotated = point.rotate(3.0 * PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - (-1e-6)).abs() < 1e-15);
        }

        #[test]
        fn rotate_360_degrees() {
            let point = Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9));
            let rotated = point.rotate(2.0 * PI);

            assert!((extract_real_value(rotated.x()) - 1e-6).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_arbitrary_point() {
            let point = Point::new(Unit::float(3.0, 1e-6, 1e-9), Unit::float(4.0, 1e-6, 1e-9));
            let rotated = point.rotate(PI / 4.0); // 45 degrees

            let expected_x = 3e-6f64.mul_add((PI / 4.0).cos(), -(4e-6 * (PI / 4.0).sin()));
            let expected_y = 3e-6f64.mul_add((PI / 4.0).sin(), 4e-6 * (PI / 4.0).cos());

            assert!((extract_real_value(rotated.x()) - expected_x).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - expected_y).abs() < 1e-15);
        }
    }
}
