use std::ops::{Add, Div, Mul, Sub};

/// Represents a unit of measurement.
///
/// Across this crate, if there is any notion of default units, for these types they will be defined as follows:
/// - Integer: `units` = 1e-9
/// - Float: `units` = 1e-6
#[derive(Clone, Copy, Debug)]
pub enum Unit {
    Integer { value: i32, units: f64 },
    Float { value: f64, units: f64 },
}

pub const DEFAULT_INTEGER_UNITS: f64 = 1e-9;
pub const DEFAULT_FLOAT_UNITS: f64 = 1e-6;

impl Unit {
    pub const fn integer(value: i32, units: f64) -> Self {
        Self::Integer { value, units }
    }

    pub const fn float(value: f64, units: f64) -> Self {
        Self::Float { value, units }
    }

    pub const fn as_float(&self) -> f64 {
        match self {
            Self::Integer { value, .. } => *value as f64,
            Self::Float { value, .. } => *value,
        }
    }

    pub fn true_value(&self) -> f64 {
        match self {
            Self::Integer { value, units } => f64::from(*value) * units,
            Self::Float { value, units } => *value * units,
        }
    }

    pub const fn expect_integer_value(&self) -> i32 {
        match self {
            Self::Integer { value, .. } => *value,
            Self::Float { .. } => panic!("Called expect_integer_value on a float unit"),
        }
    }

    pub const fn expect_float_value(&self) -> f64 {
        match self {
            Self::Integer { value, .. } => *value as f64,
            Self::Float { value, .. } => *value,
        }
    }

    pub const fn default_integer(value: i32) -> Self {
        Self::Integer {
            value,
            units: DEFAULT_INTEGER_UNITS,
        }
    }

    pub const fn default_float(value: f64) -> Self {
        Self::Float {
            value,
            units: DEFAULT_FLOAT_UNITS,
        }
    }

    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        match self {
            Self::Integer { .. } => self,
            Self::Float { value, units } => {
                // Convert float value (in units) to integer
                let value = value.round() as i32;
                Self::Integer { value, units }
            }
        }
    }

    #[must_use]
    pub fn to_float_unit(self) -> Self {
        match self {
            Self::Integer { value, units } => {
                // Convert integer value to float with new units
                let real_value = f64::from(value);
                Self::Float {
                    value: real_value,
                    units,
                }
            }
            Self::Float { .. } => self,
        }
    }

    /// Sets the units for this Unit.
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

    /// Returns a copy of this Unit with the specified units.
    /// The units of the new `Unit` are equal to `new_units`,
    /// and the value is scaled accordingly.
    #[must_use]
    pub fn scale_units(&self, new_units: f64) -> Self {
        let scale_factor = self.units() / new_units;
        match self {
            Self::Integer { value, .. } => Self::Integer {
                value: (f64::from(*value) * scale_factor).round() as i32,
                units: new_units,
            },
            Self::Float { value, .. } => Self::Float {
                value: *value * scale_factor,
                units: new_units,
            },
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, units } => {
                write!(f, "{value} ({units:.3e})")
            }
            Self::Float { value, units } => {
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
    use super::*;

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
        fn set_units_integer() {
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
        fn set_units_float() {
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
            let result = unit.to_float_unit();
            assert_eq!(result, unit);
        }

        #[test]
        fn to_float_from_integer() {
            let unit = Unit::integer(100, 1e-9);
            let result = unit.to_float_unit().scale_units(1e-6);

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
            let as_float = original.to_float_unit().scale_units(1e-6);
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
        fn different_units() {
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
            let scaled = unit.scale_units(1e-6);
            match scaled {
                Unit::Integer { value, units } => {
                    assert_eq!(value, 1);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn test_scale_units_float() {
            let unit = Unit::float(1000.0, 1e-9);
            let scaled = unit.scale_units(1e-6);
            match scaled {
                Unit::Float { value, units } => {
                    assert!((value - 1.0).abs() < 1e-10);
                    assert_eq!(units, 1e-6);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }
}
