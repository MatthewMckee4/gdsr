use crate::Point;

mod reflection;
mod rotation;
mod scale;
mod translation;

pub use reflection::Reflection;
pub use rotation::Rotation;
pub use scale::Scale;
pub use translation::Translation;

/// A composite transformation that applies reflection, rotation, scale, and translation in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transformation {
    pub reflection: Option<Reflection>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
    pub translation: Option<Translation>,
}

impl Transformation {
    /// Sets the reflection component and returns `&mut Self` for chaining.
    pub const fn with_reflection(&mut self, reflection: Option<Reflection>) -> &mut Self {
        self.reflection = reflection;
        self
    }

    /// Sets the rotation component and returns `&mut Self` for chaining.
    pub const fn with_rotation(&mut self, rotation: Option<Rotation>) -> &mut Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale component and returns `&mut Self` for chaining.
    pub const fn with_scale(&mut self, scale: Option<Scale>) -> &mut Self {
        self.scale = scale;
        self
    }

    /// Sets the translation component and returns `&mut Self` for chaining.
    pub const fn with_translation(&mut self, translation: Option<Translation>) -> &mut Self {
        self.translation = translation;
        self
    }

    /// Applies all transformation components to a point in order:
    /// reflection, rotation, scale, then translation.
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let mut new_point = *point;

        if let Some(reflection) = &self.reflection {
            new_point = reflection.apply_to_point(&new_point);
        }

        if let Some(rotation) = &self.rotation {
            new_point = rotation.apply_to_point(&new_point);
        }

        if let Some(scale) = &self.scale {
            new_point = scale.apply_to_point(&new_point);
        }

        if let Some(translation) = &self.translation {
            new_point = translation.apply_to_point(&new_point);
        }

        new_point
    }
}

impl From<Reflection> for Transformation {
    fn from(reflection: Reflection) -> Self {
        let mut transformation = Self::default();
        transformation.with_reflection(Some(reflection));
        transformation
    }
}

impl From<Rotation> for Transformation {
    fn from(rotation: Rotation) -> Self {
        let mut transformation = Self::default();
        transformation.with_rotation(Some(rotation));
        transformation
    }
}

impl From<Scale> for Transformation {
    fn from(scale: Scale) -> Self {
        let mut transformation = Self::default();
        transformation.with_scale(Some(scale));
        transformation
    }
}

impl From<Translation> for Transformation {
    fn from(translation: Translation) -> Self {
        let mut transformation = Self::default();
        transformation.with_translation(Some(translation));
        transformation
    }
}

impl From<&Self> for Transformation {
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformation_default() {
        let transformation = Transformation::default();
        assert!(transformation.reflection.is_none());
        assert!(transformation.rotation.is_none());
        assert!(transformation.scale.is_none());
        assert!(transformation.translation.is_none());
    }

    #[test]
    fn test_transformation_with_reflection() {
        let reflection = Reflection::new(0.0, Point::integer(0, 0, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_reflection(Some(reflection.clone()));

        assert!(transformation.reflection.is_some());
        assert_eq!(transformation.reflection.unwrap(), reflection);
    }

    #[test]
    fn test_transformation_with_rotation() {
        let rotation = Rotation::new(45.0, Point::integer(0, 0, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_rotation(Some(rotation.clone()));

        assert!(transformation.rotation.is_some());
        assert_eq!(transformation.rotation.unwrap(), rotation);
    }

    #[test]
    fn test_transformation_with_scale() {
        let scale = Scale::new(2.0, Point::integer(0, 0, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale.clone()));

        assert!(transformation.scale.is_some());
        assert_eq!(transformation.scale.unwrap(), scale);
    }

    #[test]
    fn test_transformation_with_translation() {
        let translation = Translation::new(Point::integer(10, 20, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_translation(Some(translation.clone()));

        assert!(transformation.translation.is_some());
        assert_eq!(transformation.translation.unwrap(), translation);
    }

    #[test]
    fn test_apply_to_point_identity() {
        let transformation = Transformation::default();
        let point = Point::integer(5, 10, 1e-9);
        let result = transformation.apply_to_point(&point);
        assert_eq!(result, point);
    }

    #[test]
    fn test_apply_to_point_translation() {
        let translation = Translation::new(Point::integer(5, 5, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_translation(Some(translation));

        let point = Point::integer(0, 0, 1e-9);
        let result = transformation.apply_to_point(&point);
        assert_eq!(result, Point::integer(5, 5, 1e-9));
    }

    #[test]
    fn test_apply_to_point_scale() {
        let scale = Scale::new(2.0, Point::integer(0, 0, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));

        let point = Point::integer(5, 10, 1e-9);
        let result = transformation.apply_to_point(&point);
        assert_eq!(result, Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_from_reflection() {
        let reflection = Reflection::new(0.0, Point::integer(0, 0, 1e-9));
        let transformation: Transformation = reflection.clone().into();

        assert!(transformation.reflection.is_some());
        assert_eq!(transformation.reflection.unwrap(), reflection);
        assert!(transformation.rotation.is_none());
        assert!(transformation.scale.is_none());
        assert!(transformation.translation.is_none());
    }

    #[test]
    fn test_from_rotation() {
        let rotation = Rotation::new(45.0, Point::integer(0, 0, 1e-9));
        let transformation: Transformation = rotation.clone().into();

        assert!(transformation.rotation.is_some());
        assert_eq!(transformation.rotation.unwrap(), rotation);
        assert!(transformation.reflection.is_none());
        assert!(transformation.scale.is_none());
        assert!(transformation.translation.is_none());
    }

    #[test]
    fn test_from_scale() {
        let scale = Scale::new(2.0, Point::integer(0, 0, 1e-9));
        let transformation: Transformation = scale.clone().into();

        assert!(transformation.scale.is_some());
        assert_eq!(transformation.scale.unwrap(), scale);
        assert!(transformation.reflection.is_none());
        assert!(transformation.rotation.is_none());
        assert!(transformation.translation.is_none());
    }

    #[test]
    fn test_from_translation() {
        let translation = Translation::new(Point::integer(10, 20, 1e-9));
        let transformation: Transformation = translation.clone().into();

        assert!(transformation.translation.is_some());
        assert_eq!(transformation.translation.unwrap(), translation);
        assert!(transformation.reflection.is_none());
        assert!(transformation.rotation.is_none());
        assert!(transformation.scale.is_none());
    }

    fn assert_point_approx_eq(actual: &Point, expected: &Point) {
        assert!(
            (actual.x() - expected.x()).absolute_value().abs() < 1e-6,
            "x mismatch: actual={actual:?}, expected={expected:?}"
        );
        assert!(
            (actual.y() - expected.y()).absolute_value().abs() < 1e-6,
            "y mismatch: actual={actual:?}, expected={expected:?}"
        );
    }

    #[test]
    fn test_rotation_and_scale_combined() {
        let origin = Point::integer(0, 0, 1e-9);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_2, origin);
        let scale = Scale::new(2.0, origin);

        let mut transformation = Transformation::default();
        transformation.with_rotation(Some(rotation));
        transformation.with_scale(Some(scale));

        let point = Point::integer(3, 0, 1e-9);
        let result = transformation.apply_to_point(&point);

        let rotated = Rotation::new(std::f64::consts::FRAC_PI_2, origin).apply_to_point(&point);
        let expected = Scale::new(2.0, origin).apply_to_point(&rotated);
        assert_point_approx_eq(&result, &expected);
        assert_point_approx_eq(&result, &Point::integer(0, 6, 1e-9));
    }

    #[test]
    fn test_rotation_and_translation_combined() {
        let origin = Point::integer(0, 0, 1e-9);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_2, origin);
        let translation = Translation::new(Point::integer(10, 5, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_rotation(Some(rotation));
        transformation.with_translation(Some(translation));

        let point = Point::integer(4, 0, 1e-9);
        let result = transformation.apply_to_point(&point);

        let rotated = Rotation::new(std::f64::consts::FRAC_PI_2, origin).apply_to_point(&point);
        let expected = Translation::new(Point::integer(10, 5, 1e-9)).apply_to_point(&rotated);
        assert_point_approx_eq(&result, &expected);
        assert_point_approx_eq(&result, &Point::integer(10, 9, 1e-9));
    }

    #[test]
    fn test_scale_and_translation_combined() {
        let origin = Point::integer(0, 0, 1e-9);
        let scale = Scale::new(3.0, origin);
        let translation = Translation::new(Point::integer(1, 2, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));
        transformation.with_translation(Some(translation));

        let point = Point::integer(2, 5, 1e-9);
        let result = transformation.apply_to_point(&point);

        let scaled = Scale::new(3.0, origin).apply_to_point(&point);
        let expected = Translation::new(Point::integer(1, 2, 1e-9)).apply_to_point(&scaled);
        assert_point_approx_eq(&result, &expected);
        assert_point_approx_eq(&result, &Point::integer(7, 17, 1e-9));
    }

    #[test]
    fn test_reflection_and_rotation_combined() {
        let origin = Point::integer(0, 0, 1e-9);
        let reflection = Reflection::new(0.0, origin);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_2, origin);

        let mut transformation = Transformation::default();
        transformation.with_reflection(Some(reflection.clone()));
        transformation.with_rotation(Some(rotation.clone()));

        let point = Point::integer(3, 4, 1e-9);
        let result = transformation.apply_to_point(&point);

        let reflected = reflection.apply_to_point(&point);
        let expected = rotation.apply_to_point(&reflected);
        assert_point_approx_eq(&result, &expected);
        assert_point_approx_eq(&reflected, &Point::integer(3, -4, 1e-9));
        assert_point_approx_eq(&result, &Point::integer(4, 3, 1e-9));
    }

    #[test]
    fn test_all_four_transformations_combined() {
        let origin = Point::integer(0, 0, 1e-9);
        let reflection = Reflection::new(0.0, origin);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_2, origin);
        let scale = Scale::new(2.0, origin);
        let translation = Translation::new(Point::integer(10, 10, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_reflection(Some(reflection.clone()));
        transformation.with_rotation(Some(rotation.clone()));
        transformation.with_scale(Some(scale.clone()));
        transformation.with_translation(Some(translation.clone()));

        let point = Point::integer(3, 4, 1e-9);
        let result = transformation.apply_to_point(&point);

        let step1 = reflection.apply_to_point(&point);
        let step2 = rotation.apply_to_point(&step1);
        let step3 = scale.apply_to_point(&step2);
        let expected = translation.apply_to_point(&step3);
        assert_point_approx_eq(&result, &expected);

        assert_point_approx_eq(&step1, &Point::integer(3, -4, 1e-9));
        assert_point_approx_eq(&step2, &Point::integer(4, 3, 1e-9));
        assert_point_approx_eq(&step3, &Point::integer(8, 6, 1e-9));
        assert_point_approx_eq(&result, &Point::integer(18, 16, 1e-9));
    }

    #[test]
    fn test_order_of_operations_reflection_before_rotation() {
        let origin = Point::integer(0, 0, 1e-9);
        let reflection = Reflection::new(0.0, origin);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_2, origin);
        let scale = Scale::new(2.0, origin);
        let translation = Translation::new(Point::integer(5, 5, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_reflection(Some(reflection.clone()));
        transformation.with_rotation(Some(rotation.clone()));
        transformation.with_scale(Some(scale.clone()));
        transformation.with_translation(Some(translation.clone()));

        let point = Point::integer(1, 3, 1e-9);

        let step1 = reflection.apply_to_point(&point);
        let step2 = rotation.apply_to_point(&step1);
        let step3 = scale.apply_to_point(&step2);
        let step4 = translation.apply_to_point(&step3);

        let result = transformation.apply_to_point(&point);
        assert_point_approx_eq(&result, &step4);

        let wrong_step1 = rotation.apply_to_point(&point);
        let wrong_step2 = reflection.apply_to_point(&wrong_step1);
        let wrong_step3 = scale.apply_to_point(&wrong_step2);
        let wrong_result = translation.apply_to_point(&wrong_step3);

        let results_differ = (result.x() - wrong_result.x()).absolute_value().abs() > 1e-12
            || (result.y() - wrong_result.y()).absolute_value().abs() > 1e-12;
        assert!(
            results_differ,
            "Swapping reflection and rotation order should produce different results"
        );
    }

    #[test]
    fn test_scale_by_zero() {
        let origin = Point::integer(0, 0, 1e-9);
        let scale = Scale::new(0.0, origin);
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));

        let point = Point::integer(5, 10, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &Point::integer(0, 0, 1e-9));
    }

    #[test]
    fn test_scale_by_zero_nonorigin_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let scale = Scale::new(0.0, centre);
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));

        let point = Point::integer(5, 20, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &centre);
    }

    #[test]
    fn test_negative_scale() {
        let origin = Point::integer(0, 0, 1e-9);
        let scale = Scale::new(-1.0, origin);
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));

        let point = Point::integer(5, 10, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &Point::integer(-5, -10, 1e-9));
    }

    #[test]
    fn test_negative_scale_with_nonorigin_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let scale = Scale::new(-1.0, centre);
        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));

        let point = Point::integer(15, 20, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &Point::integer(5, 0, 1e-9));
    }

    #[test]
    fn test_rotation_greater_than_2pi() {
        let origin = Point::integer(0, 0, 1e-9);
        let angle = std::f64::consts::TAU + std::f64::consts::FRAC_PI_2;
        let rotation = Rotation::new(angle, origin);
        let mut transformation = Transformation::default();
        transformation.with_rotation(Some(rotation));

        let point = Point::integer(10, 0, 1e-9);
        let result = transformation.apply_to_point(&point);

        let just_90 = Rotation::new(std::f64::consts::FRAC_PI_2, origin);
        let expected = just_90.apply_to_point(&point);
        assert_point_approx_eq(&result, &expected);
    }

    #[test]
    fn test_rotation_negative_angle() {
        let origin = Point::integer(0, 0, 1e-9);
        let rotation = Rotation::new(-std::f64::consts::FRAC_PI_2, origin);
        let mut transformation = Transformation::default();
        transformation.with_rotation(Some(rotation));

        let point = Point::integer(10, 0, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &Point::integer(0, -10, 1e-9));
    }

    #[test]
    fn test_all_transformations_simultaneously() {
        let origin = Point::integer(0, 0, 1e-9);
        let reflection = Reflection::new(0.0, origin);
        let rotation = Rotation::new(std::f64::consts::FRAC_PI_4, origin);
        let scale = Scale::new(3.0, origin);
        let translation = Translation::new(Point::integer(100, 200, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_reflection(Some(reflection.clone()));
        transformation.with_rotation(Some(rotation.clone()));
        transformation.with_scale(Some(scale.clone()));
        transformation.with_translation(Some(translation.clone()));

        let point = Point::integer(10, 5, 1e-9);
        let result = transformation.apply_to_point(&point);

        let step1 = reflection.apply_to_point(&point);
        let step2 = rotation.apply_to_point(&step1);
        let step3 = scale.apply_to_point(&step2);
        let expected = translation.apply_to_point(&step3);

        assert_point_approx_eq(&result, &expected);
    }

    #[test]
    fn test_scale_by_zero_then_translate() {
        let origin = Point::integer(0, 0, 1e-9);
        let scale = Scale::new(0.0, origin);
        let translation = Translation::new(Point::integer(10, 20, 1e-9));

        let mut transformation = Transformation::default();
        transformation.with_scale(Some(scale));
        transformation.with_translation(Some(translation));

        let point = Point::integer(99, 99, 1e-9);
        let result = transformation.apply_to_point(&point);

        assert_point_approx_eq(&result, &Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_from_self_reference() {
        let translation = Translation::new(Point::integer(5, 10, 1e-9));
        let mut original = Transformation::default();
        original.with_translation(Some(translation));

        let cloned: Transformation = Transformation::from(&original);
        assert_eq!(cloned, original);
    }
}
