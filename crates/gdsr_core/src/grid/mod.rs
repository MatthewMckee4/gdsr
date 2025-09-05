use crate::{
    CoordNum, DatabaseIntegerUnit, Point,
    traits::{Movable, Transformable},
    transformation::Transformation,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) origin: Point<DatabaseUnitT>,
    pub(crate) columns: u32,
    pub(crate) rows: u32,
    pub(crate) spacing_x: Point<DatabaseUnitT>,
    pub(crate) spacing_y: Point<DatabaseUnitT>,
    pub(crate) magnification: f64,
    pub(crate) angle: f64,
    pub(crate) x_reflection: bool,
}

impl<DatabaseUnitT: CoordNum> Grid<DatabaseUnitT> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: impl Into<Point<DatabaseUnitT>>,
        columns: u32,
        rows: u32,
        spacing_x: impl Into<Point<DatabaseUnitT>>,
        spacing_y: impl Into<Point<DatabaseUnitT>>,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
    ) -> Self {
        Self {
            origin: origin.into(),
            columns,
            rows,
            spacing_x: spacing_x.into(),
            spacing_y: spacing_y.into(),
            magnification,
            angle,
            x_reflection,
        }
    }
}

impl<T: CoordNum> Default for Grid<T> {
    fn default() -> Self {
        Self {
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

impl<DatabaseUnitT: CoordNum> Transformable for Grid<DatabaseUnitT> {
    fn transform(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.origin = transformation.apply_to_point(&new_self.origin);
        new_self.spacing_x = transformation.apply_to_point(&new_self.spacing_x);
        new_self.spacing_y = transformation.apply_to_point(&new_self.spacing_y);

        // Apply scale and rotation to grid properties
        if let Some(scale) = &transformation.scale {
            new_self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            new_self.angle += rotation.angle();
            let result = new_self.angle % 360.0;
            new_self.angle = if result < 0.0 { result + 360.0 } else { result };
        }

        // Handle reflection
        if transformation.reflection.is_some() {
            new_self.x_reflection = !new_self.x_reflection;
        }

        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Grid<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let mut new_self = self.clone();
        new_self.origin = Point::new(
            DatabaseUnitT::from_float(target.x().to_float()),
            DatabaseUnitT::from_float(target.y().to_float()),
        );
        new_self
    }
}
