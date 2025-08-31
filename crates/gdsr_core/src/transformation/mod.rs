use crate::Point;

mod reflection;
mod rotation;
mod scale;
mod translation;

pub use reflection::Reflection;
pub use rotation::Rotation;
pub use scale::Scale;
pub use translation::Translation;

#[derive(Clone, Debug)]
pub struct Transformation {
    pub translation: Option<Translation>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
    pub reflection: Option<Reflection>,
}

impl Transformation {
    pub fn new() -> Self {
        Self {
            translation: None,
            rotation: None,
            scale: None,
            reflection: None,
        }
    }

    pub fn with_translation(mut self, delta: Point) -> Self {
        self.translation = Some(Translation::new(delta));
        self
    }

    pub fn with_rotation(mut self, angle: f64, centre: Point) -> Self {
        self.rotation = Some(Rotation::new(angle, centre));
        self
    }

    pub fn with_scale(mut self, factor: f64, centre: Point) -> Self {
        self.scale = Some(Scale::new(factor, centre));
        self
    }

    pub fn with_reflection(mut self, angle: f64, centre: Point) -> Self {
        self.reflection = Some(Reflection::new(angle, centre));
        self
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        let mut result = *point;

        if let Some(translation) = &self.translation {
            result = translation.apply_to_point(&result);
        }

        if let Some(scale) = &self.scale {
            result = scale.apply_to_point(&result);
        }

        if let Some(rotation) = &self.rotation {
            result = rotation.apply_to_point(&result);
        }

        if let Some(reflection) = &self.reflection {
            result = reflection.apply_to_point(&result);
        }

        result
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new()
    }
}

