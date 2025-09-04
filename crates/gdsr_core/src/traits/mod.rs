use std::fs::File;
use std::io;

use crate::{
    CoordNum, DatabaseIntegerUnit, Point,
    transformation::{Reflection, Rotation, Scale, Transformation, Translation},
};

pub trait ToGds {
    fn _to_gds(&self, file: &mut File, scale: f64) -> io::Result<()>;
}

pub trait Transformable: Sized {
    fn transform(&self, transformation: &Transformation) -> Self;

    fn rotate(&self, angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform(&Transformation::default().with_rotation(Some(Rotation::new(angle, centre))))
    }

    fn scale(&self, factor: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform(&Transformation::default().with_scale(Some(Scale::new(factor, centre))))
    }

    fn reflect(&self, angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform(
            &Transformation::default().with_reflection(Some(Reflection::new(angle, centre))),
        )
    }

    fn translate(&self, delta: Point<DatabaseIntegerUnit>) -> Self {
        self.transform(&Transformation::default().with_translation(Some(Translation::new(delta))))
    }
}

pub trait Movable: Transformable {
    fn move_by(&self, delta: Point<DatabaseIntegerUnit>) -> Self {
        self.transform(&Transformation::default().with_translation(Some(Translation::new(delta))))
    }

    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self;
}

pub trait Dimensions<T: CoordNum> {
    fn bounding_box(&self) -> (Point<T>, Point<T>);
}
