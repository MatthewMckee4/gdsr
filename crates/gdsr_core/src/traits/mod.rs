use std::fs::File;
use std::io;

use crate::{Point, transformation::Transformation};

pub trait ToGds {
    fn _to_gds(&self, file: &mut File, scale: f64) -> io::Result<()>;
}

pub trait Transformable {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self;

    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.transform(&Transformation::new().with_rotation(angle, centre))
    }

    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.transform(&Transformation::new().with_scale(factor, centre))
    }

    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.transform(&Transformation::new().with_reflection(angle, centre))
    }
}

pub trait Movable: Transformable {
    fn move_by(&mut self, delta: Point) -> &mut Self {
        self.transform(&Transformation::new().with_translation(delta))
    }

    fn move_to(&mut self, target: Point) -> &mut Self;
}

pub trait Dimensions {
    fn bounding_box(&self) -> (Point, Point);
}
