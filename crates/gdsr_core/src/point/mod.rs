use std::ops::{Add, Div, Mul, Sub};

use crate::{
    Movable, Transformable, Transformation,
    units::{Unit, UnitsError},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: Unit,
    y: Unit,
}

impl Point {
    pub fn new(x: impl Into<Unit>, y: impl Into<Unit>) -> Result<Self, UnitsError> {
        let x = x.into();
        let y = y.into();

        if x.user_unit() != y.user_unit() {
            return Err(UnitsError::MismatchedUnits(x.user_unit(), y.user_unit()));
        }

        Ok(Self { x, y })
    }

    #[must_use]
    pub fn set_same_units(&self, other: &Self) -> Self {
        self.with_db_unit(other.db_unit())
            .with_user_unit(other.user_unit())
    }

    #[must_use]
    pub fn user_unit(&self) -> f64 {
        if self.x.user_unit() == self.y.user_unit() {
            self.x.user_unit()
        } else {
            panic!("Point x and y coordinates must have the same user_unit.")
        }
    }

    #[must_use]
    pub fn db_unit(&self) -> f64 {
        if self.x.db_unit() == self.y.db_unit() {
            self.x.db_unit()
        } else {
            panic!("Point x and y coordinates must have the same user_unit.")
        }
    }

    #[must_use]
    pub fn with_user_unit(&self, user_unit: f64) -> Self {
        Self {
            x: self.x.with_user_units(user_unit),
            y: self.y.with_user_units(user_unit),
        }
    }

    #[must_use]
    pub fn with_db_unit(&self, db_unit: f64) -> Self {
        Self {
            x: self.x.with_db_units(db_unit),
            y: self.y.with_db_units(db_unit),
        }
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

    /// Sets the x coordinate of the point.
    ///
    /// # Panics
    /// Panics if the new x coordinate has a different `user_unit` than the existing y coordinate.
    pub fn set_x(&mut self, x: Unit) {
        assert_eq!(
            x.user_unit(),
            self.y.user_unit(),
            "Point x and y coordinates must have the same user_unit. x.user_unit() = {}, y.user_unit() = {}",
            x.user_unit(),
            self.y.user_unit()
        );
        self.x = x;
    }

    /// Sets the y coordinate of the point.
    ///
    /// # Panics
    /// Panics if the new y coordinate has a different `user_unit` than the existing x coordinate.
    pub fn set_y(&mut self, y: Unit) {
        assert_eq!(
            y.user_unit(),
            self.x.user_unit(),
            "Point x and y coordinates must have the same user_unit. x.user_unit() = {}, y.user_unit() = {}",
            self.x.user_unit(),
            y.user_unit()
        );
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
    pub fn to_float_unit(&self, user_units: f64) -> Self {
        Self {
            x: self.x.to_float_unit(user_units),
            y: self.y.to_float_unit(user_units),
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
        let x_float = self.x.to_float_unit(1e-6);
        let y_float = self.y.to_float_unit(1e-6);

        let Unit::Float {
            value: x_val,
            user_units: x_user_unit,
            db_units: x_db_unit,
        } = x_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: y_val,
            user_units: y_user_unit,
            db_units: y_db_unit,
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

    /// Rotates the point around an arbitrary center point by the given angle in radians.
    ///
    /// # Arguments
    /// * `center` - The center point to rotate around
    /// * `angle` - The rotation angle in radians (positive = counter-clockwise)
    ///
    /// # Returns
    /// A new `Point` representing the rotated position
    #[must_use]
    pub fn rotate_around_point(&self, angle: f64, center: impl Into<Self>) -> Self {
        let center = center.into();
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Convert to float units and extract values
        let x_float = self.x.to_float_unit(1e-6);
        let y_float = self.y.to_float_unit(1e-6);
        let cx_float = center.x.to_float_unit(1e-6);
        let cy_float = center.y.to_float_unit(1e-6);

        let Unit::Float {
            value: x_val,
            user_units: x_user_unit,
            db_units: x_db_unit,
        } = x_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: y_val,
            user_units: y_user_unit,
            db_units: y_db_unit,
        } = y_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: cx_val,
            user_units: cx_user_unit,
            ..
        } = cx_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: cy_val,
            user_units: cy_user_unit,
            ..
        } = cy_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        // Calculate real world values
        let x_real = x_val * x_user_unit;
        let y_real = y_val * y_user_unit;
        let cx_real = cx_val * cx_user_unit;
        let cy_real = cy_val * cy_user_unit;

        // Translate to origin relative to center
        let dx = x_real - cx_real;
        let dy = y_real - cy_real;

        // Apply rotation transformation
        let rotated_dx = dx.mul_add(cos_a, -(dy * sin_a));
        let rotated_dy = dx.mul_add(sin_a, dy * cos_a);

        // Translate back
        let new_x_real = rotated_dx + cx_real;
        let new_y_real = rotated_dy + cy_real;

        Self {
            x: Unit::float(new_x_real, 1.0, x_db_unit),
            y: Unit::float(new_y_real, 1.0, y_db_unit),
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            x: Unit::default_integer(0),
            y: Unit::default_integer(0),
        }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

impl Transformable for Point {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        transformation.apply_to_point(self)
    }
}

impl Movable for Point {
    fn move_to(&self, target: Point) -> Self {
        target
    }
}

impl From<&Self> for Point {
    fn from(point: &Self) -> Self {
        *point
    }
}

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

impl From<[u32; 2]> for Point {
    fn from(arr: [u32; 2]) -> Self {
        Self {
            x: Unit::default_integer(arr[0] as i32),
            y: Unit::default_integer(arr[1] as i32),
        }
    }
}

impl From<(u32, u32)> for Point {
    fn from(tuple: (u32, u32)) -> Self {
        Self {
            x: Unit::default_integer(tuple.0 as i32),
            y: Unit::default_integer(tuple.1 as i32),
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

#[allow(clippy::fallible_impl_from)]
impl From<[Unit; 2]> for Point {
    fn from(arr: [Unit; 2]) -> Self {
        Self::new(arr[0], arr[1]).unwrap()
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<(Unit, Unit)> for Point {
    fn from(tuple: (Unit, Unit)) -> Self {
        Self::new(tuple.0, tuple.1).unwrap()
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<u32> for Point {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<i32> for Point {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<u32> for Point {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<f64> for Point {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
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
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9)).unwrap();
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn with_floats() {
            let point =
                Point::new(Unit::float(1.5, 1e-6, 1e-9), Unit::float(2.5, 1e-6, 1e-9)).unwrap();
            assert_eq!(point.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn with_mixed_units() {
            assert!(Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6, 1e-9)).is_err());
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
        fn from_unit_tuple() {
            let point = Point::from((Unit::integer(100, 1e-9), Unit::float(2.5, 1e-9, 1e-9)));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-9, 1e-9));
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
        fn chaining_from_and_methods() {
            let point = Point::from([100, 200]);
            let rotated = point.rotate(std::f64::consts::PI);

            // Original point should be unchanged
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            // Rotated point should be approximately (-100, -200) in real units
            let x_real = match rotated.x().to_float_unit(1e-6) {
                Unit::Float {
                    value,
                    user_units: user_unit,
                    ..
                } => value * user_unit,
                Unit::Integer { .. } => unreachable!(),
            };
            let y_real = match rotated.y().to_float_unit(1e-6) {
                Unit::Float {
                    value,
                    user_units: user_unit,
                    ..
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
                Point::new(point.x() * 2, point.y() * 2).unwrap()
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
            let mut point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9)).unwrap();

            // Test getters
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            // Test setters
            point.set_x(Unit::integer(300, 1e-9));
            point.set_y(Unit::integer(400, 1e-9));

            assert_eq!(point.x(), Unit::integer(300, 1e-9));
            assert_eq!(point.y(), Unit::integer(400, 1e-9));
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn to_integer_unit_from_integers() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9)).unwrap();
            let converted = point.to_integer_unit();

            assert_eq!(converted.x(), Unit::integer(100, 1e-9));
            assert_eq!(converted.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn to_integer_unit_from_floats() {
            let point =
                Point::new(Unit::float(1.007, 1.0, 1e-3), Unit::float(2.015, 1.0, 1e-3)).unwrap();
            let converted = point.to_integer_unit();

            match converted.x() {
                Unit::Integer {
                    value,
                    db_units: db_unit,
                } => {
                    assert_eq!(value, 1007);
                    assert_eq!(db_unit, 1e-3);
                }
                Unit::Float { .. } => panic!("Expected Integer variant"),
            }

            match converted.y() {
                Unit::Integer {
                    value,
                    db_units: db_unit,
                } => {
                    assert_eq!(value, 2015);
                    assert_eq!(db_unit, 1e-3);
                }
                Unit::Float { .. } => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_float_unit_from_floats() {
            let point =
                Point::new(Unit::float(1.5, 1e-6, 1e-9), Unit::float(2.5, 1e-6, 1e-9)).unwrap();
            let converted = point.to_float_unit(1e-6);

            assert_eq!(converted.x(), Unit::float(1.5, 1e-6, 1e-9));
            assert_eq!(converted.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn to_float_unit_from_integers() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9)).unwrap();
            let converted = point.to_float_unit(1e-6);

            match converted.x() {
                Unit::Float {
                    value,
                    user_units: user_unit,
                    db_units: db_unit,
                } => {
                    assert_eq!(value, 0.1);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                Unit::Integer { .. } => panic!("Expected Float variant"),
            }

            match converted.y() {
                Unit::Float {
                    value,
                    user_units: user_unit,
                    db_units: db_unit,
                } => {
                    assert_eq!(value, 0.2);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                Unit::Integer { .. } => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn roundtrip_integer_to_float_to_integer() {
            let original = Point::new(Unit::integer(100, 1e-9), Unit::integer(200, 1e-9)).unwrap();
            let as_float = original.to_float_unit(1e-6);
            let back_to_int = as_float.to_integer_unit();

            assert_eq!(back_to_int.x(), original.x());
            assert_eq!(back_to_int.y(), original.y());
        }

        #[test]
        fn conversion_preserves_equality() {
            let point1 = Point::new(Unit::integer(1000, 1e-9), Unit::integer(2000, 1e-9)).unwrap();
            let point2 = point1.to_float_unit(1e-6);

            assert_eq!(point1.x(), point2.x());
            assert_eq!(point1.y(), point2.y());
        }
    }

    mod rotation {
        use std::f64::consts::PI;

        use super::*;

        fn extract_real_value(unit: Unit) -> f64 {
            match unit.to_float_unit(1e-6) {
                Unit::Float {
                    value,
                    user_units: user_unit,
                    ..
                } => value * user_unit,
                Unit::Integer { .. } => unreachable!(),
            }
        }

        #[test]
        fn rotate_90_degrees() {
            let point =
                Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9)).unwrap();
            let rotated = point.rotate(PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 1e-6).abs() < 1e-15);
        }

        #[test]
        fn rotate_180_degrees() {
            let point =
                Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9)).unwrap();
            let rotated = point.rotate(PI);

            assert!((extract_real_value(rotated.x()) - (-1e-6)).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_270_degrees() {
            let point =
                Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9)).unwrap();
            let rotated = point.rotate(3.0 * PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - (-1e-6)).abs() < 1e-15);
        }

        #[test]
        fn rotate_360_degrees() {
            let point =
                Point::new(Unit::float(1.0, 1e-6, 1e-9), Unit::float(0.0, 1e-6, 1e-9)).unwrap();
            let rotated = point.rotate(2.0 * PI);

            assert!((extract_real_value(rotated.x()) - 1e-6).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_arbitrary_point() {
            let point =
                Point::new(Unit::float(3.0, 1e-6, 1e-9), Unit::float(4.0, 1e-6, 1e-9)).unwrap();
            let rotated = point.rotate(PI / 4.0); // 45 degrees

            let expected_x = 3e-6f64.mul_add((PI / 4.0).cos(), -(4e-6 * (PI / 4.0).sin()));
            let expected_y = 3e-6f64.mul_add((PI / 4.0).sin(), 4e-6 * (PI / 4.0).cos());

            assert!((extract_real_value(rotated.x()) - expected_x).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - expected_y).abs() < 1e-15);
        }
    }

    mod arithmetic {
        use super::*;

        #[test]
        fn add_points_with_integers() {
            let p1 = Point::from([100, 200]);
            let p2 = Point::from([50, 75]);
            let result = p1 + p2;

            assert_eq!(result.x(), Unit::integer(150, 1e-9));
            assert_eq!(result.y(), Unit::integer(275, 1e-9));
        }

        #[test]
        fn add_points_with_floats() {
            let p1 = Point::from([1.5, 2.5]);
            let p2 = Point::from([0.5, 1.0]);
            let result = p1 + p2;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6, 1e-9));
            assert_eq!(result.y(), Unit::float(3.5, 1e-6, 1e-9));
        }

        #[test]
        fn subtract_points_with_integers() {
            let p1 = Point::from([100, 200]);
            let p2 = Point::from([50, 75]);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::integer(50, 1e-9));
            assert_eq!(result.y(), Unit::integer(125, 1e-9));
        }

        #[test]
        fn subtract_points_with_floats() {
            let p1 = Point::from([2.5, 3.5]);
            let p2 = Point::from([0.5, 1.0]);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6, 1e-9));
            assert_eq!(result.y(), Unit::float(2.5, 1e-6, 1e-9));
        }

        #[test]
        fn multiply_point_by_i32() {
            let point = Point::from([10, 20]);
            let result: Point = point * 3;

            assert_eq!(result.x(), Unit::integer(30, 1e-9));
            assert_eq!(result.y(), Unit::integer(60, 1e-9));
        }

        #[test]
        fn multiply_point_by_f64() {
            let point = Point::from([1.0, 2.0]);
            let result = point * 2.5;

            assert_eq!(result.x(), Unit::float(2.5, 1e-6, 1e-9));
            assert_eq!(result.y(), Unit::float(5.0, 1e-6, 1e-9));
        }

        #[test]
        fn divide_point_by_i32() {
            let point = Point::from([30, 60]);
            let result: Point = point / 3;

            assert_eq!(result.x(), Unit::integer(10, 1e-9));
            assert_eq!(result.y(), Unit::integer(20, 1e-9));
        }

        #[test]
        fn divide_point_by_f64() {
            let point = Point::from([5.0, 10.0]);
            let result = point / 2.5;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6, 1e-9));
            assert_eq!(result.y(), Unit::float(4.0, 1e-6, 1e-9));
        }

        #[test]
        fn chained_arithmetic_operations() {
            let p1 = Point::from([10, 20]);
            let p2 = Point::from([5, 10]);
            let result: Point = (p1 + p2) * 2 - Point::from([10, 20]);

            assert_eq!(result.x(), Unit::integer(20, 1e-9));
            assert_eq!(result.y(), Unit::integer(40, 1e-9));
        }

        #[test]
        fn negative_results() {
            let p1 = Point::from([10, 20]);
            let p2 = Point::from([30, 50]);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::integer(-20, 1e-9));
            assert_eq!(result.y(), Unit::integer(-30, 1e-9));
        }

        #[test]
        fn multiply_by_negative_scalar() {
            let point = Point::from([10, 20]);
            let result: Point = point * -2;

            assert_eq!(result.x(), Unit::integer(-20, 1e-9));
            assert_eq!(result.y(), Unit::integer(-40, 1e-9));
        }

        #[test]
        fn divide_by_negative_scalar() {
            let point = Point::from([10.0, 20.0]);
            let result = point / -2.0;

            assert_eq!(result.x(), Unit::float(-5.0, 1e-6, 1e-9));
            assert_eq!(result.y(), Unit::float(-10.0, 1e-6, 1e-9));
        }
    }
}
