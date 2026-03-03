use quickcheck_macros::quickcheck;

use crate::units::{FloatUnit, IntegerUnit};
use crate::*;

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

/// Verifies that a + (-a) ~ 0.
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
