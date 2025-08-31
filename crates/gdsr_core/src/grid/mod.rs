use crate::{
    Point,
    traits::{Movable, Transformable},
    transformation::Transformation,
};

#[derive(Clone)]
pub struct Grid {
    pub origin: Point,
    pub columns: u32,
    pub rows: u32,
    pub spacing_x: Point,
    pub spacing_y: Point,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
}

impl Grid {
    pub fn new(
        origin: Point,
        columns: u32,
        rows: u32,
        spacing_x: Point,
        spacing_y: Point,
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

impl Default for Grid {
    fn default() -> Self {
        Grid {
            origin: Point::default(),
            columns: 1,
            rows: 1,
            spacing_x: Point::default(),
            spacing_y: Point::default(),
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
        }
    }
}

impl PartialEq for Grid {
    fn eq(&self, other: &Self) -> bool {
        (self.origin.x() - other.origin.x()).abs() < f64::EPSILON
            && (self.origin.y() - other.origin.y()).abs() < f64::EPSILON
            && self.columns == other.columns
            && self.rows == other.rows
            && (self.spacing_x.x() - other.spacing_x.x()).abs() < f64::EPSILON
            && (self.spacing_x.y() - other.spacing_x.y()).abs() < f64::EPSILON
            && (self.spacing_y.x() - other.spacing_y.x()).abs() < f64::EPSILON
            && (self.spacing_y.y() - other.spacing_y.y()).abs() < f64::EPSILON
            && self.magnification == other.magnification
            && self.angle == other.angle
            && self.x_reflection == other.x_reflection
    }
}

impl std::fmt::Display for Grid {
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

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid({:?}, {}, {}, {:?}, {:?}, {:?}, {:?}, {})",
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

impl Transformable for Grid {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.origin = transformation.apply_to_point(&self.origin);
        self.spacing_x = transformation.apply_to_point(&self.spacing_x);
        self.spacing_y = transformation.apply_to_point(&self.spacing_y);

        // Apply scale and rotation to grid properties
        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor;
        }

        if let Some(rotation) = &transformation.rotation {
            self.angle += rotation.angle;
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

impl Movable for Grid {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.origin = point;
        self
    }
}
