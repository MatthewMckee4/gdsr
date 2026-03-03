use crate::error::GdsError;
use crate::transformation::{Reflection, Rotation, Scale, Transformation, Translation};
use crate::{AngleInRadians, Point};

/// Trait for types that can be serialized to the GDSII binary format.
pub trait ToGds {
    /// Writes the GDSII binary representation to the given buffer.
    fn to_gds_impl(&self, buffer: &mut impl std::io::Write, scale: f64) -> Result<(), GdsError>;
}

/// Trait for types that can be geometrically transformed (rotated, scaled, reflected, translated).
pub trait Transformable: Sized {
    /// Applies a transformation and returns the transformed value.
    #[must_use]
    fn transform(self, transformation: impl Into<Transformation>) -> Self {
        self.transform_impl(&transformation.into())
    }

    /// Applies a transformation reference and returns the transformed value.
    #[must_use]
    fn transform_impl(self, transformation: &Transformation) -> Self;

    /// Rotates by the given angle (in radians) around the centre point.
    #[must_use]
    fn rotate(self, angle: AngleInRadians, centre: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_rotation(Some(Rotation::new(angle, centre))),
        )
    }

    /// Scales by the given factor around the centre point.
    #[must_use]
    fn scale(self, factor: f64, centre: Point) -> Self {
        self.transform_impl(Transformation::default().with_scale(Some(Scale::new(factor, centre))))
    }

    /// Reflects across the axis defined by the given angle (in radians) through the centre point.
    #[must_use]
    fn reflect(self, angle: f64, centre: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_reflection(Some(Reflection::new(angle, centre))),
        )
    }

    /// Translates by the given delta point.
    #[must_use]
    fn translate(self, delta: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }
}

/// Trait for types that can be repositioned by moving to an absolute target or by a relative delta.
pub trait Movable: Transformable {
    /// Moves the value by the given delta (relative translation).
    #[must_use]
    fn move_by(self, delta: Point) -> Self {
        self.transform_impl(
            Transformation::default().with_translation(Some(Translation::new(delta))),
        )
    }

    /// Moves the value to the given absolute target point.
    #[must_use]
    fn move_to(self, target: Point) -> Self;
}

/// Trait for types that have spatial dimensions and a bounding box.
pub trait Dimensions {
    /// Returns the axis-aligned bounding box as `(min_point, max_point)`.
    fn bounding_box(&self) -> (Point, Point);
}
