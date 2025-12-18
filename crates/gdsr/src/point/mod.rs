use std::ops::{Add, Div, Mul, Sub};

use crate::{AngleInRadians, Movable, Transformable, Transformation, units::Unit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: Unit,
    y: Unit,
}

impl Point {
    pub const fn integer(x: i32, y: i32, units: f64) -> Self {
        Self {
            x: Unit::integer(x, units),
            y: Unit::integer(y, units),
        }
    }

    pub const fn float(x: f64, y: f64, units: f64) -> Self {
        Self {
            x: Unit::float(x, units),
            y: Unit::float(y, units),
        }
    }

    /// Create a point with two arbitrary units.
    ///
    /// This is useful for when you want to create a point with different units.
    /// This is not highly recommended.
    pub const fn new(x: Unit, y: Unit) -> Self {
        Self { x, y }
    }

    pub const fn units(&self) -> (f64, f64) {
        (self.x.units(), self.y.units())
    }

    #[must_use]
    pub const fn with_units(&self, units: f64) -> Self {
        Self {
            x: self.x.with_units(units),
            y: self.y.with_units(units),
        }
    }

    /// Gets the x coordinate of the point.
    pub const fn x(&self) -> Unit {
        self.x
    }

    /// Gets the y coordinate of the point.
    pub const fn y(&self) -> Unit {
        self.y
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
    pub fn rotate(&self, angle: AngleInRadians) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Convert to float units and extract values
        let x_float = self.x.to_float_unit(1e-6);
        let y_float = self.y.to_float_unit(1e-6);

        let Unit::Float {
            value: x_val,
            units: x_units,
        } = x_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: y_val,
            units: y_units,
        } = y_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        // Calculate real world values
        let x_real = x_val * x_units;
        let y_real = y_val * y_units;

        // Apply rotation transformation
        let new_x_real = x_real.mul_add(cos_a, -(y_real * sin_a));
        let new_y_real = x_real.mul_add(sin_a, y_real * cos_a);

        Self {
            x: Unit::float(new_x_real, 1.0),
            y: Unit::float(new_y_real, 1.0),
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
    pub fn rotate_around_point(&self, angle: AngleInRadians, center: &Self) -> Self {
        if angle == 0.0 {
            return *self;
        }

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Convert to float units and extract values
        let x_float = self.x.to_float_unit(1e-6);
        let y_float = self.y.to_float_unit(1e-6);
        let cx_float = center.x.to_float_unit(1e-6);
        let cy_float = center.y.to_float_unit(1e-6);

        let Unit::Float {
            value: x_val,
            units: x_units,
        } = x_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: y_val,
            units: y_units,
        } = y_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: cx_val,
            units: cx_units,
        } = cx_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        let Unit::Float {
            value: cy_val,
            units: cy_units,
        } = cy_float
        else {
            unreachable!("to_float_unit should always return Float variant");
        };

        // Calculate real world values
        let x_real = x_val * x_units;
        let y_real = y_val * y_units;
        let cx_real = cx_val * cx_units;
        let cy_real = cy_val * cy_units;

        // Translate to origin relative to center
        let dx = x_real - cx_real;
        let dy = y_real - cy_real;

        // Apply rotation transformation
        let rotated_dx = dx.mul_add(cos_a, -(dy * sin_a));
        let rotated_dy = dx.mul_add(sin_a, dy * cos_a);

        // Translate back
        let new_x_real = rotated_dx + cx_real;
        let new_y_real = rotated_dy + cy_real;

        let (u1, u2) = self.units();

        Self {
            x: Unit::float(new_x_real, u1),
            y: Unit::float(new_y_real, u2),
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

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Self> for Point {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Point> for &Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Point> for &Point {
    type Output = Point;

    fn add(self, rhs: &Point) -> Self::Output {
        Point {
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
            let point = Point::integer(100, 200, 1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn with_floats() {
            let point = Point::float(1.5, 2.5, 1e-6);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn with_mixed_units() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-6));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn from_i32_array() {
            let point = Point::integer(100, 200, 1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn from_i32_tuple() {
            let point = Point::integer(100, 200, 1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn from_f64_array() {
            let point = Point::float(1.5, 2.5, 1e-6);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn from_f64_tuple() {
            let point = Point::float(1.5, 2.5, 1e-6);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn from_unit_tuple() {
            let point = Point::new(Unit::integer(100, 1e-9), Unit::float(2.5, 1e-9));
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-9));
        }

        #[test]
        fn into_works_with_type_inference() {
            let point = Point::integer(100, 200, 1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            let point = Point::float(1.5, 2.5, 1e-6);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn chaining_from_and_methods() {
            let point = Point::integer(100, 200, 1e-9);
            let rotated = point.rotate(std::f64::consts::PI);

            // Original point should be unchanged
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            // Rotated point should be approximately (-100, -200) in real units
            let x_real = match rotated.x().to_float_unit(1e-6) {
                Unit::Float { value, units } => value * units,
                Unit::Integer { .. } => unreachable!(),
            };
            let y_real = match rotated.y().to_float_unit(1e-6) {
                Unit::Float { value, units } => value * units,
                Unit::Integer { .. } => unreachable!(),
            };

            assert!((x_real - (-100e-9)).abs() < 1e-15);
            assert!((y_real - (-200e-9)).abs() < 1e-15);
        }

        #[test]
        fn getter_and_setter() {
            let mut point = Point::integer(100, 200, 1e-9);

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
        use approx::assert_relative_eq;

        use super::*;

        #[test]
        fn to_integer_unit_from_integers() {
            let point = Point::integer(100, 200, 1e-9);
            let converted = point.to_integer_unit();

            assert_eq!(converted.x(), Unit::integer(100, 1e-9));
            assert_eq!(converted.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn to_integer_unit_from_floats() {
            let point = Point::float(1.007, 2.015, 1e-3);
            let converted = point.to_integer_unit();

            match converted.x() {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 1);
                    assert_eq!(units, 1e-3);
                }
                Unit::Float { .. } => panic!("Expected Integer variant"),
            }

            match converted.y() {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 2);
                    assert_eq!(units, 1e-3);
                }
                Unit::Float { .. } => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_float_unit_from_floats() {
            let point = Point::float(1.5, 2.5, 1e-6);
            let converted = point.to_float_unit(1e-6);

            assert_eq!(converted.x(), Unit::float(1.5, 1e-6));
            assert_eq!(converted.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn to_float_unit_from_integers() {
            let point = Point::integer(100, 200, 1e-9);
            let converted = point.to_float_unit(1e-6);

            match converted.x() {
                Unit::Float { value, units } => {
                    assert_relative_eq!(value, 0.1);
                    assert_eq!(units, 1e-6);
                }
                Unit::Integer { .. } => panic!("Expected Float variant"),
            }

            match converted.y() {
                Unit::Float { value, units } => {
                    assert_relative_eq!(value, 0.2);
                    assert_eq!(units, 1e-6);
                }
                Unit::Integer { .. } => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn roundtrip_integer_to_float_to_integer() {
            let original = Point::integer(100, 200, 1e-9);
            let as_float = original.to_float_unit(1e-9);
            let back_to_int = as_float.to_integer_unit();

            assert_eq!(back_to_int.x(), original.x());
            assert_eq!(back_to_int.y(), original.y());
        }

        #[test]
        fn conversion_preserves_equality() {
            let point1 = Point::integer(1000, 2000, 1e-9);
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
                Unit::Float { value, units } => value * units,
                Unit::Integer { .. } => unreachable!(),
            }
        }

        #[test]
        fn rotate_90_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 1e-6).abs() < 1e-15);
        }

        #[test]
        fn rotate_180_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(PI);

            assert!((extract_real_value(rotated.x()) - (-1e-6)).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_270_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(3.0 * PI / 2.0);

            assert!((extract_real_value(rotated.x()) - 0.0).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - (-1e-6)).abs() < 1e-15);
        }

        #[test]
        fn rotate_360_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(2.0 * PI);

            assert!((extract_real_value(rotated.x()) - 1e-6).abs() < 1e-15);
            assert!((extract_real_value(rotated.y()) - 0.0).abs() < 1e-15);
        }

        #[test]
        fn rotate_arbitrary_point() {
            let point = Point::float(3.0, 4.0, 1e-6);
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
            let p1 = Point::integer(100, 200, 1e-9);
            let p2 = Point::integer(50, 75, 1e-9);
            let result = p1 + p2;

            assert_eq!(result.x(), Unit::integer(150, 1e-9));
            assert_eq!(result.y(), Unit::integer(275, 1e-9));
        }

        #[test]
        fn add_points_with_floats() {
            let p1 = Point::float(1.5, 2.5, 1e-6);
            let p2 = Point::float(0.5, 1.0, 1e-6);
            let result = p1 + p2;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6));
            assert_eq!(result.y(), Unit::float(3.5, 1e-6));
        }

        #[test]
        fn subtract_points_with_integers() {
            let p1 = Point::integer(100, 200, 1e-9);
            let p2 = Point::integer(50, 75, 1e-9);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::integer(50, 1e-9));
            assert_eq!(result.y(), Unit::integer(125, 1e-9));
        }

        #[test]
        fn subtract_points_with_floats() {
            let p1 = Point::float(2.5, 3.5, 1e-6);
            let p2 = Point::float(0.5, 1.0, 1e-6);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6));
            assert_eq!(result.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn multiply_point_by_i32() {
            let point = Point::integer(10, 20, 1e-9);
            let result: Point = point * 3;

            assert_eq!(result.x(), Unit::integer(30, 1e-9));
            assert_eq!(result.y(), Unit::integer(60, 1e-9));
        }

        #[test]
        fn multiply_point_by_f64() {
            let point = Point::float(1.0, 2.0, 1e-6);
            let result = point * 2.5;

            assert_eq!(result.x(), Unit::float(2.5, 1e-6));
            assert_eq!(result.y(), Unit::float(5.0, 1e-6));
        }

        #[test]
        fn divide_point_by_i32() {
            let point = Point::integer(30, 60, 1e-9);
            let result: Point = point / 3;

            assert_eq!(result.x(), Unit::integer(10, 1e-9));
            assert_eq!(result.y(), Unit::integer(20, 1e-9));
        }

        #[test]
        fn divide_point_by_f64() {
            let point = Point::float(5.0, 10.0, 1e-6);
            let result = point / 2.5;

            assert_eq!(result.x(), Unit::float(2.0, 1e-6));
            assert_eq!(result.y(), Unit::float(4.0, 1e-6));
        }

        #[test]
        fn chained_arithmetic_operations() {
            let p1 = Point::integer(10, 20, 1e-9);
            let p2 = Point::integer(5, 10, 1e-9);
            let result: Point = (p1 + p2) * 2 - Point::integer(10, 20, 1e-9);

            assert_eq!(result.x(), Unit::integer(20, 1e-9));
            assert_eq!(result.y(), Unit::integer(40, 1e-9));
        }

        #[test]
        fn negative_results() {
            let p1 = Point::integer(10, 20, 1e-9);
            let p2 = Point::integer(30, 50, 1e-9);
            let result = p1 - p2;

            assert_eq!(result.x(), Unit::integer(-20, 1e-9));
            assert_eq!(result.y(), Unit::integer(-30, 1e-9));
        }

        #[test]
        fn multiply_by_negative_scalar() {
            let point = Point::integer(10, 20, 1e-9);
            let result: Point = point * -2;

            assert_eq!(result.x(), Unit::integer(-20, 1e-9));
            assert_eq!(result.y(), Unit::integer(-40, 1e-9));
        }

        #[test]
        fn divide_by_negative_scalar() {
            let point = Point::float(10.0, 20.0, 1e-6);
            let result = point / -2.0;

            assert_eq!(result.x(), Unit::float(-5.0, 1e-6));
            assert_eq!(result.y(), Unit::float(-10.0, 1e-6));
        }
    }
}
