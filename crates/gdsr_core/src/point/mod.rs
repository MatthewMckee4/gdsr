use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use num_traits::Zero;

use crate::{AngleInRadians, CoordinateUnit, DatabaseFloatUnit, DatabaseIntegerUnit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T: CoordinateUnit> {
    x: T,
    y: T,
}

pub type DbPoint = Point<DatabaseIntegerUnit>;
pub type FloatPoint = Point<DatabaseFloatUnit>;

impl<T: CoordinateUnit> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    pub fn to_db_point(self) -> DbPoint {
        DbPoint {
            x: DatabaseIntegerUnit::from_db_value(self.x.to_db_value()),
            y: DatabaseIntegerUnit::from_db_value(self.y.to_db_value()),
        }
    }

    pub fn to_float_point(self) -> FloatPoint {
        FloatPoint {
            x: DatabaseFloatUnit::from_db_value(self.x.to_db_value()),
            y: DatabaseFloatUnit::from_db_value(self.y.to_db_value()),
        }
    }

    pub fn rotate(&self, angle: AngleInRadians) -> Self {
        let (sin, cos) = angle.0.sin_cos();
        let x = self.x * cos - self.y * sin;
        let y = self.x * sin + self.y * cos;
        Self { x, y }
    }

    pub fn rotate_around_point<U: CoordinateUnit>(
        &self,
        angle: AngleInRadians,
        center: Point<U>,
    ) -> Self {
        let center_t = Point::new(
            T::from_float_value(center.x.to_float_value()),
            T::from_float_value(center.y.to_float_value()),
        );
        let translated = *self - center_t;
        let rotated = translated.rotate(angle);
        rotated + center_t
    }
}

impl<T: CoordinateUnit> Zero for Point<T> {
    fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        todo!()
    }
}

impl DbPoint {
    pub fn from_coords(x: i32, y: i32) -> Self {
        Self {
            x: DatabaseIntegerUnit(x),
            y: DatabaseIntegerUnit(y),
        }
    }
}

impl FloatPoint {
    pub fn from_coords(x: f64, y: f64) -> Self {
        Self {
            x: DatabaseFloatUnit(x),
            y: DatabaseFloatUnit(y),
        }
    }
}

impl Add<FloatPoint> for DbPoint {
    type Output = FloatPoint;

    fn add(self, other: FloatPoint) -> FloatPoint {
        FloatPoint {
            x: self.x.to_float_units() + other.x,
            y: self.y.to_float_units() + other.y,
        }
    }
}

impl Add<DbPoint> for FloatPoint {
    type Output = FloatPoint;

    fn add(self, other: DbPoint) -> FloatPoint {
        FloatPoint {
            x: self.x + other.x.to_float_units(),
            y: self.y + other.y.to_float_units(),
        }
    }
}

impl<T: CoordinateUnit> Add<Point<T>> for Point<T> {
    type Output = Point<T>;

    fn add(self, other: Point<T>) -> Point<T> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub<FloatPoint> for DbPoint {
    type Output = FloatPoint;

    fn sub(self, other: FloatPoint) -> FloatPoint {
        FloatPoint {
            x: self.x.to_float_units() - other.x,
            y: self.y.to_float_units() - other.y,
        }
    }
}

impl Sub<DbPoint> for FloatPoint {
    type Output = FloatPoint;

    fn sub(self, other: DbPoint) -> FloatPoint {
        FloatPoint {
            x: self.x - other.x.to_float_units(),
            y: self.y - other.y.to_float_units(),
        }
    }
}

impl<T: CoordinateUnit> Sub<Point<T>> for Point<T> {
    type Output = Point<T>;

    fn sub(self, other: Point<T>) -> Point<T> {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Div<FloatPoint> for DbPoint {
    type Output = FloatPoint;

    fn div(self, other: FloatPoint) -> FloatPoint {
        FloatPoint {
            x: self.x.to_float_units() / other.x,
            y: self.y.to_float_units() / other.y,
        }
    }
}

impl Div<DbPoint> for FloatPoint {
    type Output = FloatPoint;

    fn div(self, other: DbPoint) -> FloatPoint {
        FloatPoint {
            x: self.x / other.x.to_float_units(),
            y: self.y / other.y.to_float_units(),
        }
    }
}

impl<T: CoordinateUnit> Mul<T> for Point<T> {
    type Output = Point<T>;

    fn mul(self, scalar: T) -> Point<T> {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: CoordinateUnit> Div<T> for Point<T> {
    type Output = Point<T>;

    fn div(self, scalar: T) -> Point<T> {
        Point {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl<T: CoordinateUnit> AddAssign<Point<T>> for Point<T> {
    fn add_assign(&mut self, other: Point<T>) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
    }
}

impl AddAssign<FloatPoint> for DbPoint {
    fn add_assign(&mut self, other: FloatPoint) {
        *self = (*self + other).to_db_point();
    }
}

impl AddAssign<DbPoint> for FloatPoint {
    fn add_assign(&mut self, other: DbPoint) {
        self.x = self.x + other.x.to_float_units();
        self.y = self.y + other.y.to_float_units();
    }
}

impl<T: CoordinateUnit> SubAssign<Point<T>> for Point<T> {
    fn sub_assign(&mut self, other: Point<T>) {
        self.x = self.x - other.x;
        self.y = self.y - other.y;
    }
}

impl SubAssign<FloatPoint> for DbPoint {
    fn sub_assign(&mut self, other: FloatPoint) {
        *self = (*self - other).to_db_point();
    }
}

impl SubAssign<DbPoint> for FloatPoint {
    fn sub_assign(&mut self, other: DbPoint) {
        self.x = self.x - other.x.to_float_units();
        self.y = self.y - other.y.to_float_units();
    }
}

impl DivAssign<FloatPoint> for DbPoint {
    fn div_assign(&mut self, other: FloatPoint) {
        *self = (*self / other).to_db_point();
    }
}

impl DivAssign<DbPoint> for FloatPoint {
    fn div_assign(&mut self, other: DbPoint) {
        self.x = self.x / other.x.to_float_units();
        self.y = self.y / other.y.to_float_units();
    }
}

impl<T: CoordinateUnit> MulAssign<T> for Point<T> {
    fn mul_assign(&mut self, scalar: T) {
        self.x = self.x * scalar;
        self.y = self.y * scalar;
    }
}

impl<T: CoordinateUnit> DivAssign<T> for Point<T> {
    fn div_assign(&mut self, scalar: T) {
        self.x = self.x / scalar;
        self.y = self.y / scalar;
    }
}

impl<T: CoordinateUnit> Mul<u32> for Point<T> {
    type Output = Point<T>;

    fn mul(self, rhs: u32) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: CoordinateUnit> Mul<f64> for Point<T> {
    type Output = Point<T>;

    fn mul(self, scalar: f64) -> Self::Output {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<T: CoordinateUnit> From<geo::Point<f64>> for Point<T> {
    fn from(point: geo::Point<f64>) -> Self {
        Point {
            x: T::from_float_value(point.x()),
            y: T::from_float_value(point.y()),
        }
    }
}

impl<T: CoordinateUnit> From<Point<T>> for geo::Point<f64> {
    fn from(point: Point<T>) -> Self {
        geo::Point::new(point.x().to_float_value(), point.y().to_float_value())
    }
}

impl<T: CoordinateUnit> From<&Point<T>> for geo::Coord<f64> {
    fn from(point: &Point<T>) -> Self {
        geo::Coord {
            x: point.x().to_float_value(),
            y: point.y().to_float_value(),
        }
    }
}
