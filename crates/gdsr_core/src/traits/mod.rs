use std::{fs::File, io};

use crate::{
    CoordNum, DatabaseIntegerUnit, Point,
    transformation::{Reflection, Rotation, Scale, Transformation, Translation},
};

pub trait ToGds {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()>;
}

pub trait Transformable: Sized {
    #[must_use]
    fn transform(&self, transformation: impl Into<Transformation>) -> Self {
        self.transform_impl(&transformation.into())
    }

    #[must_use]
    fn transform_impl(&self, transformation: &Transformation) -> Self;

    #[must_use]
    fn rotate(&self, angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform_impl(
            Transformation::default().with_rotation(Some(Rotation::new(angle, centre))),
        )
    }

    #[must_use]
    fn scale(&self, factor: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform_impl(Transformation::default().with_scale(Some(Scale::new(factor, centre))))
    }

    #[must_use]
    fn reflect(&self, angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        self.transform_impl(
            Transformation::default().with_reflection(Some(Reflection::new(angle, centre))),
        )
    }

    #[must_use]
    fn translate(&self, delta: Point<DatabaseIntegerUnit>) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }
}

pub trait Movable: Transformable {
    #[must_use]
    fn move_by(&self, delta: Point<DatabaseIntegerUnit>) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }

    #[must_use]
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self;
}

pub trait Dimensions<T: CoordNum> {
    fn bounding_box(&self) -> (Point<T>, Point<T>);
}
