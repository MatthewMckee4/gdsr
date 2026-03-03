use std::f64::consts::TAU;

use quickcheck_macros::quickcheck;

use crate::*;

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
