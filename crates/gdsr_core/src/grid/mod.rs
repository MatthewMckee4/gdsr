use crate::{
    CoordNum, DatabaseIntegerUnit, Point,
    traits::{Movable, Transformable},
    transformation::Transformation,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid<DatabaseUnitT: CoordNum> {
    pub origin: Point<DatabaseUnitT>,
    pub columns: u32,
    pub rows: u32,
    pub spacing_x: Point<DatabaseUnitT>,
    pub spacing_y: Point<DatabaseUnitT>,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
}

impl<DatabaseUnitT: CoordNum> Grid<DatabaseUnitT> {
    pub fn new(
        origin: Point<DatabaseUnitT>,
        columns: u32,
        rows: u32,
        spacing_x: Point<DatabaseUnitT>,
        spacing_y: Point<DatabaseUnitT>,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
    ) -> Self {
        Self {
            origin,
            columns,
            rows,
            spacing_x,
            spacing_y,
            magnification,
            angle,
            x_reflection,
        }
    }
}

impl<T: CoordNum> Default for Grid<T> {
    fn default() -> Self {
        Grid {
            origin: Point::new(T::zero(), T::zero()),
            columns: 1,
            rows: 1,
            spacing_x: Point::new(T::zero(), T::zero()),
            spacing_y: Point::new(T::zero(), T::zero()),
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
        }
    }
}

impl<T: CoordNum> std::fmt::Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid at {:?} with {} columns and {} rows, spacing ({:?}, {:?}), magnification {:?}, angle {:?}, x_reflection {}",
            self.origin,
            self.columns,
            self.rows,
            self.spacing_x,
            self.spacing_y,
            self.magnification,
            self.angle,
            self.x_reflection,
        )
    }
}

impl Transformable for Grid<DatabaseIntegerUnit> {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.origin = transformation.apply_to_point(&self.origin);
        self.spacing_x = transformation.apply_to_point(&self.spacing_x);
        self.spacing_y = transformation.apply_to_point(&self.spacing_y);

        // Apply scale and rotation to grid properties
        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            self.angle += rotation.angle();
            let result = self.angle % 360.0;
            self.angle = if result < 0.0 { result + 360.0 } else { result };
        }

        // Handle reflection
        if transformation.reflection.is_some() {
            self.x_reflection = !self.x_reflection;
        }

        self
    }
}

impl Movable for Grid<DatabaseIntegerUnit> {
    fn move_to(&mut self, point: Point<DatabaseIntegerUnit>) -> &mut Self {
        self.origin = point;
        self
    }
}
