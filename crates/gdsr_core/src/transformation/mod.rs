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
    pub fn with_reflection(&mut self, reflection: Option<Reflection>) -> &mut Self {
        self.reflection = reflection;
        self
    }

    pub fn with_rotation(&mut self, rotation: Option<Rotation>) -> &mut Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(&mut self, scale: Option<Scale>) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn with_translation(&mut self, translation: Option<Translation>) -> &mut Self {
        self.translation = translation;
        self
    }

    pub fn apply_to_point<DatabaseUnitT: CoordNum>(
        &self,
        point: &Point<DatabaseUnitT>,
    ) -> Point<DatabaseUnitT> {
        let mut new_point = point.clone();

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
