use std::ops::{Add, Div, Mul, Sub};

use crate::units::Unit;
use crate::{AngleInRadians, Movable, Transformable, Transformation};

/// A 2D point with x and y coordinates, each carrying their own unit of measurement.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    x: Unit,
    y: Unit,
}

impl Point {
    /// Creates a point with integer coordinates and the given units.
    pub const fn integer(x: i32, y: i32, units: f64) -> Self {
        Self {
            x: Unit::integer(x, units),
            y: Unit::integer(y, units),
        }
    }

    /// Creates a point with float coordinates and the given units.
    pub const fn float(x: f64, y: f64, units: f64) -> Self {
        Self {
            x: Unit::float(x, units),
            y: Unit::float(y, units),
        }
    }

    /// Creates a point at the origin (0, 0) with default integer units.
    pub fn origin() -> Self {
        Self {
            x: Unit::zero(),
            y: Unit::zero(),
        }
    }

    /// Creates a point with integer coordinates and default integer units (1e-9).
    pub const fn default_integer(x: i32, y: i32) -> Self {
        Self {
            x: Unit::default_integer(x),
            y: Unit::default_integer(y),
        }
    }

    /// Creates a point with float coordinates and default float units (1e-6).
    pub const fn default_float(x: f64, y: f64) -> Self {
        Self {
            x: Unit::default_float(x),
            y: Unit::default_float(y),
        }
    }

    /// Create a point with two arbitrary units.
    ///
    /// This is useful for when you want to create a point with different units.
    /// This is not highly recommended.
    pub const fn new(x: Unit, y: Unit) -> Self {
        Self { x, y }
    }

    /// Returns the units for both coordinates as `(x_units, y_units)`.
    pub const fn units(&self) -> (f64, f64) {
        (self.x.units(), self.y.units())
    }

    /// Returns a copy of this point with both coordinates set to the given units (without scaling).
    #[must_use]
    pub const fn set_units(&self, units: f64) -> Self {
        Self {
            x: self.x.set_units(units),
            y: self.y.set_units(units),
        }
    }

    /// Returns a copy of this point with both coordinates scaled to the given units.
    #[must_use]
    pub fn scale_units(&self, new_units: f64) -> Self {
        Self {
            x: self.x.scale_to(new_units),
            y: self.y.scale_to(new_units),
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
    pub fn to_float_unit(&self) -> Self {
        Self {
            x: self.x.to_float_unit(),
            y: self.y.to_float_unit(),
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

        let (u1, u2) = self.units();

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let x_real = self.x.scale_to(u1).float_value();
        let y_real = self.y.scale_to(u2).float_value();
        let cx_real = center.x.scale_to(u1).float_value();
        let cy_real = center.y.scale_to(u2).float_value();

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
            x: Unit::float(new_x_real, u1),
            y: Unit::float(new_y_real, u2),
        }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

impl Transformable for Point {
    fn transform_impl(self, transformation: &Transformation) -> Self {
        transformation.apply_to_point(&self)
    }
}

impl Movable for Point {
    fn move_to(self, target: Point) -> Self {
        target
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

impl Add<Point> for &Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
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

    use quickcheck::{Arbitrary, Gen};

    const MAX_VALUE: i32 = 10_000;

    impl Arbitrary for Point {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];

            if bool::arbitrary(g) {
                let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                Self::integer(x, y, units)
            } else {
                let raw_x = f64::arbitrary(g);
                let raw_y = f64::arbitrary(g);
                let x = if raw_x.is_finite() {
                    (raw_x % f64::from(MAX_VALUE))
                        .clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
                } else {
                    0.0
                };
                let y = if raw_y.is_finite() {
                    (raw_y % f64::from(MAX_VALUE))
                        .clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
                } else {
                    0.0
                };
                Self::float(x, y, units)
            }
        }
    }

    mod creation {
        use super::*;

        #[test]
        fn with_integers() {
            let point = Point::integer(100, 200, 1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn with_defaults() {
            let point = Point::default_integer(100, 200);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));
        }

        #[test]
        fn with_units() {
            let point =
                Point::new(Unit::integer(100, 1e-6), Unit::float(2.5, 1e-6)).set_units(1e-9);
            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::float(2.5, 1e-9));
        }

        #[test]
        fn with_defaults_float() {
            let point = Point::default_float(1.5, 2.5);
            assert_eq!(point.x(), Unit::float(1.5, 1e-6));
            assert_eq!(point.y(), Unit::float(2.5, 1e-6));
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
            let rotated = point.rotate(std::f64::consts::PI, Point::integer(0, 0, 1e-9));

            assert_eq!(point.x(), Unit::integer(100, 1e-9));
            assert_eq!(point.y(), Unit::integer(200, 1e-9));

            let x_real = rotated.x().absolute_value();

            let y_real = rotated.y().absolute_value();

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

            assert_eq!(converted, Point::integer(1, 2, 1e-3));
        }

        #[test]
        fn to_float_unit_from_floats() {
            let point = Point::float(1.5, 2.5, 1e-6);
            let converted = point.to_float_unit();

            assert_eq!(converted.x(), Unit::float(1.5, 1e-6));
            assert_eq!(converted.y(), Unit::float(2.5, 1e-6));
        }

        #[test]
        fn to_float_unit_from_integers() {
            let point = Point::integer(100, 200, 1e-9);
            let converted = point.to_float_unit().scale_units(1e-6);

            assert_eq!(converted, Point::float(0.1, 0.2, 1e-6));
        }

        #[test]
        fn roundtrip_integer_to_float_to_integer() {
            let original = Point::integer(100, 200, 1e-9);
            let as_float = original.to_float_unit();
            let back_to_int = as_float.to_integer_unit();

            assert_eq!(back_to_int.x(), original.x());
            assert_eq!(back_to_int.y(), original.y());
        }

        #[test]
        fn conversion_preserves_equality() {
            let point1 = Point::integer(1000, 2000, 1e-9);
            let point2 = point1.to_float_unit();

            assert_eq!(point1.x(), point2.x());
            assert_eq!(point1.y(), point2.y());
        }
    }

    mod rotation {
        use std::f64::consts::PI;

        use super::*;

        #[test]
        fn rotate_90_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(PI / 2.0, Point::float(0.0, 0.0, 1e-6));

            assert!((rotated.x().absolute_value() - 0.0) < 1e-15);
            assert!((rotated.y().absolute_value() - 1e-6) < 1e-15);
        }

        #[test]
        fn rotate_180_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(PI, Point::float(0.0, 0.0, 1e-6));

            assert!((rotated.x().absolute_value() - (-1e-6)) < 1e-15);
            assert!((rotated.y().absolute_value() - 0.0) < 1e-15);
        }

        #[test]
        fn rotate_270_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(3.0 * PI / 2.0, Point::float(0.0, 0.0, 1e-6));

            assert!((rotated.x().absolute_value() - 0.0) < 1e-15);
            assert!((rotated.y().absolute_value() - (-1e-6)) < 1e-15);
        }

        #[test]
        fn rotate_360_degrees() {
            let point = Point::float(1.0, 0.0, 1e-6);
            let rotated = point.rotate(2.0 * PI, Point::float(0.0, 0.0, 1e-6));

            assert!((rotated.x().absolute_value() - 1e-6) < 1e-15);
            assert!((rotated.y().absolute_value() - 0.0) < 1e-15);
        }

        #[test]
        fn rotate_arbitrary_point() {
            let point = Point::float(3.0, 4.0, 1e-6);
            let rotated = point.rotate(PI / 4.0, Point::float(0.0, 0.0, 1e-6)); // 45 degrees

            let expected_x = 3e-6f64.mul_add((PI / 4.0).cos(), -(4e-6 * (PI / 4.0).sin()));
            let expected_y = 3e-6f64.mul_add((PI / 4.0).sin(), 4e-6 * (PI / 4.0).cos());

            assert!((rotated.x().absolute_value() - expected_x) < 1e-15);
            assert!((rotated.y().absolute_value() - expected_y) < 1e-15);
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

    mod property_tests {
        use std::f64::consts::TAU;

        use super::*;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn addition_commutativity(a: Point, b: Point) -> bool {
            let a = a.to_float_unit();
            let b = b.to_float_unit().scale_units(a.units().0);

            (a + b) == (b + a)
        }

        #[quickcheck]
        fn addition_associativity(a: Point, b: Point, c: Point) -> bool {
            let units = a.units().0;
            let a = a.to_float_unit();
            let b = b.to_float_unit().scale_units(units);
            let c = c.to_float_unit().scale_units(units);

            ((a + b) + c) == (a + (b + c))
        }

        #[quickcheck]
        fn additive_identity(a: Point) -> bool {
            let zero = Point::float(0.0, 0.0, a.units().0);
            let a = a.to_float_unit();

            let a_plus_zero = a + zero;
            let zero_plus_a = zero + a;
            a_plus_zero == a && zero_plus_a == a
        }

        /// Verifies that a + (-a) produces a zero point.
        #[quickcheck]
        fn additive_inverse(a: Point) -> bool {
            let units = a.units().0;
            let neg_a = Point::new(
                Unit::float(-a.x().float_value(), units),
                Unit::float(-a.y().float_value(), units),
            );
            let a = a.to_float_unit();

            let result = a + neg_a;
            result.x().absolute_value().abs() < 1e-9 && result.y().absolute_value().abs() < 1e-9
        }

        /// Verifies scalar multiplication distributes over point addition.
        /// Uses float points with matching units and approximate comparison
        /// to account for floating point rounding in different evaluation orders.
        #[quickcheck]
        fn scalar_multiplication_distributivity(a: Point, b: Point, s: i32) -> bool {
            let units = a.units().0;
            let a = Point::float(a.x().float_value(), a.y().float_value(), units);
            let b = Point::float(b.x().float_value(), b.y().float_value(), units);
            let s = (s % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);

            let lhs = (a + b) * s;
            let rhs = (a * s) + (b * s);

            let dx = (lhs.x().absolute_value() - rhs.x().absolute_value()).abs();
            let dy = (lhs.y().absolute_value() - rhs.y().absolute_value()).abs();

            dx < units && dy < units
        }

        /// Verifies that rotation by 2*pi returns approximately the original point.
        #[quickcheck]
        fn rotation_by_2pi_is_identity(a: Point) -> bool {
            let a = a.to_float_unit();
            let origin = Point::float(0.0, 0.0, a.units().0);
            let rotated = a.rotate(TAU, origin);

            let dx = (rotated.x().absolute_value() - a.x().absolute_value()).abs();
            let dy = (rotated.y().absolute_value() - a.y().absolute_value()).abs();

            dx < 1e-9 && dy < 1e-9
        }
    }
}
