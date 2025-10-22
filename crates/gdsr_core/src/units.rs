/// Represents a unit of measurement.
///
/// Across this crate, if there is any notion of default units, for these types they will be defined as follows:
/// - Integer: `db_unit` = 1e-9
/// - Float: `user_unit` = 1e-6, `db_unit` = 1e-9
#[derive(Debug, Clone, Copy)]
pub enum Unit {
    Integer {
        value: i32,
        db_unit: f64,
    },
    Float {
        value: f64,
        user_unit: f64,
        db_unit: f64,
    },
}

const DEFAULT_DB_UNIT: f64 = 1e-9;
const DEFAULT_FLOAT_USER_UNIT: f64 = 1e-6;

impl Unit {
    #[must_use]
    pub const fn integer(value: i32, db_unit: f64) -> Self {
        Self::Integer { value, db_unit }
    }

    #[must_use]
    pub const fn float(value: f64, user_unit: f64, db_unit: f64) -> Self {
        Self::Float {
            value,
            user_unit,
            db_unit,
        }
    }

    #[must_use]
    pub const fn default_integer(value: i32) -> Self {
        Self::Integer {
            value,
            db_unit: DEFAULT_DB_UNIT,
        }
    }

    #[must_use]
    pub const fn default_float(value: f64) -> Self {
        Self::Float {
            value,
            user_unit: DEFAULT_FLOAT_USER_UNIT,
            db_unit: DEFAULT_DB_UNIT,
        }
    }

    #[must_use]
    pub fn to_integer_unit(&self) -> Self {
        match self {
            Self::Integer { .. } => *self,
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => {
                // Convert from user units to db units
                let value_in_real_units = value * user_unit;
                let value_in_db_units = (value_in_real_units / db_unit).round() as i32;
                Self::Integer {
                    value: value_in_db_units,
                    db_unit: *db_unit,
                }
            }
        }
    }

    #[must_use]
    pub fn to_float_unit(&self) -> Self {
        match self {
            Self::Integer { value, db_unit } => {
                let value_in_real_units = f64::from(*value) * db_unit;
                Self::Float {
                    value: value_in_real_units,
                    user_unit: 1.0,
                    db_unit: *db_unit,
                }
            }
            Self::Float { .. } => *self,
        }
    }

    /// Sets the database unit for this Unit.
    /// For Integer variants, this changes the `db_unit`.
    /// For Float variants, this changes the `db_unit`.
    pub const fn set_db_units(&mut self, new_db_unit: f64) {
        match self {
            Self::Integer { db_unit, .. } | Self::Float { db_unit, .. } => *db_unit = new_db_unit,
        }
    }

    /// Sets the user units for this Unit.
    /// For Integer variants, this changes the `db_unit` (integers use `db_unit` as their unit).
    /// For Float variants, this changes the `user_unit`.
    pub const fn set_user_units(&mut self, new_user_unit: f64) {
        match self {
            Self::Integer { db_unit, .. } => *db_unit = new_user_unit,
            Self::Float { user_unit, .. } => *user_unit = new_user_unit,
        }
    }

    /// Returns a copy of this Unit with the specified database unit.
    /// For Integer variants, this changes the `db_unit`.
    /// For Float variants, this changes the `db_unit`.
    #[must_use]
    pub const fn with_db_units(&self, new_db_unit: f64) -> Self {
        match self {
            Self::Integer { value, .. } => Self::Integer {
                value: *value,
                db_unit: new_db_unit,
            },
            Self::Float {
                value, user_unit, ..
            } => Self::Float {
                value: *value,
                user_unit: *user_unit,
                db_unit: new_db_unit,
            },
        }
    }

    /// Returns a copy of this Unit with the specified user unit.
    /// For Integer variants, this changes the `db_unit` (integers use `db_unit` as their unit).
    /// For Float variants, this changes the `user_unit`.
    #[must_use]
    pub const fn with_user_units(&self, new_user_unit: f64) -> Self {
        match self {
            Self::Integer { value, .. } => Self::Integer {
                value: *value,
                db_unit: new_user_unit,
            },
            Self::Float { value, db_unit, .. } => Self::Float {
                value: *value,
                user_unit: new_user_unit,
                db_unit: *db_unit,
            },
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, db_unit } => {
                write!(f, "{value}db (db_unit: {db_unit:.3e})")
            }
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => {
                write!(
                    f,
                    "{value:.6}u (user_unit: {user_unit:.3e}, db_unit: {db_unit:.3e})"
                )
            }
        }
    }
}

use std::ops::{Add, Div, Mul, Sub};

impl Add for Unit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Integer + Integer: use left's db_unit, convert right if needed
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                if db1 == db2 {
                    Self::Integer {
                        value: v1 + v2,
                        db_unit: db1,
                    }
                } else {
                    // Convert v2 to real units, then to db1 units
                    let v2_real = f64::from(v2) * db2;
                    let v2_in_db1 = (v2_real / db1).round() as i32;
                    Self::Integer {
                        value: v1 + v2_in_db1,
                        db_unit: db1,
                    }
                }
            }
            // Float + Float: use left's units, convert right if needed
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    db_unit: _,
                },
            ) => {
                // Convert both to real units for addition
                let v1_real = v1 * u1;
                let v2_real = v2 * u2;
                let sum_real = v1_real + v2_real;
                // Express result in left's user_unit
                let result_value = sum_real / u1;
                Self::Float {
                    value: result_value,
                    user_unit: u1,
                    db_unit: db1,
                }
            }
            // Integer + Float: convert both to Float using left's units
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    ..
                },
            ) => {
                let v1_real = f64::from(v1) * db1;
                let v2_real = v2 * u2;
                Self::Float {
                    value: v1_real + v2_real,
                    user_unit: 1.0,
                    db_unit: db1,
                }
            }
            // Float + Integer: convert Integer to Float using left's units
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                let v2_real = f64::from(v2) * db2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 + v2_in_u1,
                    user_unit: u1,
                    db_unit: db1,
                }
            }
        }
    }
}

impl Sub for Unit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Integer - Integer: use left's db_unit, convert right if needed
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                if db1 == db2 {
                    Self::Integer {
                        value: v1 - v2,
                        db_unit: db1,
                    }
                } else {
                    // Convert v2 to real units, then to db1 units
                    let v2_real = f64::from(v2) * db2;
                    let v2_in_db1 = (v2_real / db1).round() as i32;
                    Self::Integer {
                        value: v1 - v2_in_db1,
                        db_unit: db1,
                    }
                }
            }
            // Float - Float: use left's units, convert right if needed
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    db_unit: _,
                },
            ) => {
                // Convert both to real units for subtraction
                let v1_real = v1 * u1;
                let v2_real = v2 * u2;
                let diff_real = v1_real - v2_real;
                // Express result in left's user_unit
                let result_value = diff_real / u1;
                Self::Float {
                    value: result_value,
                    user_unit: u1,
                    db_unit: db1,
                }
            }
            // Integer - Float: convert both to Float using left's units
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    ..
                },
            ) => {
                let v1_real = f64::from(v1) * db1;
                let v2_real = v2 * u2;
                Self::Float {
                    value: v1_real - v2_real,
                    user_unit: 1.0,
                    db_unit: db1,
                }
            }
            // Float - Integer: convert Integer to Float using left's units
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                let v2_real = f64::from(v2) * db2;
                let v2_in_u1 = v2_real / u1;
                Self::Float {
                    value: v1 - v2_in_u1,
                    user_unit: u1,
                    db_unit: db1,
                }
            }
        }
    }
}

impl Mul<f64> for Unit {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Float {
                value: f64::from(value) * scalar,
                user_unit: 1.0,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value * scalar,
                user_unit,
                db_unit,
            },
        }
    }
}

impl Mul<i32> for Unit {
    type Output = Self;

    fn mul(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Integer {
                value: value * scalar,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value * f64::from(scalar),
                user_unit,
                db_unit,
            },
        }
    }
}

impl Div<f64> for Unit {
    type Output = Self;

    fn div(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Float {
                value: f64::from(value) / scalar,
                user_unit: 1.0,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value / scalar,
                user_unit,
                db_unit,
            },
        }
    }
}

impl Div<i32> for Unit {
    type Output = Self;

    fn div(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Float {
                value: f64::from(value) / f64::from(scalar),
                user_unit: 1.0,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value / f64::from(scalar),
                user_unit,
                db_unit,
            },
        }
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        // Convert both to real-world values and compare
        let self_real = match self {
            Self::Integer { value, db_unit } => f64::from(*value) * db_unit,
            Self::Float {
                value,
                user_unit,
                db_unit: _,
            } => value * user_unit,
        };

        let other_real = match other {
            Self::Integer { value, db_unit } => f64::from(*value) * db_unit,
            Self::Float {
                value,
                user_unit,
                db_unit: _,
            } => value * user_unit,
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

impl From<f64> for Unit {
    fn from(value: f64) -> Self {
        Self::default_float(value)
    }
}

impl Sub<i32> for Unit {
    type Output = Self;

    fn sub(self, scalar: i32) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Integer {
                value: value - scalar,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value - f64::from(scalar),
                user_unit,
                db_unit,
            },
        }
    }
}

impl Sub<f64> for Unit {
    type Output = Self;

    fn sub(self, scalar: f64) -> Self::Output {
        match self {
            Self::Integer { value, db_unit } => Self::Float {
                value: f64::from(value) - scalar,
                user_unit: 1.0,
                db_unit,
            },
            Self::Float {
                value,
                user_unit,
                db_unit,
            } => Self::Float {
                value: value - scalar,
                user_unit,
                db_unit,
            },
        }
    }
}

impl Mul for Unit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                let real1 = f64::from(v1) * db1;
                let real2 = f64::from(v2) * db2;
                Self::Float {
                    value: real1 * real2,
                    user_unit: 1.0,
                    db_unit: db1 * db2,
                }
            }
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    db_unit: db2,
                },
            ) => {
                let real1 = v1 * u1;
                let real2 = v2 * u2;
                Self::Float {
                    value: real1 * real2,
                    user_unit: 1.0,
                    db_unit: db1 * db2,
                }
            }
            (
                Self::Integer {
                    value: v1,
                    db_unit: db1,
                },
                Self::Float {
                    value: v2,
                    user_unit: u2,
                    db_unit: db2,
                },
            ) => {
                let real1 = f64::from(v1) * db1;
                let real2 = v2 * u2;
                Self::Float {
                    value: real1 * real2,
                    user_unit: 1.0,
                    db_unit: db1 * db2,
                }
            }
            (
                Self::Float {
                    value: v1,
                    user_unit: u1,
                    db_unit: db1,
                },
                Self::Integer {
                    value: v2,
                    db_unit: db2,
                },
            ) => {
                let real1 = v1 * u1;
                let real2 = f64::from(v2) * db2;
                Self::Float {
                    value: real1 * real2,
                    user_unit: 1.0,
                    db_unit: db1 * db2,
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
            let db_exponent = (i32::arbitrary(g) % 10) - 12;
            let db_unit = 10_f64.powi(db_exponent);

            if bool::arbitrary(g) {
                let value = (i32::arbitrary(g) % 1_000_000) - 500_000;
                Self::Integer { value, db_unit }
            } else {
                let user_exponent = (i32::arbitrary(g) % 10) - 12;
                let user_unit = 10_f64.powi(user_exponent);
                let mut value = f64::arbitrary(g);
                if !value.is_finite() || value.abs() > 1e100 {
                    value = f64::from(i32::arbitrary(g) % 1_000_000) / 1000.0;
                }
                Self::Float {
                    value,
                    user_unit,
                    db_unit,
                }
            }
        }
    }

    mod creation {
        use super::*;

        #[test]
        fn integer() {
            let unit = Unit::integer(100, 0.001);
            match unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 0.001);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float() {
            let unit = Unit::float(100.5, 1.0, 0.001);
            match unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 100.5);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 0.001);
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
            let a = Unit::float(100.5, 1e-6, 1e-9);
            let b = Unit::float(100.5, 1e-6, 1e-9);
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
            let a = Unit::float(1000.0, 1e-9, 1e-12); // 1000 * 1e-9 = 1e-6
            let b = Unit::float(1.0, 1e-6, 1e-12); // 1 * 1e-6 = 1e-6
            assert_eq!(a, b);
        }

        #[test]
        fn integer_and_float_equal() {
            let a = Unit::integer(1000, 1e-9); // 1000 * 1e-9 = 1e-6
            let b = Unit::float(1.0, 1e-6, 1e-9); // 1 * 1e-6 = 1e-6
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
            assert_eq!(unit, Unit::float(1.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_negative() {
            let unit = Unit::from(-3.5);
            assert_eq!(unit, Unit::float(-3.5, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_zero() {
            let unit = Unit::from(0.0);
            assert_eq!(unit, Unit::float(0.0, 1e-6, 1e-9));
        }

        #[test]
        fn from_f64_large() {
            let unit = Unit::from(1000.0);
            assert_eq!(unit, Unit::float(1000.0, 1e-6, 1e-9));
        }

        #[test]
        fn into_with_type_inference() {
            let unit: Unit = 100.into();
            assert_eq!(unit, Unit::integer(100, 1e-9));

            let unit: Unit = 1.5.into();
            assert_eq!(unit, Unit::float(1.5, 1e-6, 1e-9));
        }
    }

    mod unit_setters {
        use super::*;

        #[test]
        fn set_db_units_integer() {
            let mut unit = Unit::integer(100, 1e-9);
            unit.set_db_units(1e-6);

            match unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn set_db_units_float() {
            let mut unit = Unit::float(1.5, 1e-6, 1e-9);
            unit.set_db_units(1e-12);

            match unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn set_user_units_integer() {
            let mut unit = Unit::integer(100, 1e-9);
            unit.set_user_units(1e-6);

            match unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn set_user_units_float() {
            let mut unit = Unit::float(1.5, 1e-6, 1e-9);
            unit.set_user_units(1e-3);

            match unit {
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
        }

        #[test]
        fn with_db_units_integer() {
            let unit = Unit::integer(100, 1e-9);
            let new_unit = unit.with_db_units(1e-6);

            // Original should be unchanged
            match unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }

            // New unit should have new db_unit
            match new_unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn with_db_units_float() {
            let unit = Unit::float(1.5, 1e-6, 1e-9);
            let new_unit = unit.with_db_units(1e-12);

            // Original should be unchanged
            match unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }

            // New unit should have new db_unit
            match new_unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn with_user_units_integer() {
            let unit = Unit::integer(100, 1e-9);
            let new_unit = unit.with_user_units(1e-6);

            // Original should be unchanged
            match unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }

            // New unit should have new db_unit
            match new_unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn with_user_units_float() {
            let unit = Unit::float(1.5, 1e-6, 1e-9);
            let new_unit = unit.with_user_units(1e-3);

            // Original should be unchanged
            match unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 1.5);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }

            // New unit should have new user_unit
            match new_unit {
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
        }

        #[test]
        fn chaining_with_methods() {
            let unit = Unit::integer(100, 1e-9);
            let new_unit = unit.with_db_units(1e-6).with_user_units(1e-3);

            match new_unit {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-3);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn set_methods_modify_in_place() {
            let mut unit = Unit::float(2.5, 1e-6, 1e-9);
            unit.set_user_units(1e-3);
            unit.set_db_units(1e-12);

            match unit {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 2.5);
                    assert_eq!(user_unit, 1e-3);
                    assert_eq!(db_unit, 1e-12);
                }
                _ => panic!("Expected Float variant"),
            }
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn to_integer_from_integer() {
            let unit = Unit::integer(100, 0.001);
            let result = unit.to_integer_unit();
            assert_eq!(result, unit);
        }

        #[test]
        fn to_integer_from_float() {
            let unit = Unit::float(1.007, 1.0, 0.001);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 1007);
                    assert_eq!(db_unit, 0.001);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_integer_with_user_unit() {
            let unit = Unit::float(100.0, 0.001, 1e-6);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100_000);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn to_float_from_float() {
            let unit = Unit::float(100.5, 1e-6, 1e-9);
            let result = unit.to_float_unit();
            assert_eq!(result, unit);
        }

        #[test]
        fn to_float_from_integer() {
            let unit = Unit::integer(100, 1e-9);
            let result = unit.to_float_unit();

            match result {
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
        }

        #[test]
        fn roundtrip() {
            let original = Unit::integer(100, 1e-9);
            let as_float = original.to_float_unit();
            let back_to_int = as_float.to_integer_unit();

            match back_to_int {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn negative_values() {
            let unit = Unit::float(-100.5, 1e-6, 1e-9);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, -100_500);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn zero_value() {
            let unit = Unit::float(0.0, 1e-6, 1e-9);
            let result = unit.to_integer_unit();

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 0);
                    assert_eq!(db_unit, 1e-9);
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
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 150);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6, 1e-9);
            let u2 = Unit::float(50.3, 1e-6, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 150.8).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_db_units() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::integer(50, 1e-6);
            let result = u1 + u2;

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 50100);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(100, 1e-9);
            let u2 = Unit::float(50.5, 1e-6, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 5.06e-5).abs() < 1e-10);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(50.5, 1e-6, 1e-9);
            let u2 = Unit::integer(100, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 50.6).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_user_units() {
            let u1 = Unit::float(100.0, 1e-6, 1e-9);
            let u2 = Unit::float(50.0, 1e-3, 1e-9);
            let result = u1 + u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 50100.0).abs() < 1e-6);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
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
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 70);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(100.5, 1e-6, 1e-9);
            let u2 = Unit::float(50.3, 1e-6, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 50.2).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn different_db_units() {
            let u1 = Unit::integer(100, 1e-6);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 100);
                    assert_eq!(db_unit, 1e-6);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(100.5, 1e-6, 1e-9);
            let u2 = Unit::integer(50, 1e-9);
            let result = u1 - u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 100.45).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_by_i32() {
            let u = Unit::integer(100, 1e-9);
            let result = u - 30;

            match result {
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 70);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn float_by_i32() {
            let u = Unit::float(100.5, 1e-6, 1e-9);
            let result = u - 10;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 90.5).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u - 25.5;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 74.5).abs() < 1e-10);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.5, 1e-6, 1e-9);
            let result = u - 25.5;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 75.0).abs() < 1e-10);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
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
                Unit::Integer { value, db_unit } => {
                    assert_eq!(value, 300);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Integer variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u * 2.5;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 250.0);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.5, 1e-6, 1e-9);
            let result = u * 2.0;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 201.0);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
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
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 5e-8).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert!((db_unit - 1e-9).abs() < 1e-18);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn floats() {
            let u1 = Unit::float(2.0, 1e-3, 1e-6);
            let u2 = Unit::float(3.0, 1e-3, 1e-6);
            let result = u1 * u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 6e-6).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert!((db_unit - 1e-12).abs() < 1e-21);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_and_float() {
            let u1 = Unit::integer(10, 1e-3);
            let u2 = Unit::float(5.0, 1e-3, 1e-6);
            let result = u1 * u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 5e-5).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert!((db_unit - 1e-9).abs() < 1e-18);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_and_integer() {
            let u1 = Unit::float(5.0, 1e-3, 1e-6);
            let u2 = Unit::integer(10, 1e-3);
            let result = u1 * u2;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert!((value - 5e-5).abs() < 1e-15);
                    assert_eq!(user_unit, 1.0);
                    assert!((db_unit - 1e-9).abs() < 1e-18);
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
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 25.0);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn integer_by_f64() {
            let u = Unit::integer(100, 1e-9);
            let result = u / 2.5;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 40.0);
                    assert_eq!(user_unit, 1.0);
                    assert_eq!(db_unit, 1e-9);
                }
                _ => panic!("Expected Float variant"),
            }
        }

        #[test]
        fn float_by_f64() {
            let u = Unit::float(100.0, 1e-6, 1e-9);
            let result = u / 2.0;

            match result {
                Unit::Float {
                    value,
                    user_unit,
                    db_unit,
                } => {
                    assert_eq!(value, 50.0);
                    assert_eq!(user_unit, 1e-6);
                    assert_eq!(db_unit, 1e-9);
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
            Unit::Integer { value, db_unit } => {
                assert_eq!(value, 300);
                assert_eq!(db_unit, 1e-9);
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
                    Unit::Integer { value, db_unit } => {
                        let as_float = unit.to_float_unit();
                        let back = as_float.to_integer_unit();

                        match back {
                            Unit::Integer {
                                value: back_value,
                                db_unit: back_db,
                            } => TestResult::from_bool(
                                value == back_value && (db_unit - back_db).abs() < 1e-10,
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
                    Unit::Integer { value, db_unit } => {
                        let as_float = unit.to_float_unit();
                        if let Unit::Float {
                            value: float_value,
                            user_unit,
                            ..
                        } = as_float
                        {
                            let original_real = f64::from(value) * db_unit;
                            let float_real = float_value * user_unit;
                            (original_real - float_real).abs() < 1e-10
                        } else {
                            false
                        }
                    }
                    Unit::Float { .. } => unit.to_float_unit() == unit,
                }
            }

            #[quickcheck]
            fn preserves_sign(unit: Unit) -> TestResult {
                match unit {
                    Unit::Float {
                        value, user_unit, ..
                    } => {
                        if !value.is_finite() {
                            return TestResult::discard();
                        }

                        let result = unit.to_integer_unit();
                        match result {
                            Unit::Integer {
                                value: int_value, ..
                            } => {
                                let real_value = value * user_unit;
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
                    Unit::Integer { db_unit, .. } => Unit::integer(0, db_unit),
                    Unit::Float {
                        user_unit, db_unit, ..
                    } => Unit::float(0.0, user_unit, db_unit),
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

            #[quickcheck]
            fn self_is_zero(unit: Unit) -> bool {
                let result = unit - unit;
                match result {
                    Unit::Integer { value, .. } => value == 0,
                    Unit::Float { value, .. } => value.abs() < 1e-10,
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
                            db_unit: db1,
                        },
                        Unit::Integer {
                            value: v2,
                            db_unit: db2,
                        },
                    ) => v1 == v2 && (db1 - db2).abs() < 1e-10,
                    (
                        Unit::Float {
                            value: v1,
                            user_unit: u1,
                            db_unit: db1,
                        },
                        Unit::Float {
                            value: v2,
                            user_unit: u2,
                            db_unit: db2,
                        },
                    ) => {
                        (v1 - v2).abs() < 1e-10
                            && (u1 - u2).abs() < 1e-10
                            && (db1 - db2).abs() < 1e-10
                    }
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

            #[quickcheck]
            fn mul_inverse(unit: Unit, scalar: i32) -> TestResult {
                if scalar == 0 {
                    return TestResult::discard();
                }

                let divided = unit / scalar;
                let back = divided * scalar;

                match (unit, back) {
                    (Unit::Integer { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        if !v2.is_finite() {
                            return TestResult::failed();
                        }
                        let diff = (f64::from(v1) - v2).abs();
                        let rel_error = if v1.abs() > 0 {
                            diff / f64::from(v1).abs()
                        } else {
                            diff
                        };
                        TestResult::from_bool(rel_error < 1e-6 || diff < 1e-6)
                    }
                    (Unit::Float { value: v1, .. }, Unit::Float { value: v2, .. }) => {
                        if !v1.is_finite() || !v2.is_finite() {
                            return TestResult::from_bool(v1.is_finite() == v2.is_finite());
                        }
                        let diff = (v1 - v2).abs();
                        let rel_error = if v1.abs() > 1e-10 {
                            diff / v1.abs()
                        } else {
                            diff
                        };
                        TestResult::from_bool(rel_error < 1e-6 || diff < 1e-6)
                    }
                    _ => TestResult::failed(),
                }
            }
        }
    }
}
