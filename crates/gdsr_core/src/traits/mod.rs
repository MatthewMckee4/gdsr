use std::{fs::File, io};

use crate::Point;
// TODO: Re-enable transformation imports after transformation module is updated
// use crate::transformation::{Reflection, Rotation, Scale, Transformation, Translation};

pub trait ToGds {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()>;
}

// TODO: Re-enable Transformable trait after transformation module is updated
// pub trait Transformable: Sized {
//     #[must_use]
//     fn transform(&self, transformation: impl Into<Transformation>) -> Self {
//         self.transform_impl(&transformation.into())
//     }
//
//     #[must_use]
//     fn transform_impl(&self, transformation: &Transformation) -> Self;
//
//     #[must_use]
//     fn rotate(&self, angle: f64, centre: Point) -> Self {
//         self.transform_impl(
//             Transformation::default().with_rotation(Some(Rotation::new(angle, centre))),
//         )
//     }
//
//     #[must_use]
//     fn scale(&self, factor: f64, centre: Point) -> Self {
//         self.transform_impl(Transformation::default().with_scale(Some(Scale::new(factor, centre))))
//     }
//
//     #[must_use]
//     fn reflect(&self, angle: f64, centre: Point) -> Self {
//         self.transform_impl(
//             Transformation::default().with_reflection(Some(Reflection::new(angle, centre))),
//         )
//     }
//
//     #[must_use]
//     fn translate(&self, delta: Point) -> Self {
//         self.transform_impl(
//             Transformation::default().with_translation(Some(Translation::new(delta))),
//         )
//     }
// }

// TODO: Re-enable Movable trait after Transformable is updated
// pub trait Movable: Transformable {
//     #[must_use]
//     fn move_by(&self, delta: Point) -> Self {
//         self.transform_impl(
//             Transformation::default().with_translation(Some(Translation::new(delta))),
//         )
//     }
//
//     #[must_use]
//     fn move_to(&self, target: Point) -> Self;
// }

pub trait Dimensions {
    fn bounding_box(&self) -> (Point, Point);
}
