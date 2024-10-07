use path_type::PathType;
use pyo3::prelude::*;

use crate::{
    boolean::{
        boolean, BooleanOperationInput, BooleanOperationOperation, BooleanOperationResult,
        ExternalPolygonGroup,
    },
    element::Element,
    point::Point,
    traits::{
        Dimensions, LayerDataTypeMatches, Movable, Reflect, Rotatable, Scalable,
        ToExternalPolygonGroup,
    },
};

mod general;
mod io;
pub mod path_type;

#[pyclass(eq)]
#[derive(Clone, Default)]
pub struct Path {
    #[pyo3(get)]
    pub points: Vec<Point>,
    #[pyo3(get)]
    pub layer: i32,
    #[pyo3(get)]
    pub data_type: i32,
    #[pyo3(get)]
    pub path_type: Option<PathType>,
    #[pyo3(get)]
    pub width: Option<f64>,
}

impl Path {
    pub fn boolean(
        &self,
        other: BooleanOperationInput,
        operation: BooleanOperationOperation,
        py: Python,
    ) -> BooleanOperationResult {
        boolean(
            vec![Element::Path(Py::new(py, self.clone())?)],
            other,
            operation,
            self.layer,
            self.data_type,
        )
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        if self.points.len() != other.points.len() {
            return false;
        }

        for (self_point, other_point) in self.points.iter().zip(other.points.iter()) {
            if !self_point.epsilon_is_close(*other_point) {
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

        let mut extended_points = vec![];

        // Clone points and add the extra points at the front and back
        let mut points = self.points.clone();

        if points.len() >= 2 {
            // Add extra point at the front
            let first_point = points[0];
            let second_point = points[1];

            if let Some(angle) = first_point.angle_to(second_point) {
                let angle = angle.to_radians();
                let (sin, cos) = angle.sin_cos();
                let new_point = Point::new(
                    first_point.x - half_width * cos,
                    first_point.y - half_width * sin,
                );
                points.insert(0, new_point);
            }

            // Add extra point at the back
            let last_point = points[points.len() - 1];
            let second_last_point = points[points.len() - 2];

            if let Some(angle) = second_last_point.angle_to(last_point) {
                let angle = angle.to_radians();
                let (sin, cos) = angle.sin_cos();
                let new_point = Point::new(
                    last_point.x + half_width * cos,
                    last_point.y + half_width * sin,
                );
                points.push(new_point);
            }
        }

        for i in 0..points.len() {
            let point = points[i];

            // For the first and last points, we only consider one segment
            if i == 0 || i == points.len() - 1 {
                let next_point = if i == 0 { points[i + 1] } else { points[i - 1] };

                if let Some(angle) = point.angle_to(next_point) {
                    let angle = angle.to_radians();
                    let (sin, cos) = angle.sin_cos();
                    let perp_x = -sin;
                    let perp_y = cos;

                    extended_points.push(Point::new(
                        point.x + half_width * perp_x,
                        point.y + half_width * perp_y,
                    ));
                    extended_points.push(Point::new(
                        point.x - half_width * perp_x,
                        point.y - half_width * perp_y,
                    ));
                }
            } else {
                // For all other points, consider both segments
                let prev_point = points[i - 1];
                let next_point = points[i + 1];

                if let Some(angle_prev) = point.angle_to(prev_point) {
                    let angle_prev = angle_prev.to_radians();
                    let (sin_prev, cos_prev) = angle_prev.sin_cos();
                    let perp_x_prev = -sin_prev;
                    let perp_y_prev = cos_prev;

                    extended_points.push(Point::new(
                        point.x + half_width * perp_x_prev,
                        point.y + half_width * perp_y_prev,
                    ));
                    extended_points.push(Point::new(
                        point.x - half_width * perp_x_prev,
                        point.y - half_width * perp_y_prev,
                    ));
                }

                if let Some(angle_next) = point.angle_to(next_point) {
                    let angle_next = angle_next.to_radians();
                    let (sin_next, cos_next) = angle_next.sin_cos();
                    let perp_x_next = -sin_next;
                    let perp_y_next = cos_next;

                    extended_points.push(Point::new(
                        point.x + half_width * perp_x_next,
                        point.y + half_width * perp_y_next,
                    ));
                    extended_points.push(Point::new(
                        point.x - half_width * perp_x_next,
                        point.y - half_width * perp_y_next,
                    ));
                }
            }
        }

        for point in extended_points {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }

        (min, max)
    }
}

impl Reflect for Path {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        for point in &mut self.points {
            *point = point.reflect(angle, centre);
        }
        self
    }
}

impl LayerDataTypeMatches for Path {
    fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        layer_data_types.contains(&(self.layer, self.data_type)) || layer_data_types.is_empty()
    }
}

impl ToExternalPolygonGroup for Path {
    fn to_external_polygon_group(&self) -> PyResult<ExternalPolygonGroup> {
        let half_width = self.width.unwrap_or(0.0) / 2.0;
        let mut points: Vec<(f64, f64)> = Vec::new();

        for window in self.points.windows(2) {
            let (start, end) = (window[0], window[1]);
            let angle = start.angle_to(end).unwrap_or(0.0).to_radians();
            let (sin, cos) = angle.sin_cos();
            let perp_x = -sin;
            let perp_y = cos;

            points.push((start.x + half_width * perp_x, start.y + half_width * perp_y));
            points.push((end.x + half_width * perp_x, end.y + half_width * perp_y));
        }

        for window in self.points.windows(2).rev() {
            let (start, end) = (window[0], window[1]);
            let angle = start.angle_to(end).unwrap_or(0.0).to_radians();
            let (sin, cos) = angle.sin_cos();
            let perp_x = -sin;
            let perp_y = cos;

            points.push((end.x - half_width * perp_x, end.y - half_width * perp_y));
            points.push((start.x - half_width * perp_x, start.y - half_width * perp_y));
        }

        if let Some(first) = self.points.first() {
            points.push((first.x + half_width, first.y));
        }

        Ok(points.into())
    }
}
