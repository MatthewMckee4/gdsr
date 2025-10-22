use core::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_traits::{MulAdd, Zero};

pub trait CoordinateUnit:
    Copy
    + PartialOrd
    + Debug
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<u32, Output = Self>
    + Mul<f64, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + Zero
    + MulAdd<Self, Self, Output = Self>
    + Neg
{
    fn to_db_value(self) -> i32;
    fn to_db_units(self) -> DatabaseIntegerUnit {
        DatabaseIntegerUnit(self.to_db_value())
    }
    fn from_db_value(db_units: i32) -> Self;

    fn to_float_value(self) -> f64;
    fn to_float_units(self) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.to_float_value())
    }
    fn from_float_value(float_units: f64) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct DatabaseIntegerUnit(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DatabaseFloatUnit(pub f64);

impl CoordinateUnit for DatabaseIntegerUnit {
    fn to_db_value(self) -> i32 {
        self.0
    }

    fn from_db_value(db_units: i32) -> Self {
        Self(db_units)
    }

    fn to_float_value(self) -> f64 {
        self.0 as f64
    }

    fn from_float_value(float_units: f64) -> Self {
        Self(float_units as i32)
    }
}

impl CoordinateUnit for DatabaseFloatUnit {
    fn to_db_value(self) -> i32 {
        self.0.round() as i32
    }

    fn from_db_value(db_units: i32) -> Self {
        Self(f64::from(db_units))
    }

    fn to_float_value(self) -> f64 {
        self.0
    }

    fn from_float_value(float_units: f64) -> Self {
        Self(float_units)
    }
}

impl Add for DatabaseIntegerUnit {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Add for DatabaseFloatUnit {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Add<DatabaseFloatUnit> for DatabaseIntegerUnit {
    type Output = DatabaseFloatUnit;

    fn add(self, other: DatabaseFloatUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 as f64 + other.0)
    }
}

impl Add<DatabaseIntegerUnit> for DatabaseFloatUnit {
    type Output = DatabaseFloatUnit;

    fn add(self, other: DatabaseIntegerUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 + other.0 as f64)
    }
}

impl Sub for DatabaseIntegerUnit {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Sub for DatabaseFloatUnit {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Sub<DatabaseFloatUnit> for DatabaseIntegerUnit {
    type Output = DatabaseFloatUnit;

    fn sub(self, other: DatabaseFloatUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 as f64 - other.0)
    }
}

impl Sub<DatabaseIntegerUnit> for DatabaseFloatUnit {
    type Output = DatabaseFloatUnit;

    fn sub(self, other: DatabaseIntegerUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 - other.0 as f64)
    }
}

impl Mul for DatabaseIntegerUnit {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl Mul for DatabaseFloatUnit {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

impl Mul<DatabaseFloatUnit> for DatabaseIntegerUnit {
    type Output = DatabaseFloatUnit;

    fn mul(self, other: DatabaseFloatUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 as f64 * other.0)
    }
}

impl Mul<DatabaseIntegerUnit> for DatabaseFloatUnit {
    type Output = DatabaseFloatUnit;

    fn mul(self, other: DatabaseIntegerUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 * other.0 as f64)
    }
}

impl Div for DatabaseIntegerUnit {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl Div for DatabaseFloatUnit {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self(self.0 / other.0)
    }
}

impl Div<DatabaseFloatUnit> for DatabaseIntegerUnit {
    type Output = DatabaseFloatUnit;

    fn div(self, other: DatabaseFloatUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 as f64 / other.0)
    }
}

impl Div<DatabaseIntegerUnit> for DatabaseFloatUnit {
    type Output = DatabaseFloatUnit;

    fn div(self, other: DatabaseIntegerUnit) -> DatabaseFloatUnit {
        DatabaseFloatUnit(self.0 / other.0 as f64)
    }
}

impl AddAssign for DatabaseIntegerUnit {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl AddAssign for DatabaseFloatUnit {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl AddAssign<DatabaseFloatUnit> for DatabaseIntegerUnit {
    fn add_assign(&mut self, other: DatabaseFloatUnit) {
        *self = Self((self.0 as f64 + other.0) as i32);
    }
}

impl AddAssign<DatabaseIntegerUnit> for DatabaseFloatUnit {
    fn add_assign(&mut self, other: DatabaseIntegerUnit) {
        self.0 += other.0 as f64;
    }
}

impl SubAssign for DatabaseIntegerUnit {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl SubAssign for DatabaseFloatUnit {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl SubAssign<DatabaseFloatUnit> for DatabaseIntegerUnit {
    fn sub_assign(&mut self, other: DatabaseFloatUnit) {
        *self = Self((self.0 as f64 - other.0) as i32);
    }
}

impl SubAssign<DatabaseIntegerUnit> for DatabaseFloatUnit {
    fn sub_assign(&mut self, other: DatabaseIntegerUnit) {
        self.0 -= other.0 as f64;
    }
}

impl MulAssign for DatabaseIntegerUnit {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl MulAssign for DatabaseFloatUnit {
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl MulAssign<DatabaseFloatUnit> for DatabaseIntegerUnit {
    fn mul_assign(&mut self, other: DatabaseFloatUnit) {
        *self = Self((self.0 as f64 * other.0) as i32);
    }
}

impl MulAssign<DatabaseIntegerUnit> for DatabaseFloatUnit {
    fn mul_assign(&mut self, other: DatabaseIntegerUnit) {
        self.0 *= other.0 as f64;
    }
}

impl DivAssign for DatabaseIntegerUnit {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}

impl DivAssign for DatabaseFloatUnit {
    fn div_assign(&mut self, other: Self) {
        self.0 /= other.0;
    }
}

impl DivAssign<DatabaseFloatUnit> for DatabaseIntegerUnit {
    fn div_assign(&mut self, other: DatabaseFloatUnit) {
        *self = Self((self.0 as f64 / other.0) as i32);
    }
}

impl DivAssign<DatabaseIntegerUnit> for DatabaseFloatUnit {
    fn div_assign(&mut self, other: DatabaseIntegerUnit) {
        self.0 /= other.0 as f64;
    }
}

impl Mul<u32> for DatabaseIntegerUnit {
    type Output = Self;

    fn mul(self, scalar: u32) -> Self {
        Self(self.0 * scalar as i32)
    }
}

impl Mul<u32> for DatabaseFloatUnit {
    type Output = Self;

    fn mul(self, scalar: u32) -> Self {
        Self(self.0 * scalar as f64)
    }
}

impl Mul<f64> for DatabaseIntegerUnit {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self((self.0 as f64 * scalar) as i32)
    }
}

impl Mul<f64> for DatabaseFloatUnit {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self(self.0 * scalar)
    }
}

impl Zero for DatabaseIntegerUnit {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Zero for DatabaseFloatUnit {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl MulAdd<Self, Self> for DatabaseIntegerUnit {
    type Output = Self;

    fn mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0 * a.0 + b.0)
    }
}

impl MulAdd<Self, Self> for DatabaseFloatUnit {
    type Output = Self;

    fn mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0 * a.0 + b.0)
    }
}

impl Neg for DatabaseIntegerUnit {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Neg for DatabaseFloatUnit {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}
