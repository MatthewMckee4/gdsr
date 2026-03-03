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

    use quickcheck::{Arbitrary, Gen};

    const MAX_COORD: i32 = 10_000;
    const MAX_ANGLE: f64 = std::f64::consts::TAU;
    const MAX_SCALE: f64 = 100.0;

    impl Arbitrary for Reflection {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_angle = f64::arbitrary(g);
            let angle = if raw_angle.is_finite() {
                raw_angle % MAX_ANGLE
            } else {
                0.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(angle, centre)
        }
    }

    impl Arbitrary for Rotation {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_angle = f64::arbitrary(g);
            let angle = if raw_angle.is_finite() {
                raw_angle % MAX_ANGLE
            } else {
                0.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(angle, centre)
        }
    }

    impl Arbitrary for Scale {
        fn arbitrary(g: &mut Gen) -> Self {
            let raw_factor = f64::arbitrary(g);
            let factor = if raw_factor.is_finite() {
                (raw_factor % MAX_SCALE).clamp(-MAX_SCALE, MAX_SCALE)
            } else {
                1.0
            };
            let centre = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(factor, centre)
        }
    }

    impl Arbitrary for Translation {
        fn arbitrary(g: &mut Gen) -> Self {
            let delta = Point::integer(
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
                1e-9,
            );
            Self::new(delta)
        }
    }

    impl Arbitrary for Transformation {
        fn arbitrary(g: &mut Gen) -> Self {
            let reflection = if bool::arbitrary(g) {
                Some(Reflection::arbitrary(g))
            } else {
                None
            };
            let rotation = if bool::arbitrary(g) {
                Some(Rotation::arbitrary(g))
            } else {
                None
            };
            let scale = if bool::arbitrary(g) {
                Some(Scale::arbitrary(g))
            } else {
                None
            };
            let translation = if bool::arbitrary(g) {
                Some(Translation::arbitrary(g))
            } else {
                None
            };
            let mut t = Self::default();
            t.with_reflection(reflection);
            t.with_rotation(rotation);
            t.with_scale(scale);
            t.with_translation(translation);
            t
        }
    }

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

    #[allow(clippy::needless_pass_by_value)]
    mod property_tests {
        use super::*;
        use quickcheck_macros::quickcheck;

        fn point_approx_eq(a: &Point, b: &Point, epsilon: f64) -> bool {
            (a.x() - b.x()).absolute_value().abs() < epsilon
                && (a.y() - b.y()).absolute_value().abs() < epsilon
        }

        /// Identity transformation does not change a point.
        #[quickcheck]
        fn identity_does_not_change_point(x: i32, y: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let identity = Transformation::default();
            identity.apply_to_point(&point) == point
        }

        /// Double reflection with the same axis cancels out.
        #[quickcheck]
        fn double_reflection_cancels(reflection: Reflection, x: i32, y: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let once = reflection.apply_to_point(&point);
            let twice = reflection.apply_to_point(&once);
            point_approx_eq(&twice, &point, 1e-6)
        }

        /// Rotation by 0 is identity.
        #[quickcheck]
        fn rotation_by_zero_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let centre = Point::integer(cx, cy, 1e-9);
            let rotation = Rotation::new(0.0, centre);
            let result = rotation.apply_to_point(&point);
            point_approx_eq(&result, &point, 1e-6)
        }

        /// Rotation by 2*pi is approximately identity.
        #[quickcheck]
        fn rotation_by_two_pi_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let centre = Point::integer(cx, cy, 1e-9);
            let rotation = Rotation::new(std::f64::consts::TAU, centre);
            let result = rotation.apply_to_point(&point);
            point_approx_eq(&result, &point, 1e-6)
        }

        /// Scale by 1 is identity.
        #[quickcheck]
        fn scale_by_one_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let centre = Point::integer(cx, cy, 1e-9);
            let scale = Scale::new(1.0, centre);
            let result = scale.apply_to_point(&point);
            point_approx_eq(&result, &point, 1e-6)
        }

        /// Translation composition: translate(a) then translate(b) == translate(a + b).
        #[quickcheck]
        fn translation_composition(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> bool {
            let ax = (ax % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let ay = (ay % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let bx = (bx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let by = (by % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let px = (px % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let py = (py % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);

            let point = Point::integer(px, py, 1e-9);
            let t_a = Translation::new(Point::integer(ax, ay, 1e-9));
            let t_b = Translation::new(Point::integer(bx, by, 1e-9));
            let t_ab = Translation::new(Point::integer(ax + bx, ay + by, 1e-9));

            let sequential = t_b.apply_to_point(&t_a.apply_to_point(&point));
            let composed = t_ab.apply_to_point(&point);
            point_approx_eq(&sequential, &composed, 1e-6)
        }

        /// Scale then inverse scale returns to the original point.
        #[quickcheck]
        fn scale_then_inverse_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let centre = Point::integer(cx, cy, 1e-9);

            let factor = 2.5;
            let scale = Scale::new(factor, centre);
            let inverse_scale = Scale::new(1.0 / factor, centre);

            let result = inverse_scale.apply_to_point(&scale.apply_to_point(&point));
            point_approx_eq(&result, &point, 1e-6)
        }

        /// Rotation then inverse rotation returns to the original point.
        #[quickcheck]
        fn rotation_then_inverse_is_identity(rotation: Rotation, x: i32, y: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let inverse = Rotation::new(-rotation.angle(), *rotation.centre());

            let result = inverse.apply_to_point(&rotation.apply_to_point(&point));
            point_approx_eq(&result, &point, 1e-6)
        }

        /// Translation then inverse translation returns to the original point.
        #[quickcheck]
        fn translation_then_inverse_is_identity(translation: Translation, x: i32, y: i32) -> bool {
            let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
            let point = Point::integer(x, y, 1e-9);
            let applied = translation.apply_to_point(&point);
            let delta = applied - point;
            let inverse = Translation::new(delta * -1);

            let result = inverse.apply_to_point(&applied);
            point_approx_eq(&result, &point, 1e-6)
        }
    }
}
