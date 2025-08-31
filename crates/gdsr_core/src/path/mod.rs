use path_type::PathType;

use crate::{
    Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
pub mod path_type;

#[derive(Clone, Default)]
pub struct Path {
    pub points: Vec<Point>,
    pub layer: i32,
    pub data_type: i32,
    pub path_type: Option<PathType>,
    pub width: Option<f64>,
}

impl Path {
    pub fn new(
        points: Vec<Point>,
        layer: i32,
        data_type: i32,
        path_type: Option<PathType>,
        width: Option<f64>,
    ) -> Self {
        Self {
            points,
            layer,
            data_type,
            path_type,
            width,
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        if self.points.len() != other.points.len() {
            return false;
        }

        for (self_point, other_point) in self.points.iter().zip(other.points.iter()) {
            if (self_point.x() - other_point.x()).abs() > f64::EPSILON
                || (self_point.y() - other_point.y()).abs() > f64::EPSILON
            {
                return false;
            }
        }

        self.layer == other.layer
            && self.data_type == other.data_type
            && self.path_type == other.path_type
            && self.width == other.width
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Path with {} points on layer {} with data type {}, {:?} and width {}",
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
        match self.points.as_slice() {
            [] => {
                write!(
                    f,
                    "Path([], {}, {}, {:?}, {:?})",
                    self.layer,
                    self.data_type,
                    self.path_type.unwrap_or_default(),
                    self.width.unwrap_or_default()
                )
            }
            [first_point] => write!(
                f,
                "Path([{:?}], {}, {}, {:?}, {})",
                first_point,
                self.layer,
                self.data_type,
                self.path_type.unwrap_or_default(),
                self.width.unwrap_or_default()
            ),
            [first_point, last_point] => write!(
                f,
                "Path([{:?}, {:?}], {}, {}, {:?}, {})",
                first_point,
                last_point,
                self.layer,
                self.data_type,
                self.path_type.unwrap_or_default(),
                self.width.unwrap_or_default()
            ),
            [first_point, .., last_point] => write!(
                f,
                "Path([{:?}, ..., {:?}], {}, {}, {:?}, {})",
                first_point,
                last_point,
                self.layer,
                self.data_type,
                self.path_type.unwrap_or_default(),
                self.width.unwrap_or_default()
            ),
        }
    }
}

impl Transformable for Path {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.points = self
            .points
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        self
    }
}

impl Movable for Path {
    fn move_to(&mut self, target: Point) -> &mut Self {
        let first_point = &self.points[0];
        let delta = Point::new(target.x() - first_point.x(), target.y() - first_point.y());
        self.move_by(delta)
    }
}

impl Dimensions for Path {
    fn bounding_box(&self) -> (Point, Point) {
        if self.width.is_some() && self.width.unwrap() > 0.0 {
            // For paths with width, we need to consider the extended points
            let width = self.width.unwrap();
            let half_width = width / 2.0;

            // Create extended points considering the width
            let mut extended_points = Vec::new();

            for i in 0..self.points.len() {
                let point = self.points[i];

                // Add points offset by half-width in perpendicular directions
                if i > 0 {
                    let prev = self.points[i - 1];
                    let dx = point.x() - prev.x();
                    let dy = point.y() - prev.y();
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }

                if i < self.points.len() - 1 {
                    let next = self.points[i + 1];
                    let dx = next.x() - point.x();
                    let dy = next.y() - point.y();
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }
            }

            bounding_box(&extended_points)
        } else {
            // For paths without width, use the standard bounding box
            bounding_box(&self.points)
        }
    }
}
