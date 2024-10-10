use crate::{
    boolean::{
        boolean, BooleanOperationInput, BooleanOperationOperation, BooleanOperationResult,
        ExternalPolygonGroup,
    },
    config::gds_file_types::MAX_POLYGON_POINTS,
    element::Element,
    point::Point,
    traits::{
        Dimensions, FromExternalPolygonGroup, LayerDataTypeMatches, Movable, Reflect, Rotatable,
        Scalable, Simplifiable, ToExternalPolygonGroup,
    },
    utils::geometry::{bounding_box, rotate_points_to_minimum},
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

impl Polygon {
    pub fn boolean(
        &self,
        other: BooleanOperationInput,
        operation: BooleanOperationOperation,
        py: Python,
    ) -> BooleanOperationResult {
        boolean(
            vec![Element::Polygon(Py::new(py, self.clone())?)],
            other,
            operation,
            self.layer,
            self.data_type,
        )
    }
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
                "Polygon([{:?}], {}, {})",
                first_point, self.layer, self.data_type
            ),
            [first_point, _] => write!(
                f,
                "Polygon([{:?}], {}, {})",
                first_point, self.layer, self.data_type
            ),
            [first_point, second_point, _] => write!(
                f,
                "Polygon([{:?}, {:?}], {}, {})",
                first_point, second_point, self.layer, self.data_type
            ),
            [first_point, second_point, third_point, _] => write!(
                f,
                "Polygon([{:?}, {:?}, {:?}], {}, {})",
                first_point, second_point, third_point, self.layer, self.data_type
            ),
            [first_point, second_point, third_point, fourth_point, _] => write!(
                f,
                "Polygon([{:?}, {:?}, {:?}, {:?}], {}, {})",
                first_point, second_point, third_point, fourth_point, self.layer, self.data_type
            ),
            [first_point, second_point, third_point, .., second_last_point, _] => write!(
                f,
                "Polygon([{:?}, {:?}, {:?}, ..., {:?}], {}, {})",
                first_point,
                second_point,
                third_point,
                second_last_point,
                self.layer,
                self.data_type
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

impl Simplifiable for Polygon {
    fn simplify(&mut self) -> &mut Self {
        let mut simplified_points = Vec::new();
        let n = self.points.len();

        if n < 3 {
            return self;
        }

        let mut unique_points = self.points.clone();
        unique_points.dedup();

        simplified_points.push(unique_points[0]);
        let m: usize = unique_points.len();
        for i in 1..m - 1 {
            let prev = unique_points[i - 1];
            let curr = unique_points[i];
            let next = unique_points[i + 1];

            let dx1 = curr.x - prev.x;
            let dy1 = curr.y - prev.y;
            let dx2 = next.x - curr.x;
            let dy2 = next.y - curr.y;

            if dx1 * dy2 != dy1 * dx2 {
                simplified_points.push(curr);
            }
        }

        simplified_points.dedup();

        if simplified_points.first() != simplified_points.last() {
            simplified_points.push(simplified_points[0]);
        }

        self.points = simplified_points;
        self
    }
}

impl ToExternalPolygonGroup for Polygon {
    fn to_external_polygon_group(&self) -> PyResult<ExternalPolygonGroup> {
        let points: Vec<(f64, f64)> = self.points.iter().map(|p| p.as_tuple()).collect();
        Ok(points.into())
    }
}

impl FromExternalPolygonGroup for Polygon {
    fn from_external_polygon_group(
        external_polygon_group: ExternalPolygonGroup,
        layer: i32,
        data_type: i32,
    ) -> PyResult<Vec<Self>> {
        let mut points: Vec<Vec<Point>> = Vec::new();
        for polygon in external_polygon_group {
            let mut current_polygon_points: Vec<Point> =
                polygon.iter().map(|p| Point::new(p.x(), p.y())).collect();
            for point in &mut current_polygon_points {
                *point = point.round(9);
            }

            rotate_points_to_minimum(&mut current_polygon_points);

            points.push(current_polygon_points);
        }
        let mut polygons = Vec::new();
        let mut current_points = Vec::new();

        for polygon_points in points {
            if current_points.len() + polygon_points.len() <= MAX_POLYGON_POINTS {
                current_points.extend(polygon_points);
            } else {
                let mut polygon = Polygon {
                    points: current_points,
                    layer,
                    data_type,
                };
                polygon.simplify();
                polygons.push(polygon);
                current_points = polygon_points;
            }
        }

        if !current_points.is_empty() {
            let mut polygon = Polygon {
                points: current_points,
                layer,
                data_type,
            };
            polygon.simplify();
            polygons.push(polygon);
        }
        Ok(polygons)
    }
}
