use crate::transformation::{Reflection, Rotation, Scale, Transformation, Translation};
use crate::{AngleInRadians, Point};

pub trait ToGds {
    fn to_gds_impl(&self, buffer: &mut impl std::io::Write, scale: f64) -> std::io::Result<()>;
}

pub trait Transformable: Sized {
    #[must_use]
    fn transform(self, transformation: impl Into<Transformation>) -> Self {
        self.transform_impl(&transformation.into())
    }

    #[must_use]
    fn transform_impl(self, transformation: &Transformation) -> Self;

    #[must_use]
    fn rotate(self, angle: AngleInRadians, centre: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_rotation(Some(Rotation::new(angle, centre))),
        )
    }

    #[must_use]
    fn scale(self, factor: f64, centre: Point) -> Self {
        self.transform_impl(Transformation::default().with_scale(Some(Scale::new(factor, centre))))
    }

    #[must_use]
    fn reflect(self, angle: f64, centre: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_reflection(Some(Reflection::new(angle, centre))),
        )
    }

    #[must_use]
    fn translate(self, delta: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }
}

pub trait Movable: Transformable {
    #[must_use]
    fn move_by(self, delta: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }

    #[must_use]
    fn move_to(self, target: Point) -> Self;
}

pub trait Dimensions {
    fn bounding_box(&self) -> (Point, Point);
}
