use crate::{CoordNum, Point};

mod reflection;
mod rotation;
mod scale;
mod translation;

pub use reflection::Reflection;
pub use rotation::Rotation;
pub use scale::Scale;
pub use translation::Translation;

#[derive(Clone, Debug, Default)]
pub struct Transformation {
    pub reflection: Option<Reflection>,
    pub rotation: Option<Rotation>,
    pub scale: Option<Scale>,
    pub translation: Option<Translation>,
}

impl Transformation {
    #[must_use] 
    pub const fn translation(translation: Translation) -> Self {
        Self {
            reflection: None,
            rotation: None,
            scale: None,
            translation: Some(translation),
        }
    }

    #[must_use] 
    pub const fn rotation(rotation: Rotation) -> Self {
        Self {
            reflection: None,
            rotation: Some(rotation),
            scale: None,
            translation: None,
        }
    }

    #[must_use] 
    pub const fn scale(scale: Scale) -> Self {
        Self {
            reflection: None,
            rotation: None,
            scale: Some(scale),
            translation: None,
        }
    }

    #[must_use] 
    pub const fn reflection(reflection: Reflection) -> Self {
        Self {
            reflection: Some(reflection),
            rotation: None,
            scale: None,
            translation: None,
        }
    }

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

    pub fn apply_to_point<DatabaseUnitT: CoordNum>(
        &self,
        point: &Point<DatabaseUnitT>,
    ) -> Point<DatabaseUnitT> {
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
