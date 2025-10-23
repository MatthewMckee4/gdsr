use crate::Point;

mod reflection;
mod rotation;
mod scale;
mod translation;

pub use reflection::Reflection;
pub use rotation::Rotation;
pub use scale::Scale;
pub use translation::Translation;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transformation {
    pub reflection: Option<Reflection>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
    pub translation: Option<Translation>,
}

impl Transformation {
    pub const fn with_reflection(&mut self, reflection: Option<Reflection>) -> &mut Self {
        self.reflection = reflection;
        self
    }

    pub const fn with_rotation(&mut self, rotation: Option<Rotation>) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub const fn with_scale(&mut self, scale: Option<Scale>) -> &mut Self {
        self.scale = scale;
        self
    }

    pub const fn with_translation(&mut self, translation: Option<Translation>) -> &mut Self {
        self.translation = translation;
        self
    }

    #[must_use]
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

impl From<&mut Self> for Transformation {
    fn from(value: &mut Self) -> Self {
        value.clone()
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

    #[test]
    fn test_clone() {
        let translation = Translation::new(Point::integer(10, 20, 1e-9));
        let mut transformation = Transformation::default();
        transformation.with_translation(Some(translation));

        let cloned = transformation.clone();
        assert_eq!(cloned.translation, transformation.translation);
    }
}
