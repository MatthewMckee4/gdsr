/// Represents a unit of measurement.
///
/// Across this crate, if there is any notion of default units, for these types they will be defined as follows:
/// - Integer: `units` = 1e-9
/// - Float: `units` = 1e-6
#[derive(Clone, Copy)]
pub enum Unit {
    Integer { value: i32, units: f64 },
    Float { value: f64, units: f64 },
}

#[derive(Debug, Clone, Copy)]
pub enum UnitsError {
    MismatchedUnits(f64, f64),
}

const DEFAULT_INTEGER_UNITS: f64 = 1e-9;
const DEFAULT_FLOAT_UNITS: f64 = 1e-6;

impl Unit {
    #[must_use]
    pub const fn integer(value: i32, units: f64) -> Self {
        Self::Integer { value, units }
    }

    #[must_use]
    pub const fn float(value: f64, units: f64) -> Self {
        Self::Float { value, units }
    }

    #[must_use]
    pub const fn as_float(&self) -> f64 {
        match self {
            Self::Integer { value, .. } => *value as f64,
            Self::Float { value, .. } => *value,
        }
    }

    #[must_use]
    pub fn true_value(&self) -> f64 {
        match self {
            Self::Integer { value, units } => f64::from(*value) * units,
            Self::Float { value, units } => *value * units,
        }
    }

    #[must_use]
    pub const fn expect_integer_value(&self) -> i32 {
        match self {
            Self::Integer { value, .. } => *value,
            Self::Float { .. } => panic!("Called expect_integer_value on a float unit"),
        }
    }

    #[must_use]
    pub const fn expect_float_value(&self) -> f64 {
        match self {
            Self::Integer { value, .. } => *value as f64,
            Self::Float { value, .. } => *value,
        }
    }

    #[must_use]
    pub const fn default_integer(value: i32) -> Self {
        Self::Integer {
            value,
            units: DEFAULT_INTEGER_UNITS,
        }
    }

    #[must_use]
    pub const fn default_float(value: f64) -> Self {
        Self::Float {
            value,
            units: DEFAULT_FLOAT_UNITS,
        }
    }

    #[must_use]
    pub fn to_integer_unit(&self) -> Self {
        match self {
            Self::Integer { .. } => *self,
            Self::Float { value, units } => {
                // Convert float value (in units) to integer
                let value = value.round() as i32;
                Self::Integer {
                    value,
                    units: *units,
                }
            }
        }
    }

    #[must_use]
    pub fn to_float_unit(&self, units: f64) -> Self {
        match self {
            Self::Integer {
                value,
                units: current_units,
            } => {
                // Convert integer value to float with new units
                let real_value = f64::from(*value) * current_units;
                let value = real_value / units;
                Self::Float { value, units }
            }
            Self::Float { .. } => *self,
        }
    }

    /// Sets the units for this Unit.
    #[must_use]
    pub const fn units(&self) -> f64 {
        match self {
            Self::Integer { units, .. } | Self::Float { units, .. } => *units,
        }
    }

    /// Sets the units for this Unit (mutable).
    pub const fn set_units(&mut self, new_units: f64) {
        match self {
            Self::Integer { units, .. } | Self::Float { units, .. } => *units = new_units,
        }
    }

    /// Returns a copy of this Unit with the specified units.
    #[must_use]
    pub const fn with_units(&self, new_units: f64) -> Self {
        match self {
            Self::Integer { value, .. } => Self::Integer {
                value: *value,
                units: new_units,
            },
            Self::Float { value, .. } => Self::Float {
                value: *value,
                units: new_units,
            },
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, units } => {
                write!(f, "{value}i (units: {units:.3e})")
            }
            Self::Float { value, units } => {
                write!(f, "{value:.6}f (units: {units:.3e})")
            }
        }
    }
}

impl Debug for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, units } => {
                write!(f, "{value} (units: {units:.3e})")
            }
            Self::Float { value, units } => {
                write!(f, "{value:.6} (units: {units:.3e})")
            }
        }
    }
}

use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

impl Add for Unit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Integer + Integer: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                // Convert v2 to real units, then to self's units
                let v2_real = f64::from(v2) * u2;
                let v2_in_u1 = (v2_real / u1).round() as i32;
                Self::Integer {
                    value: v1 + v2_in_u1,
                    units: u1,
                }
            }
            // Integer + Float: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = v2 * u2;
                let v2_in_u1 = (v2_real / u1).round() as i32;
                Self::Integer {
                    value: v1 + v2_in_u1,
                    units: u1,
                }
            }
            // Float + Integer: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = f64::from(v2) * u2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 + v2_in_u1,
                    units: u1,
                }
            }
            // Float + Float: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = v2 * u2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 + v2_in_u1,
                    units: u1,
                }
            }
        }
    }
}

impl Sub for Unit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Integer - Integer: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = f64::from(v2) * u2;
                let v2_in_u1 = (v2_real / u1).round() as i32;
                Self::Integer {
                    value: v1 - v2_in_u1,
                    units: u1,
                }
            }
            // Integer - Float: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = v2 * u2;
                let v2_in_u1 = (v2_real / u1).round() as i32;
                Self::Integer {
                    value: v1 - v2_in_u1,
                    units: u1,
                }
            }
            // Float - Integer: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = f64::from(v2) * u2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 - v2_in_u1,
                    units: u1,
                }
            }
            // Float - Float: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let v2_real = v2 * u2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 - v2_in_u1,
                    units: u1,
                }
            }
        }
    }
}

impl Mul<f64> for Unit {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: (f64::from(value) * scalar).round() as i32,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value * scalar,
                units,
            },
        }
    }
}

impl Mul<i32> for Unit {
    type Output = Self;

    fn mul(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: value * scalar,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value * f64::from(scalar),
                units,
            },
        }
    }
}

impl Mul<u32> for Unit {
    type Output = Self;

    fn mul(self, scalar: u32) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: value * scalar as i32,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value * f64::from(scalar),
                units,
            },
        }
    }
}

impl Div<f64> for Unit {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: (f64::from(value) / scalar).round() as i32,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value / scalar,
                units,
            },
        }
    }
}

impl Div<i32> for Unit {
    type Output = Self;

    fn div(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, units } => {
                if value % scalar == 0 {
                    Self::Integer {
                        value: value / scalar,
                        units,
                    }
                } else {
                    Self::Integer {
                        value: (f64::from(value) / f64::from(scalar)).round() as i32,
                        units,
                    }
                }
            }
            Self::Float { value, units } => Self::Float {
                value: value / f64::from(scalar),
                units,
            },
        }
    }
}

impl Div<u32> for Unit {
    type Output = Self;

    fn div(self, scalar: u32) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: (f64::from(value) / f64::from(scalar)).round() as i32,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value / f64::from(scalar),
                units,
            },
        }
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        // Convert both to real-world values and compare
        let self_real = match self {
            Self::Integer { value, units } => f64::from(*value) * units,
            Self::Float { value, units } => value * units,
        };

        let other_real = match other {
            Self::Integer { value, units } => f64::from(*value) * units,
            Self::Float { value, units } => value * units,
        };

        // Use a small epsilon for floating point comparison
        (self_real - other_real).abs() < 1e-15
    }
}

// From implementations for common types
impl From<i32> for Unit {
    fn from(value: i32) -> Self {
        Self::default_integer(value)
    }
}

impl From<u32> for Unit {
    fn from(value: u32) -> Self {
        Self::default_integer(value as i32)
    }
}

impl From<f64> for Unit {
    fn from(value: f64) -> Self {
        Self::default_float(value)
    }
}

impl Sub<i32> for Unit {
    type Output = Self;

    fn sub(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: value - scalar,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value - f64::from(scalar),
                units,
            },
        }
    }
}

impl Sub<f64> for Unit {
    type Output = Self;

    fn sub(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, units } => Self::Integer {
                value: (f64::from(value) - scalar).round() as i32,
                units,
            },
            Self::Float { value, units } => Self::Float {
                value: value - scalar,
                units,
            },
        }
    }
}

impl Mul for Unit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Integer * Integer: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                let real1 = f64::from(v1) * u1;
                let real2 = f64::from(v2) * u2;
                let result = (real1 * real2) / u1;
                Self::Integer {
                    value: result.round() as i32,
                    units: u1,
                }
            }
            // Integer * Float: always return Integer with self's units
            (
                Self::Integer {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let scale = u1 / u2;
                let real1 = f64::from(v1);
                let real2 = v2 * scale;
                let result = (real1 * real2) / scale;
                Self::Integer {
                    value: result.round() as i32,
                    units: u1,
                }
            }
            // Float * Integer: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Integer {
                    value: v2,
                    units: u2,
                },
            ) => {
                let scale = u1 / u2;
                let real1 = v1;
                let real2 = f64::from(v2) * scale;
                let result = (real1 * real2) / scale;
                Self::Float {
                    value: result,
                    units: u1,
                }
            }
            // Float * Float: always return Float with self's units
            (
                Self::Float {
                    value: v1,
                    units: u1,
                },
                Self::Float {
                    value: v2,
                    units: u2,
                },
            ) => {
                let scale = u1 / u2;
                let real1 = v1;
                let real2 = v2 * scale;
                let result = (real1 * real2) / scale;
                Self::Float {
                    value: result,
                    units: u1,
                }
            }
        }
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
#[cfg(test)]
mod tests {
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    use super::*;

    impl Arbitrary for Unit {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_exponent = (i32::arbitrary(g) % 10) - 12;
            let units = 10_f64.powi(units_exponent);

            if bool::arbitrary(g) {
                let value = (i32::arbitrary(g) % 1_000_000) - 500_000;
                Self::Integer { value, units }
            } else {
                let mut value = f64::arbitrary(g);
                if !value.is_finite() || value.abs() > 1e100 {
                    value = f64::from(i32::arbitrary(g) % 1_000_000) / 1000.0;
                }
                Self::Float { value, units }
            }
        }
    }

    mod creation {
        use super::*;

        #[test]
        fn integer() {
            let unit = Unit::integer(100, 0.001);
            match unit {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 0.001);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float() {
            let unit = Unit::float(100.5, 1.0);
            match unit {
                Unit::Float { value, units } => {
                    assert_eq!(value, 100.5);
                    assert_eq!(units, 1.0);
                }
                _ => panic!("Expected Float variant"),
            }
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
            let a = Unit::integer(1000, 1e-9); // 1000 * 1e-9 = 1e-6
            let b = Unit::integer(1, 1e-6); // 1 * 1e-6 = 1e-6
            assert_eq!(a, b);
        }

        #[test]
        fn float_different_scales_equal() {
            let a = Unit::float(1000.0, 1e-9); // 1000 * 1e-9 = 1e-6
            let b = Unit::float(1.0, 1e-6); // 1 * 1e-6 = 1e-6
            assert_eq!(a, b);
        }

        #[test]
        fn integer_and_float_equal() {
            let a = Unit::integer(1000, 1e-9); // 1000 * 1e-9 = 1e-6
            let b = Unit::float(1.0, 1e-6); // 1 * 1e-6 = 1e-6
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
            let a = Unit::integer(1000, 1e-9); // 1e-6
            let b = Unit::integer(2000, 1e-9); // 2e-6
            assert_ne!(a, b);
        }
    }

    mod from_conversions {
        use super::*;

        #[test]
        fn from_i32() {
            let unit = Unit::from(100);
            assert_eq!(unit, Unit::integer(100, 1e-9));
        }

        #[test]
        fn from_i32_negative() {
            let unit = Unit::from(-50);
            assert_eq!(unit, Unit::integer(-50, 1e-9));
        }

        #[test]
        fn from_i32_zero() {
            let unit = Unit::from(0);
            assert_eq!(unit, Unit::integer(0, 1e-9));
        }

        #[test]
        fn from_f64() {
            let unit = Unit::from(1.5);
            assert_eq!(unit, Unit::float(1.5, 1e-6));
        }

        #[test]
        fn from_f64_negative() {
            let unit = Unit::from(-3.5);
            assert_eq!(unit, Unit::float(-3.5, 1e-6));
        }

        #[test]
        fn from_f64_zero() {
            let unit = Unit::from(0.0);
            assert_eq!(unit, Unit::float(0.0, 1e-6));
        }

        #[test]
        fn from_f64_large() {
            let unit = Unit::from(1000.0);
            assert_eq!(unit, Unit::float(1000.0, 1e-6));
        }

        #[test]
        fn into_with_type_inference() {
            let unit: Unit = 100.into();
            assert_eq!(unit, Unit::integer(100, 1e-9));

            let unit: Unit = 1.5.into();
            assert_eq!(unit, Unit::float(1.5, 1e-6));
        }
    }

    mod unit_setters {
        use super::*;

        #[test]
        fn set_unitss_integer() {
            let mut unit = Unit::integer(100, 1e-9);
            unit.set_units(1e-6);

            match unit {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn set_unitss_float() {
            let mut unit = Unit::float(1.5, 1e-6);
            unit.set_units(1e-12);

            match unit {
                Unit::Float { value, units } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(units, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn with_units_integer() {
            let unit = Unit::integer(100, 1e-9);
            let new_unit = unit.with_units(1e-6);

            match unit {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }

            match new_unit {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn with_units_float() {
            let unit = Unit::float(1.5, 1e-6);
            let new_unit = unit.with_units(1e-12);

            match unit {
                Unit::Float { value, units } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }

            match new_unit {
                Unit::Float { value, units } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(units, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn set_methods_modify_in_place() {
            let mut unit = Unit::float(2.5, 1e-6);
            unit.set_units(1e-3);

            match unit {
                Unit::Float { value, units } => {
                    assert_eq!(value, 2.5);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod conversion {
        use approx::assert_relative_eq;

        use super::*;

        #[test]
        fn as_true_float_value() {
            let unit = Unit::float(2.5, 1e-6);
            let result = unit.true_value();
            assert_relative_eq!(result, 2.5 * 1e-6);
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

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 1);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_integer_with_user_unit() {
            let unit = Unit::float(100.0, 0.001);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_float_from_float() {
            let unit = Unit::float(100.5, 1e-6);
            let result = unit.to_float_unit(1e-6);
            assert_eq!(result, unit);
        }

        #[test]
        fn to_float_from_integer() {
            let unit = Unit::integer(100, 1e-9);
            let result = unit.to_float_unit(1e-6);

            match result {
                Unit::Float { value, units } => {
                    assert_relative_eq!(value, 0.1);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn roundtrip() {
            let original = Unit::integer(100, 1e-9);
            let as_float = original.to_float_unit(1e-6);
            let back_to_int = as_float.to_integer_unit();

            match back_to_int {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 0);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn negative_values() {
            let unit = Unit::float(-100.5, 1e-6);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, -101);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn zero_value() {
            let unit = Unit::float(0.0, 1e-6);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 0);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }
    }

    mod addition {
        use approx::assert_relative_eq;

        use super::*;

        #[test]
        fn integers() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 150);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::float(50.3, 1e-6);
            let result = u1 + u2;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 150.8).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_unitss() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(50, 1e-6);
            let result = u1 + u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 50100);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::float(50.5, 1e-6);
            let result = u1 + u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 50600);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(50.5, 1e-6);
            let u2 = Unit::integer(100, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 50.6).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_user_units() {
            let u1 = Unit::float(100.0, 1e-6);
            let u2 = Unit::float(50.0, 1e-3);
            let result = u1 + u2;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 50100.0).abs() < 1e-6);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod subtraction {
        use super::*;

        #[test]
        fn integers() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(30, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 70);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::float(50.3, 1e-6);
            let result = u1 - u2;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 50.2).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_units() {
            let u1 = Unit::integer(100, 1e-6);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 100);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(100.5, 1e-6);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 100.45).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u - 30;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 70);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_by_i32() {
            let u = Unit::float(100.5, 1e-6);
            let result = u - 10;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 90.5).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u - 25.5;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 75);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.5, 1e-6);
            let result = u - 25.5;

            match result {
                Unit::Float { value, units } => {
                    assert!((value - 75.0).abs() < 1e-10);

                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod multiplication {
        use super::*;

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u * 3;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 300);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u * 2.5;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 250);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.5, 1e-6);
            let result = u * 2.0;

            match result {
                Unit::Float { value, units } => {
                    assert_eq!(value, 201.0);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integers() {
            let u1 = Unit::integer(10, 1e-3);
            let u2 = Unit::integer(5, 1e-6);
            let result = u1 * u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 0);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(2.0, 1e-3);
            let u2 = Unit::float(3.0, 1e-3);
            let result = u1 * u2;

            match result {
                Unit::Float { value, units } => {
                    assert_eq!(value, 6.0);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(10, 1e-3);
            let u2 = Unit::float(5.0, 1e-3);
            let result = u1 * u2;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 50);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(5.0, 1e-3);
            let u2 = Unit::integer(10, 1e-3);
            let result = u1 * u2;

            match result {
                Unit::Float { value, units } => {
                    assert_eq!(value, 50.0);
                    assert_eq!(units, 1e-3);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod division {
        use super::*;

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 4;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 25);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 2.5;

            match result {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 40);
                    assert_eq!(units, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.0, 1e-6);
            let result = u / 2.0;

            match result {
                Unit::Float { value, units } => {
                    assert_eq!(value, 50.0);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    #[test]
    fn chained_operations() {
        let u1 = Unit::integer(100, 1e-9);
        let u2 = Unit::integer(50, 1e-9);
        let result = (u1 + u2) * 2;

        match result {
            Unit::Integer { value, units } => {
                assert_eq!(value, 300);
                assert_eq!(units, 1e-9);
            }
            _ => panic!("Expected Integer variant"),
        }
    }

    mod property_tests {
        use super::*;

        mod conversions {
            use super::*;

            #[quickcheck]
            fn integer_to_float_roundtrip(unit: Unit) -> TestResult {
                match unit {
                    Unit::Integer { value, units } => {
                        let as_float = unit.to_float_unit(units);
                        let back = as_float.to_integer_unit();

                        match back {
                            Unit::Integer {
                                value: back_value,
                                units: back_units,
                            } => TestResult::from_bool(
                                value == back_value && (units - back_units).abs() < 1e-10,
                            ),
                            _ => TestResult::failed(),
                        }
                    }
                    _ => TestResult::discard(),
                }
            }

            #[quickcheck]
            fn to_float_preserves_real_value(unit: Unit) -> bool {
                match unit {
                    Unit::Integer { value, units } => {
                        let as_float = unit.to_float_unit(1e-6);
                        if let Unit::Float {
                            value: float_value,
                            units: float_units,
                            ..
                        } = as_float
                        {
                            let original_real = f64::from(value) * units;
                            let float_real = float_value * float_units;
                            (original_real - float_real).abs() < 1e-10
                        } else {
                            false
                        }
                    }
                    Unit::Float { .. } => unit.to_float_unit(1e-6) == unit,
                }
            }

            #[quickcheck]
            fn preserves_sign(unit: Unit) -> TestResult {
                match unit {
                    Unit::Float { value, units, .. } => {
                        if !value.is_finite() {
                            return TestResult::discard();
                        }

                        let result = unit.to_integer_unit();
                        match result {
                            Unit::Integer {
                                value: int_value,
                                units: int_units,
                            } => {
                                let real_value = value * units;
                                if int_value == 0 {
                                    return TestResult::from_bool(real_value.abs() < 1.0);
                                }
                                TestResult::from_bool(
                                    (real_value >= 0.0 && int_value >= 0)
                                        || (real_value < 0.0 && int_value < 0),
                                )
                            }
                            _ => TestResult::failed(),
                        }
                    }
                    _ => TestResult::discard(),
                }
            }
        }

        mod addition {
            use super::*;

            #[quickcheck]
            fn commutative(a: Unit, b: Unit) -> bool {
                let a = Unit::integer(
                    if let Unit::Integer { value, .. } = a {
                        value
                    } else {
                        100
                    },
                    1e-9,
                );
                let b = Unit::integer(
                    if let Unit::Integer { value, .. } = b {
                        value
                    } else {
                        50
                    },
                    1e-9,
                );

                let ab = a + b;
                let ba = b + a;

                match (ab, ba) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => v1 == v2,
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        (v1 - v2).abs() < 1e-10
                    }
                    _ => false,
                }
            }

            #[quickcheck]
            fn associative(a: Unit, b: Unit, c: Unit) -> bool {
                let a = Unit::integer(
                    if let Unit::Integer { value, .. } = a {
                        value
                    } else {
                        100
                    },
                    1e-9,
                );
                let b = Unit::integer(
                    if let Unit::Integer { value, .. } = b {
                        value
                    } else {
                        50
                    },
                    1e-9,
                );
                let c = Unit::integer(
                    if let Unit::Integer { value, .. } = c {
                        value
                    } else {
                        25
                    },
                    1e-9,
                );

                let abc1 = (a + b) + c;
                let abc2 = a + (b + c);

                match (abc1, abc2) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => v1 == v2,
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        (v1 - v2).abs() < 1e-8
                    }
                    _ => false,
                }
            }

            #[quickcheck]
            fn zero_identity(unit: Unit) -> bool {
                let zero = match unit {
                    Unit::Integer { units, .. } => Unit::integer(0, units),
                    Unit::Float { units, .. } => Unit::float(0.0, units),
                };

                let result = unit + zero;

                match (unit, result) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => v1 == v2,
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        if v1.is_finite() && v2.is_finite() {
                            let diff = (v1 - v2).abs();
                            let rel_error = if v1.abs() > 1e-10 {
                                diff / v1.abs()
                            } else {
                                diff
                            };
                            rel_error < 1e-10 || diff < 1e-10
                        } else {
                            v1.is_finite() == v2.is_finite()
                        }
                    }
                    _ => false,
                }
            }
        }

        mod subtraction {
            use super::*;

            #[quickcheck]
            fn inverse_of_add(a: Unit, b: Unit) -> bool {
                let a = Unit::integer(
                    if let Unit::Integer { value, .. } = a {
                        value
                    } else {
                        1000
                    },
                    1e-9,
                );
                let b = Unit::integer(
                    if let Unit::Integer { value, .. } = b {
                        value
                    } else {
                        500
                    },
                    1e-9,
                );

                let sum = a + b;
                let diff = sum - b;

                match (diff, a) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => v1 == v2,
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        (v1 - v2).abs() < 1e-8
                    }
                    _ => false,
                }
            }
        }

        mod multiplication {
            use super::*;

            #[quickcheck]
            fn by_one_identity(unit: Unit) -> bool {
                let result = unit * 1;

                match (unit, result) {
                    (
                        Unit::Integer {
                            value: v1,
                            units: u1,
                        },
                        Unit::Integer {
                            value: v2,
                            units: u2,
                        },
                    ) => v1 == v2 && (u1 - u2).abs() < 1e-10,
                    (
                        Unit::Float {
                            value: v1,
                            units: u1,
                        },
                        Unit::Float {
                            value: v2,
                            units: u2,
                        },
                    ) => (v1 - v2).abs() < 1e-10 && (u1 - u2).abs() < 1e-10,
                    _ => false,
                }
            }

            #[quickcheck]
            fn by_zero_is_zero(unit: Unit) -> bool {
                #[allow(clippy::erasing_op)]
                let result = unit * 0;
                match result {
                    Unit::Float { value, .. } => value == 0.0 || value.abs() < 1e-10,
                    Unit::Integer { value, .. } => value == 0,
                }
            }

            #[quickcheck]
            fn distributive(unit: Unit, scalar1: i32, scalar2: i32) -> TestResult {
                let s1 = scalar1 % 10;
                let s2 = scalar2 % 10;

                if s1.checked_add(s2).is_none() {
                    return TestResult::discard();
                }

                let left = unit * (s1 + s2);
                let right = (unit * s1) + (unit * s2);

                match (left, right) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => {
                        TestResult::from_bool((v1 - v2).abs() < 2)
                    }
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        if !v1.is_finite() || !v2.is_finite() {
                            return TestResult::from_bool(v1.is_finite() == v2.is_finite());
                        }
                        let diff = (v1 - v2).abs();
                        let rel_error = if v1.abs().max(v2.abs()) > 1e-10 {
                            diff / v1.abs().max(v2.abs())
                        } else {
                            diff
                        };
                        TestResult::from_bool(rel_error < 1e-6)
                    }
                    _ => TestResult::failed(),
                }
            }

            #[quickcheck]
            fn associative(unit: Unit, s1: i32, s2: i32) -> TestResult {
                let s1 = (s1 % 10).max(-10);
                let s2 = (s2 % 10).max(-10);

                if s1.checked_mul(s2).is_none() {
                    return TestResult::discard();
                }

                let left = (unit * s1) * s2;
                let right = unit * (s1 * s2);

                match (left, right) {
                    (Unit::Integer { value: v1, .. }, Unit::Integer { value: v2, .. }) => {
                        TestResult::from_bool(v1 == v2)
                    }
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        if !v1.is_finite() || !v2.is_finite() {
                            return TestResult::from_bool(v1.is_finite() == v2.is_finite());
                        }
                        let diff = (v1 - v2).abs();
                        let rel_error = if v1.abs().max(v2.abs()) > 1e-10 {
                            diff / v1.abs().max(v2.abs())
                        } else {
                            diff
                        };
                        TestResult::from_bool(rel_error < 1e-6)
                    }
                    _ => TestResult::failed(),
                }
            }
        }

        mod division {
            use super::*;
        }
    }
}
