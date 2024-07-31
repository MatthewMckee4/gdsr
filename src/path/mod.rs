use path_type::PathType;
use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
};

mod general;
mod io;
pub mod path_type;

#[pyclass(eq)]
#[derive(Clone, PartialEq, Default)]
pub struct Path {
    #[pyo3(get)]
    pub points: Vec<Point>,
    #[pyo3(get)]
    pub layer: i32,
    #[pyo3(get)]
    pub data_type: i32,
    #[pyo3(get, set)]
    pub path_type: Option<PathType>,
    #[pyo3(get, set)]
    pub width: Option<f64>,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Path with {} points on layer {} and data type {}. PathType: {:?} with width {:?}",
            self.points.len(),
            self.layer,
            self.data_type,
            self.path_type.unwrap_or_default(),
            self.width.unwrap_or_default()
        )
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Path([{:?}, ..., {:?}], {}, {}, {:?}, {:?})",
            self.points.first(),
            self.points.last(),
            self.layer,
            self.data_type,
            self.path_type,
            self.width
        )
    }
}

impl Movable for Path {
    fn move_by(&mut self, delta: Point) -> &mut Self {
        for point in &mut self.points {
            *point += delta;
        }
        self
    }

    fn move_to(&mut self, target: Point) -> &mut Self {
        let delta = target - self.points[0];
        self.move_by(delta)
    }
}

impl Rotatable for Path {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.rotate(angle, centre);
        }
        self
    }
}

impl Scalable for Path {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.scale(factor, centre);
        }
        self
    }
}

impl Dimensions for Path {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min = Point::new(f64::INFINITY, f64::INFINITY);
        let mut max = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);

        let width = self.width.unwrap_or(0.0);
        let half_width = width / 2.0;

        for point in &self.points {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }

        let angle_from_second_last_to_last = self.points[self.points.len() - 1]
            .angle_to(
                *self
                    .points
                    .get(self.points.len() - 2)
                    .unwrap_or(&self.points[0]),
            )
            .unwrap()
            .unwrap_or_default();

        let angle_from_second_to_first = self.points[0]
            .angle_to(*self.points.get(1).unwrap_or(&self.points[0]))
            .unwrap()
            .unwrap_or_default();

        match self.path_type {
            Some(PathType::Overlap) => {
                // multiple width by the cos of the angle between the first and second point
                let (sin, cos) = angle_from_second_to_first.to_radians().sin_cos();
                min.x -= half_width * cos;
                min.y -= half_width * sin;

                // multiple width by the cos of the angle between the second last and last point
                let (sin, cos) = angle_from_second_last_to_last.to_radians().sin_cos();
                max.x += half_width * cos;
                max.y += half_width * sin;
            }
            Some(PathType::Round) => {
                min.x -= half_width;
                min.y -= half_width;
                max.x += half_width;
                max.y += half_width;
            }
            Some(PathType::Square) | None => {}
        }

        (min, max)
    }
}
