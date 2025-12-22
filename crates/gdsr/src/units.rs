use std::ops::{Add, Div, Mul, Sub};

type IntegerType = i32;
type FloatType = f64;
type UnitsType = f64;

#[derive(Clone, Copy, Debug)]
pub struct IntegerUnit {
    pub value: IntegerType,
    pub units: UnitsType,
}

#[derive(Clone, Copy, Debug)]
pub struct FloatUnit {
    pub value: FloatType,
    pub units: UnitsType,
}

/// Represents a unit of measurement.
///
/// Across this crate, if there is any notion of default units, for these types they will be defined as follows:
/// - Integer: `units` = 1e-9
/// - Float: `units` = 1e-6
///
/// For all binary operations, the units of the new object, are that of the first operand.
/// The second operand is scaled to match the first operand's units.
#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Integer(IntegerUnit),
    Float(FloatUnit),
}

pub const DEFAULT_INTEGER_UNITS: UnitsType = 1e-9;
pub const DEFAULT_FLOAT_UNITS: UnitsType = 1e-6;

impl Unit {
    /// Creates a new integer unit with the given value and units.
    pub const fn integer(value: IntegerType, units: UnitsType) -> Self {
        Self::Integer(IntegerUnit { value, units })
    }

    /// Creates a new float unit with the given value and units.
    pub const fn float(value: FloatType, units: UnitsType) -> Self {
        Self::Float(FloatUnit { value, units })
    }

    /// Creates a new integer unit with the given value and default units (1e-9).
    pub const fn default_integer(value: IntegerType) -> Self {
        Self::Integer(IntegerUnit {
            value,
            units: DEFAULT_INTEGER_UNITS,
        })
    }

    /// Creates a new float unit with the given value and default units (1e-6).
    pub const fn default_float(value: FloatType) -> Self {
        Self::Float(FloatUnit {
            value,
            units: DEFAULT_FLOAT_UNITS,
        })
    }

    pub fn zero() -> Self {
        Self::default()
    }

    /// Returns the inner value as a float, disregarding units.
    pub const fn float_value(&self) -> FloatType {
        match self {
            Self::Integer(IntegerUnit { value, .. }) => *value as f64,
            Self::Float(FloatUnit { value, .. }) => *value,
        }
    }

    /// Returns the inner value as an integer, rounding to the nearest integer.
    pub fn integer_value(&self) -> IntegerType {
        match self {
            Self::Integer(IntegerUnit { value, .. }) => *value,
            Self::Float(FloatUnit { value, .. }) => value.round() as IntegerType,
        }
    }

    /// Returns the absolute value of the unit.
    pub fn absolute_value(&self) -> f64 {
        match self {
            Self::Integer(IntegerUnit { value, units }) => f64::from(*value) * units,
            Self::Float(FloatUnit { value, units }) => *value * units,
        }
    }

    /// Converts the unit to an integer unit.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self::Integer(self.as_integer_unit())
    }

    /// Converts the unit to a float unit.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self::Float(self.as_float_unit())
    }

    /// Converts the unit to an integer unit.
    #[must_use]
    pub fn as_integer_unit(self) -> IntegerUnit {
        match self {
            Self::Integer(integer) => integer,
            Self::Float(FloatUnit { value, units }) => {
                // Convert float value (in units) to integer
                let value = value.round() as i32;
                IntegerUnit { value, units }
            }
        }
    }

    /// Converts the unit to a float unit.
    #[must_use]
    pub fn as_float_unit(self) -> FloatUnit {
        match self {
            Self::Integer(IntegerUnit { value, units }) => {
                // Convert integer value to float with new units
                let real_value = f64::from(value);
                FloatUnit {
                    value: real_value,
                    units,
                }
            }
            Self::Float(float) => float,
        }
    }

    /// Returns the units for this `Unit`.
    pub const fn units(&self) -> UnitsType {
        match self {
            Self::Integer(IntegerUnit { units, .. }) | Self::Float(FloatUnit { units, .. }) => {
                *units
            }
        }
    }

    /// Returns a copy of this Unit with the specified units.
    #[must_use]
    pub const fn set_units(&self, new_units: UnitsType) -> Self {
        match self {
            Self::Integer(IntegerUnit { value, .. }) => Self::Integer(IntegerUnit {
                value: *value,
                units: new_units,
            }),
            Self::Float(FloatUnit { value, .. }) => Self::Float(FloatUnit {
                value: *value,
                units: new_units,
            }),
        }
    }

    /// Returns a copy of this Unit with the specified units.
    /// The units of the new `Unit` are equal to `new_units`,
    /// and the value is scaled accordingly.
    #[must_use]
    pub fn scale_to(&self, new_units: UnitsType) -> Self {
        let scale_factor = self.units() / new_units;
        match self {
            Self::Integer(IntegerUnit { value, .. }) => Self::Integer(IntegerUnit {
                value: (f64::from(*value) * scale_factor).round() as i32,
                units: new_units,
            }),
            Self::Float(FloatUnit { value, .. }) => Self::Float(FloatUnit {
                value: *value * scale_factor,
                units: new_units,
            }),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(IntegerUnit { value, units }) => {
                write!(f, "{value} ({units:.3e})")
            }
            Self::Float(FloatUnit { value, units }) => {
                write!(f, "{value:.6} ({units:.3e})")
            }
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Self::default_integer(0)
    }
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for Unit {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                let scaled_rhs = rhs.to_float_unit().scale_to(self.units());
                match self {
                    Self::Integer(IntegerUnit {
                        value: v1,
                        units: u1,
                    }) => Self::Integer(IntegerUnit {
                        value: v1 $op scaled_rhs.integer_value(),
                        units: u1,
                    }),
                    Self::Float(FloatUnit {
                        value: v1,
                        units: u1,
                    }) => Self::Float(FloatUnit {
                        value: v1 $op scaled_rhs.float_value(),
                        units: u1,
                    }),
                }
            }
        }
    };
}

impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);

macro_rules! impl_scalar_op {
    ($trait:ident, $method:ident, $($scalar_type:ty => { int: $int_expr:expr, float: $float_expr:expr }),+ $(,)?) => {
        $(
            impl $trait<$scalar_type> for Unit {
                type Output = Self;
                fn $method(self, scalar: $scalar_type) -> Self::Output {
                    match self {
                        Self::Integer(IntegerUnit { value, units }) => {
                            Self::Integer(IntegerUnit {
                                value: $int_expr(value, scalar),
                                units,
                            })
                        }
                        Self::Float(FloatUnit { value, units }) => {
                            Self::Float(FloatUnit {
                                value: $float_expr(value, scalar),
                                units,
                            })
                        }
                    }
                }
            }
        )+
    };
}

impl_scalar_op!(Mul, mul,
    f64 => { int: |v: i32, s: f64| (f64::from(v) * s).round() as i32, float: |v: f64, s| v * s },
    i32 => { int: |v: i32, s| v * s, float: |v: f64, s| v * f64::from(s) },
    u32 => { int: |v: i32, s| v * s as i32, float: |v: f64, s| v * f64::from(s) },
);

impl_scalar_op!(Div, div,
    f64 => { int: |v: i32, s: f64| (f64::from(v) / s).round() as i32, float: |v: f64, s| v / s },
    i32 => { int: |v: i32, s: i32| if v % s == 0 { v / s } else { (f64::from(v) / f64::from(s)).round() as i32 }, float: |v: f64, s| v / f64::from(s) },
    u32 => { int: |v: i32, s| (f64::from(v) / f64::from(s)).round() as i32, float: |v: f64, s| v / f64::from(s) },
);

const EPSILON: f64 = 1e-15;

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        let self_real = self.absolute_value();

        let other_real = other.absolute_value();

        (self_real - other_real).abs() < EPSILON
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
#[cfg(test)]
mod tests {

    use super::*;

    use quickcheck::{Arbitrary, Gen};

    const MAX_VALUE: i32 = 10_000;

    impl Arbitrary for IntegerUnit {
        fn arbitrary(g: &mut Gen) -> Self {
            let value = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            Self { value, units }
        }
    }

    impl Arbitrary for FloatUnit {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_value = f64::arbitrary(g);
            let value = if raw_value.is_finite() {
                (raw_value % f64::from(MAX_VALUE))
                    .clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
            } else {
                0.0 // Replace NaN/inf with 0
            };
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            Self { value, units }
        }
    }

    impl Arbitrary for Unit {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                Self::Integer(IntegerUnit::arbitrary(g))
            } else {
                Self::Float(FloatUnit::arbitrary(g))
            }
        }
    }

    mod creation {
        use super::*;

        #[test]
        fn integer() {
            let unit = Unit::integer(100, 0.001);

            let integer_unit = unit.as_integer_unit();
            assert_eq!(integer_unit.value, 100);
            assert_eq!(integer_unit.units, 0.001);
        }

        #[test]
        fn float() {
            let unit = Unit::float(100.5, 1.0);

            let float_unit = unit.as_float_unit();
            assert_eq!(float_unit.value, 100.5);
            assert_eq!(float_unit.units, 1.0);
        }

        #[test]
        fn zero() {
            let unit = Unit::zero();

            assert_eq!(unit, Unit::default_float(0.0));
            assert_eq!(unit, Unit::default_integer(0));
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn same_integer_same_units() {
            let a = Unit::integer(100, 1e-9);
            let b = Unit::integer(100, 1e-9);
            assert_eq!(a, b);
        }

        #[test]
        fn same_float_same_units() {
            let a = Unit::float(100.5, 1e-6);
            let b = Unit::float(100.5, 1e-6);
            assert_eq!(a, b);
        }

        #[test]
        fn integer_different_scales_equal() {
            let a = Unit::integer(1000, 1e-9);
            let b = Unit::integer(1, 1e-6);
            assert_eq!(a, b);
        }

        #[test]
        fn float_different_scales_equal() {
            let a = Unit::float(1000.0, 1e-9);
            let b = Unit::float(1.0, 1e-6);
            assert_eq!(a, b);
        }

        #[test]
        fn integer_and_float_equal() {
            let a = Unit::integer(1000, 1e-9);
            let b = Unit::float(1.0, 1e-6);
            assert_eq!(a, b);
        }

        #[test]
        fn different_values_not_equal() {
            let a = Unit::integer(100, 1e-9);
            let b = Unit::integer(200, 1e-9);
            assert_ne!(a, b);
        }

        #[test]
        fn different_real_values_not_equal() {
            let a = Unit::integer(1000, 1e-9);
            let b = Unit::integer(2000, 1e-9);
            assert_ne!(a, b);
        }
    }

    mod unit_setters {
        use super::*;

        #[test]
        fn set_units_integer() {
            let unit = Unit::integer(100, 1e-9).set_units(1e-6);

            assert_eq!(unit, Unit::integer(100, 1e-6));
        }

        #[test]
        fn set_units_float() {
            let unit = Unit::float(1.5, 1e-6).set_units(1e-12);

            assert_eq!(unit, Unit::float(1.5, 1e-12));
        }

        #[test]
        fn with_units_integer() {
            let unit = Unit::integer(100, 1e-9);
            let new_unit = unit.set_units(1e-6);

            assert_eq!(new_unit, Unit::integer(100, 1e-6));
        }

        #[test]
        fn with_units_float() {
            let unit = Unit::float(1.5, 1e-6);
            let new_unit = unit.set_units(1e-12);

            assert_eq!(new_unit, Unit::float(1.5, 1e-12));
        }
    }

    mod conversion {
        use approx::assert_relative_eq;

        use crate::FloatUnit;

        use super::*;

        #[test]
        fn absolute_value() {
            let unit = Unit::float(2.5, 1e-6);
            let result = unit.absolute_value();
            assert_relative_eq!(result, 2.5 * 1e-6);
        }

        #[test]
        fn integer_value() {
            let unit = Unit::float(2.6, 1e-6);
            let result = unit.integer_value();
            assert_eq!(result, 3);

            let unit = Unit::integer(100, 0.001);
            let result = unit.integer_value();
            assert_eq!(result, 100);
        }

        #[test]
        fn to_integer_from_integer() {
            let unit = Unit::integer(100, 0.001);
            let result = unit.to_integer_unit();
            assert_eq!(result, unit);
        }

        #[test]
        fn to_integer_from_float() {
            let unit = Unit::float(1.007, 1e-6);
            let result = unit.to_integer_unit();

            assert_eq!(result, Unit::integer(1, 1e-6));
        }

        #[test]
        fn to_integer_with_user_unit() {
            let unit = Unit::float(100.0, 0.001);
            let result = unit.to_integer_unit();

            assert_eq!(result, Unit::integer(100, 1e-3));
        }

        #[test]
        fn to_float_from_float() {
            let unit = Unit::float(100.5, 1e-6);
            let result = unit.to_float_unit();
            assert_eq!(result, unit);
        }

        #[test]
        fn to_float_from_integer() {
            let unit = Unit::integer(100, 1e-9);
            let result = unit.to_float_unit().scale_to(1e-6);

            assert_eq!(result, Unit::float(0.1, 1e-6));
        }

        #[test]
        fn negative_values() {
            let unit = Unit::float(-100.5, 1e-6);
            let result = unit.to_integer_unit();

            assert_eq!(result, Unit::integer(-101, 1e-6));
        }

        #[test]
        fn zero_value() {
            let unit = Unit::float(0.0, 1e-6);
            let result = unit.to_integer_unit();

            assert_eq!(result, Unit::integer(0, 1e-6));
        }

        #[test]
        fn test_as_integer_unit() {
            let unit = Unit::integer(101, 1e-6);
            let IntegerUnit { value, units } = unit.as_integer_unit();

            assert_eq!(value, 101);
            assert_eq!(units, 1e-6);
        }

        #[test]
        fn test_as_float_unit() {
            let unit = Unit::float(101.0, 1e-6);
            let FloatUnit { value, units } = unit.as_float_unit();

            assert_eq!(value, 101.0);
            assert_eq!(units, 1e-6);
        }
    }

    mod addition {
        use super::*;

        #[test]
        fn integers() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 + u2;
            assert_eq!(result, Unit::integer(150, 1e-9));
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::float(50.3, 1e-6);
            let result = u1 + u2;
            assert_eq!(result, Unit::float(150.8, 1e-6));
        }

        #[test]
        fn different_units() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(50, 1e-6);
            let result = u1 + u2;
            assert_eq!(result, Unit::integer(50100, 1e-9));
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::float(50.5, 1e-6);
            let result = u1 + u2;
            assert_eq!(result, Unit::integer(50600, 1e-9));
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(50.5, 1e-6);
            let u2 = Unit::integer(100, 1e-9);
            let result = u1 + u2;
            assert_eq!(result, Unit::float(50.6, 1e-6));
        }

        #[test]
        fn different_user_units() {
            let u1 = Unit::float(100.0, 1e-6);
            let u2 = Unit::float(50.0, 1e-3);
            let result = u1 + u2;
            assert_eq!(result, Unit::float(50100.0, 1e-6));
        }
    }

    mod subtraction {

        use super::*;

        #[test]
        fn integers() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(30, 1e-9);
            let result = u1 - u2;
            assert_eq!(result, Unit::integer(70, 1e-9));
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::float(50.3, 1e-6);
            let result = u1 - u2;
            assert_eq!(result, Unit::float(50.2, 1e-6));
        }

        #[test]
        fn different_units() {
            let u1 = Unit::integer(100, 1e-6);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;
            assert_eq!(result, Unit::integer(100, 1e-6));
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;
            assert_eq!(result, Unit::float(100.45, 1e-6));
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::float(50.5, 1e-9);
            let result = u1 - u2;
            assert_eq!(result, Unit::integer(49, 1e-9));
        }
    }

    mod multiplication {

        use super::*;

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u * 3;
            assert_eq!(result, Unit::integer(300, 1e-9));
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u * 2.5;
            assert_eq!(result, Unit::integer(250, 1e-9));
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.5, 1e-6);
            let result = u * 2.0;
            assert_eq!(result, Unit::float(201.0, 1e-6));
        }

        #[test]
        fn float_by_i32() {
            let u = Unit::float(100.5, 1e-6);
            let result = u * 2i32;
            assert_eq!(result, Unit::float(201.0, 1e-6));
        }

        #[test]
        fn float_by_u32() {
            let u = Unit::float(100.5, 1e-6);
            let result = u * 2u32;
            assert_eq!(result, Unit::float(201.0, 1e-6));
        }

        #[test]
        fn integers() {
            let u1 = Unit::integer(10, 1e-3);
            let u2 = Unit::integer(5, 1e-6);
            let result = u1 * u2;
            assert_eq!(result, Unit::integer(0, 1e-3));
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(2.0, 1e-3);
            let u2 = Unit::float(3.0, 1e-3);
            let result = u1 * u2;
            assert_eq!(result, Unit::float(6.0, 1e-3));
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(10, 1e-3);
            let u2 = Unit::float(5.0, 1e-3);
            let result = u1 * u2;
            assert_eq!(result, Unit::integer(50, 1e-3));
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(5.0, 1e-3);
            let u2 = Unit::integer(10, 1e-3);
            let result = u1 * u2;
            assert_eq!(result, Unit::float(50.0, 1e-3));
        }
    }

    mod division {

        use super::*;

        #[test]
        fn integer_by_i32_exact() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 4;
            assert_eq!(result, Unit::integer(25, 1e-9));
        }

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 3;
            assert_eq!(result, Unit::integer(33, 1e-9));
        }

        #[test]
        fn float_by_i32() {
            let u = Unit::float(100.0, 1e-9);
            let result = u / 4;
            assert_eq!(result, Unit::float(25.0, 1e-9));
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 2.5;
            assert_eq!(result, Unit::integer(40, 1e-9));
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.0, 1e-6);
            let result = u / 2.0;
            assert_eq!(result, Unit::float(50.0, 1e-6));
        }

        #[test]
        fn float_by_u32() {
            let u = Unit::float(100.0, 1e-9);
            let result = u / 4u32;
            assert_eq!(result, Unit::float(25.0, 1e-9));
        }
    }

    #[test]
    fn chained_operations() {
        let u1 = Unit::integer(100, 1e-9);
        let u2 = Unit::integer(50, 1e-9);
        let result = (u1 + u2) * 2;
        assert_eq!(result, Unit::integer(300, 1e-9));
    }

    mod display_and_default {

        use super::*;

        #[test]
        fn test_display_integer() {
            let unit = Unit::integer(100, 1e-9);
            let display = format!("{unit}");
            assert_eq!(display, "100 (1.000e-9)");
        }

        #[test]
        fn test_display_float() {
            let unit = Unit::float(1.5, 1e-6);
            let display = format!("{unit}");
            assert_eq!(display, "1.500000 (1.000e-6)");
        }

        #[test]
        fn test_display_negative_integer() {
            let unit = Unit::integer(-50, 1e-9);
            let display = format!("{unit}");
            assert_eq!(display, "-50 (1.000e-9)");
        }

        #[test]
        fn test_display_negative_float() {
            let unit = Unit::float(-2.75, 1e-6);
            let display = format!("{unit}");
            assert_eq!(display, "-2.750000 (1.000e-6)");
        }

        #[test]
        fn test_default() {
            let unit = Unit::default();
            assert_eq!(unit, Unit::integer(0, 1e-9));
        }

        #[test]
        fn test_scale_units_integer() {
            let unit = Unit::integer(1000, 1e-9);
            let scaled = unit.scale_to(1e-6);

            assert_eq!(scaled, Unit::integer(1, 1e-6));
        }

        #[test]
        fn test_scale_units_float() {
            let unit = Unit::float(1000.0, 1e-9);
            let scaled = unit.scale_to(1e-6);

            assert_eq!(scaled, Unit::float(1.0, 1e-6));
        }
    }

    mod property_tests {
        use super::*;
        use quickcheck_macros::quickcheck;

        fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
            (a - b).abs() < epsilon
        }

        #[quickcheck]
        fn float_addition_commutativity(a: FloatUnit, b: FloatUnit) -> bool {
            let a = Unit::Float(a);
            let b = Unit::Float(b);
            let sum_a_b = a + b;
            let sum_b_a = b + a;

            sum_a_b == sum_b_a
        }

        #[quickcheck]
        fn float_addition_associativity(a: FloatUnit, b: FloatUnit, c: FloatUnit) -> bool {
            let a = Unit::Float(a);
            let b = Unit::Float(b);
            let c = Unit::Float(c);
            let sum_ab_c = (a + b) + c;
            let sum_a_bc = a + (b + c);

            sum_a_bc == sum_ab_c
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
            let abs_a = a.absolute_value();
            let abs_b = b.absolute_value();
            let expected = abs_a + abs_b;

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
            let abs_result = result.absolute_value().abs();
            abs_result < 1e-9
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

            let integer_result = result.to_integer_unit();

            integer_result == Unit::integer(a.value + b.value, a.units)
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

            let float_result = result.to_float_unit();

            float_result == Unit::float(a.value + b.value, a.units)
        }
    }
}
