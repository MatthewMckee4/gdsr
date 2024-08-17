use crate::{
    point::Point,
    traits::{Dimensions, LayerDataTypeMatches, Movable, Reflect, Rotatable, Scalable},
    utils::geometry::bounding_box,
};
use pyo3::prelude::*;

mod general;
mod io;
mod utils;

#[pyclass(eq)]
#[derive(Clone, Default)]
pub struct Polygon {
    #[pyo3(get)]
    pub points: Vec<Point>,
    #[pyo3(get)]
    pub layer: i32,
    #[pyo3(get)]
    pub data_type: i32,
}

impl PartialEq for Polygon {
    fn eq(&self, other: &Self) -> bool {
        if self.points.len() != other.points.len() {
            return false;
        }

        for (self_point, other_point) in self.points.iter().zip(other.points.iter()) {
            if !self_point.epsilon_is_close(*other_point) {
                return false;
            }
        }

        self.layer == other.layer && self.data_type == other.data_type
    }
}

impl std::fmt::Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Polygon with {} point(s), starting at {:?} on layer {}, data type {}",
            self.points.len(),
            self.points[0],
            self.layer,
            self.data_type
        )
    }
}

impl std::fmt::Debug for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.points.as_slice() {
            [] => {
                write!(f, "Polygon([], {}, {})", self.layer, self.data_type)
            }
            [first_point] => write!(
                f,
                "Polygon([{:?}, ..., {:?}], {}, {})",
                first_point, first_point, self.layer, self.data_type
            ),
            [first_point, _] => write!(
                f,
                "Polygon([{:?}, ..., {:?}], {}, {})",
                first_point, first_point, self.layer, self.data_type
            ),
            [first_point, .., second_last_point, _] => write!(
                f,
                "Polygon([{:?}, ..., {:?}], {}, {})",
                first_point, second_last_point, self.layer, self.data_type
            ),
        }
    }
}

impl Movable for Polygon {
    fn move_by(&mut self, delta: Point) -> &mut Self {
        let new_points = self.points.iter().map(|point| *point + delta).collect();
        self.points = new_points;
        self
    }

    fn move_to(&mut self, target: Point) -> &mut Self {
        let delta = target - self.points[0];
        self.move_by(delta)
    }
}

impl Rotatable for Polygon {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.rotate(angle, centre);
        }
        self
    }
}

impl Scalable for Polygon {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.scale(factor, centre);
        }
        self
    }
}

impl Dimensions for Polygon {
    fn bounding_box(&self) -> (Point, Point) {
        bounding_box(&self.points)
    }
}

impl Reflect for Polygon {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.reflect(angle, centre);
        }
        self
    }
}

impl LayerDataTypeMatches for Polygon {
    fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        layer_data_types.contains(&(self.layer, self.data_type)) || layer_data_types.is_empty()
    }
}
