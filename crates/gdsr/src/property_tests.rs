use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

use crate::*;

mod arbitrary {
    use super::*;

    const MAX_VALUE: i32 = 10_000;
    const MAX_ANGLE: f64 = std::f64::consts::TAU;
    const MAX_SCALE: f64 = 100.0;
    const MAX_COORD: i32 = 10_000;
    const MIN_POLYGON_VERTICES: usize = 3;
    const MAX_EXTRA_VERTICES: usize = 20;
    const MIN_PATH_POINTS: usize = 2;
    const MAX_EXTRA_POINTS: usize = 18;

    impl Arbitrary for units::IntegerUnit {
        fn arbitrary(g: &mut Gen) -> Self {
            let value = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            Self { value, units }
        }
    }

    impl Arbitrary for units::FloatUnit {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_value = f64::arbitrary(g);
            let value = if raw_value.is_finite() {
                (raw_value % f64::from(MAX_VALUE))
                    .clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
            } else {
                0.0
            };
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            Self { value, units }
        }
    }

    impl Arbitrary for Unit {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                Self::Integer(units::IntegerUnit::arbitrary(g))
            } else {
                Self::Float(units::FloatUnit::arbitrary(g))
            }
        }
    }

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

    impl Arbitrary for Reflection {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_angle = f64::arbitrary(g);
            let angle = if raw_angle.is_finite() {
                raw_angle % MAX_ANGLE
            } else {
                0.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(angle, centre)
        }
    }

    impl Arbitrary for Rotation {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_angle = f64::arbitrary(g);
            let angle = if raw_angle.is_finite() {
                raw_angle % MAX_ANGLE
            } else {
                0.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(angle, centre)
        }
    }

    impl Arbitrary for Scale {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_factor = f64::arbitrary(g);
            let factor = if raw_factor.is_finite() {
                (raw_factor % MAX_SCALE).clamp(-MAX_SCALE, MAX_SCALE)
            } else {
                1.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(factor, centre)
        }
    }

    impl Arbitrary for Translation {
        fn arbitrary(g: &mut Gen) -> Self {
            let delta = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(delta)
        }
    }

    impl Arbitrary for Transformation {
        fn arbitrary(g: &mut Gen) -> Self {
            let reflection = if bool::arbitrary(g) {
                Some(Reflection::arbitrary(g))
            } else {
                None
            };
            let rotation = if bool::arbitrary(g) {
                Some(Rotation::arbitrary(g))
            } else {
                None
            };
            let scale = if bool::arbitrary(g) {
                Some(Scale::arbitrary(g))
            } else {
                None
            };
            let translation = if bool::arbitrary(g) {
                Some(Translation::arbitrary(g))
            } else {
                None
            };
            let mut t = Self::default();
            t.with_reflection(reflection);
            t.with_rotation(rotation);
            t.with_scale(scale);
            t.with_translation(translation);
            t
        }
    }

    impl Arbitrary for Polygon {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            let num_vertices =
                MIN_POLYGON_VERTICES + (usize::arbitrary(g) % (MAX_EXTRA_VERTICES + 1));
            let points: Vec<Point> = (0..num_vertices)
                .map(|_| {
                    let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    Point::integer(x, y, units)
                })
                .collect();
            let layer = u16::arbitrary(g);
            let data_type = u16::arbitrary(g);
            Self::new(points, layer, data_type)
        }
    }

    impl Arbitrary for PathType {
        fn arbitrary(g: &mut Gen) -> Self {
            let types = [Self::Square, Self::Round, Self::Overlap];
            types[usize::arbitrary(g) % types.len()]
        }
    }

    impl Arbitrary for Path {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            let num_points = MIN_PATH_POINTS + (usize::arbitrary(g) % (MAX_EXTRA_POINTS + 1));
            let points: Vec<Point> = (0..num_points)
                .map(|_| {
                    let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    Point::integer(x, y, units)
                })
                .collect();
            let layer = u16::arbitrary(g);
            let data_type = u16::arbitrary(g);
            let path_type = if bool::arbitrary(g) {
                Some(PathType::arbitrary(g))
            } else {
                None
            };
            let width = if bool::arbitrary(g) {
                let w = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
                Some(Unit::integer(w, units))
            } else {
                None
            };
            Self::new(points, layer, data_type, path_type, width)
        }
    }

    impl Arbitrary for Text {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            let origin = Point::integer(x, y, units);
            let len = 1 + (usize::arbitrary(g) % 20);
            let value: String = (0..len)
                .map(|_| (b'a' + (u8::arbitrary(g) % 26)) as char)
                .collect();
            let layer = u16::arbitrary(g) % 256;
            let datatype = u16::arbitrary(g) % 256;
            let vp_options = [
                HorizontalPresentation::Left,
                HorizontalPresentation::Centre,
                HorizontalPresentation::Right,
            ];
            let hp_options = [
                VerticalPresentation::Top,
                VerticalPresentation::Middle,
                VerticalPresentation::Bottom,
            ];
            Self::new(
                &value,
                origin,
                layer,
                datatype,
                1.0,
                0.0,
                false,
                hp_options[usize::arbitrary(g) % hp_options.len()],
                vp_options[usize::arbitrary(g) % vp_options.len()],
            )
        }
    }

    impl Arbitrary for Grid {
        fn arbitrary(g: &mut Gen) -> Self {
            let cols = 1 + (u32::arbitrary(g) % 10);
            let rows = 1 + (u32::arbitrary(g) % 10);
            let x = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let sx = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let sy = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            Self::new(
                Point::integer(x, y, 1e-9),
                cols,
                rows,
                Some(Point::integer(sx, 0, 1e-9)),
                Some(Point::integer(0, sy, 1e-9)),
                1.0,
                0.0,
                false,
            )
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
mod unit_tests {
    use super::*;
    use units::{FloatUnit, IntegerUnit};

    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[quickcheck]
    fn float_addition_commutativity(a: FloatUnit, b: FloatUnit) -> bool {
        let a = Unit::Float(a);
        let b = Unit::Float(b);
        (a + b) == (b + a)
    }

    #[quickcheck]
    fn float_addition_associativity(a: FloatUnit, b: FloatUnit, c: FloatUnit) -> bool {
        let a = Unit::Float(a);
        let b = Unit::Float(b);
        let c = Unit::Float(c);
        ((a + b) + c) == (a + (b + c))
    }

    #[quickcheck]
    fn addition_identity(a: Unit) -> bool {
        let zero = match a {
            Unit::Integer(_) => Unit::integer(0, a.units()),
            Unit::Float(_) => Unit::float(0.0, a.units()),
        };

        let a_plus_zero = a + zero;
        let zero_plus_a = zero + a;
        zero_plus_a == a && a_plus_zero == zero_plus_a
    }

    #[quickcheck]
    fn multiplicative_identity(a: Unit) -> bool {
        let one = match a {
            Unit::Integer(_) => Unit::integer(1, a.units()),
            Unit::Float(_) => Unit::float(1.0, a.units()),
        };

        let a_times_one = a * one;
        let one_times_a = one * a;
        one_times_a == a && a_times_one == one_times_a
    }

    #[quickcheck]
    fn addition_preserves_units(a: Unit, b: Unit) -> bool {
        let result = a + b;
        result.units() == a.units()
    }

    #[quickcheck]
    fn subtraction_preserves_units(a: Unit, b: Unit) -> bool {
        let result = a - b;
        result.units() == a.units()
    }

    /// Verifies that the absolute value of the sum equals the sum of absolute values.
    /// Tolerance accounts for unit scaling and integer rounding.
    #[quickcheck]
    fn addition_absolute_value_correctness(a: Unit, b: Unit) -> bool {
        let sum = a + b;
        let abs_sum = sum.absolute_value();
        let expected = a.absolute_value() + b.absolute_value();

        approx_eq(abs_sum, expected, a.units())
    }

    /// Verifies that a + (-a) ≈ 0.
    #[quickcheck]
    fn addition_inverse_property(a: Unit) -> bool {
        let neg_a = match a {
            Unit::Integer(IntegerUnit { value, units }) => Unit::Integer(IntegerUnit {
                value: -value,
                units,
            }),
            Unit::Float(FloatUnit { value, units }) => Unit::Float(FloatUnit {
                value: -value,
                units,
            }),
        };

        let result = a + neg_a;
        result.absolute_value().abs() < 1e-9
    }

    /// Verifies exact integer addition when units are identical.
    #[quickcheck]
    fn addition_same_units_no_scaling_error_integer(a: IntegerUnit, b: IntegerUnit) -> bool {
        let a_unit = Unit::Integer(IntegerUnit {
            value: a.value,
            units: a.units,
        });
        let b_unit = Unit::Integer(IntegerUnit {
            value: b.value,
            units: a.units,
        });

        let result = a_unit + b_unit;

        result.to_integer_unit() == Unit::integer(a.value + b.value, a.units)
    }

    /// Verifies precise float addition when units are identical.
    #[quickcheck]
    fn addition_same_units_no_scaling_error_float(a: FloatUnit, b: FloatUnit) -> bool {
        let a_unit = Unit::Float(FloatUnit {
            value: a.value,
            units: a.units,
        });
        let b_unit = Unit::Float(FloatUnit {
            value: b.value,
            units: a.units,
        });

        let result = a_unit + b_unit;

        result.to_float_unit() == Unit::float(a.value + b.value, a.units)
    }
}

#[allow(clippy::needless_pass_by_value)]
mod point_tests {
    use std::f64::consts::TAU;

    use super::*;

    const MAX_VALUE: i32 = 10_000;

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

#[allow(clippy::needless_pass_by_value)]
mod transformation_tests {
    use super::*;

    const MAX_COORD: i32 = 10_000;

    fn point_approx_eq(a: &Point, b: &Point, epsilon: f64) -> bool {
        (a.x() - b.x()).absolute_value().abs() < epsilon
            && (a.y() - b.y()).absolute_value().abs() < epsilon
    }

    /// Identity transformation does not change a point.
    #[quickcheck]
    fn identity_does_not_change_point(x: i32, y: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let identity = Transformation::default();
        identity.apply_to_point(&point) == point
    }

    /// Double reflection with the same axis cancels out.
    #[quickcheck]
    fn double_reflection_cancels(reflection: Reflection, x: i32, y: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let once = reflection.apply_to_point(&point);
        let twice = reflection.apply_to_point(&once);
        point_approx_eq(&twice, &point, 1e-6)
    }

    /// Rotation by 0 is identity.
    #[quickcheck]
    fn rotation_by_zero_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let centre = Point::integer(cx, cy, 1e-9);
        let rotation = Rotation::new(0.0, centre);
        let result = rotation.apply_to_point(&point);
        point_approx_eq(&result, &point, 1e-6)
    }

    /// Rotation by 2*pi is approximately identity.
    #[quickcheck]
    fn rotation_by_two_pi_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let centre = Point::integer(cx, cy, 1e-9);
        let rotation = Rotation::new(std::f64::consts::TAU, centre);
        let result = rotation.apply_to_point(&point);
        point_approx_eq(&result, &point, 1e-6)
    }

    /// Scale by 1 is identity.
    #[quickcheck]
    fn scale_by_one_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let centre = Point::integer(cx, cy, 1e-9);
        let scale = Scale::new(1.0, centre);
        let result = scale.apply_to_point(&point);
        point_approx_eq(&result, &point, 1e-6)
    }

    /// Translation composition: translate(a) then translate(b) == translate(a + b).
    #[quickcheck]
    fn translation_composition(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> bool {
        let ax = (ax % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let ay = (ay % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let bx = (bx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let by = (by % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let px = (px % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let py = (py % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);

        let point = Point::integer(px, py, 1e-9);
        let t_a = Translation::new(Point::integer(ax, ay, 1e-9));
        let t_b = Translation::new(Point::integer(bx, by, 1e-9));
        let t_ab = Translation::new(Point::integer(ax + bx, ay + by, 1e-9));

        let sequential = t_b.apply_to_point(&t_a.apply_to_point(&point));
        let composed = t_ab.apply_to_point(&point);
        point_approx_eq(&sequential, &composed, 1e-6)
    }

    /// Scale then inverse scale returns to the original point.
    #[quickcheck]
    fn scale_then_inverse_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let centre = Point::integer(cx, cy, 1e-9);

        let factor = 2.5;
        let scale = Scale::new(factor, centre);
        let inverse_scale = Scale::new(1.0 / factor, centre);

        let result = inverse_scale.apply_to_point(&scale.apply_to_point(&point));
        point_approx_eq(&result, &point, 1e-6)
    }

    /// Rotation then inverse rotation returns to the original point.
    #[quickcheck]
    fn rotation_then_inverse_is_identity(rotation: Rotation, x: i32, y: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let inverse = Rotation::new(-rotation.angle(), *rotation.centre());

        let result = inverse.apply_to_point(&rotation.apply_to_point(&point));
        point_approx_eq(&result, &point, 1e-6)
    }

    /// Translation then inverse translation returns to the original point.
    #[quickcheck]
    fn translation_then_inverse_is_identity(translation: Translation, x: i32, y: i32) -> bool {
        let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let point = Point::integer(x, y, 1e-9);
        let applied = translation.apply_to_point(&point);
        let delta = applied - point;
        let inverse = Translation::new(delta * -1);

        let result = inverse.apply_to_point(&applied);
        point_approx_eq(&result, &point, 1e-6)
    }
}

#[allow(clippy::needless_pass_by_value)]
mod polygon_tests {
    use super::*;
    use crate::elements::polygon::close_points;

    #[quickcheck]
    fn area_is_non_negative(polygon: Polygon) -> bool {
        polygon.area().float_value() >= 0.0
    }

    #[quickcheck]
    fn perimeter_is_non_negative(polygon: Polygon) -> bool {
        polygon.perimeter().float_value() >= 0.0
    }

    #[quickcheck]
    fn translation_preserves_area(polygon: Polygon, dx: i32, dy: i32) -> bool {
        let units = polygon.points()[0].units().0;
        let dx = (dx % 10_000).clamp(-10_000, 10_000);
        let dy = (dy % 10_000).clamp(-10_000, 10_000);
        let delta = Point::integer(dx, dy, units);
        let translated = polygon.clone().translate(delta);
        polygon.area() == translated.area()
    }

    #[quickcheck]
    fn translation_preserves_perimeter(polygon: Polygon, dx: i32, dy: i32) -> bool {
        let units = polygon.points()[0].units().0;
        let dx = (dx % 10_000).clamp(-10_000, 10_000);
        let dy = (dy % 10_000).clamp(-10_000, 10_000);
        let delta = Point::integer(dx, dy, units);
        let translated = polygon.clone().translate(delta);
        polygon.perimeter() == translated.perimeter()
    }

    /// Rotation should preserve area (within floating point tolerance).
    #[quickcheck]
    fn rotation_preserves_area(polygon: Polygon) -> bool {
        let units = polygon.points()[0].units().0;
        let centre = Point::integer(0, 0, units);
        let rotated = polygon.clone().rotate(std::f64::consts::FRAC_PI_2, centre);
        let original_area = polygon.area().float_value();
        let rotated_area = rotated.area().float_value();
        if original_area == 0.0 {
            return rotated_area == 0.0;
        }
        ((original_area - rotated_area) / original_area).abs() < 1e-6
    }

    #[quickcheck]
    fn bounding_box_contains_all_points(polygon: Polygon) -> bool {
        let (min, max) = polygon.bounding_box();
        polygon.points().iter().all(|p| {
            p.x().float_value() >= min.x().float_value()
                && p.x().float_value() <= max.x().float_value()
                && p.y().float_value() >= min.y().float_value()
                && p.y().float_value() <= max.y().float_value()
        })
    }

    #[quickcheck]
    fn close_points_is_idempotent(polygon: Polygon) -> bool {
        let once = close_points(polygon.points().to_vec());
        let twice = close_points(once.clone());
        once == twice
    }
}

#[allow(clippy::needless_pass_by_value)]
mod path_tests {
    use super::*;

    #[quickcheck]
    fn bounding_box_contains_all_points(path: Path) -> bool {
        if path.points().is_empty() {
            return true;
        }
        let (min, max) = path.bounding_box();
        path.points().iter().all(|p| {
            p.x().float_value() >= min.x().float_value()
                && p.x().float_value() <= max.x().float_value()
                && p.y().float_value() >= min.y().float_value()
                && p.y().float_value() <= max.y().float_value()
        })
    }

    #[quickcheck]
    fn translation_preserves_point_count(path: Path, dx: i32, dy: i32) -> bool {
        let units = path.points()[0].units().0;
        let dx = (dx % 10_000).clamp(-10_000, 10_000);
        let dy = (dy % 10_000).clamp(-10_000, 10_000);
        let delta = Point::integer(dx, dy, units);
        let translated = path.clone().translate(delta);
        translated.points().len() == path.points().len()
    }
}

#[allow(clippy::needless_pass_by_value)]
mod text_tests {
    use super::*;

    #[quickcheck]
    fn double_reflection_cancels(text: Text) -> bool {
        let centre = Point::integer(0, 0, text.origin().units().0);
        let reflected_twice = text.clone().reflect(0.0, centre).reflect(0.0, centre);
        reflected_twice.x_reflection() == text.x_reflection()
    }

    #[quickcheck]
    fn scale_multiplies_magnification(text: Text) -> bool {
        let centre = Point::integer(0, 0, text.origin().units().0);
        let scaled = text.clone().scale(2.0, centre);
        (scaled.magnification() - text.magnification() * 2.0).abs() < 1e-10
    }
}

#[allow(clippy::needless_pass_by_value)]
mod grid_tests {
    use super::*;

    #[quickcheck]
    fn double_reflection_cancels(grid: Grid) -> bool {
        let centre = Point::integer(0, 0, 1e-9);
        let transformed = grid.reflect(0.0, centre).reflect(0.0, centre);
        !transformed.x_reflection()
    }

    #[quickcheck]
    fn translation_preserves_dimensions(grid: Grid) -> bool {
        let delta = Point::integer(42, -17, 1e-9);
        let translated = grid.clone().translate(delta);
        translated.columns() == grid.columns()
            && translated.rows() == grid.rows()
            && translated.magnification() == grid.magnification()
    }
}
