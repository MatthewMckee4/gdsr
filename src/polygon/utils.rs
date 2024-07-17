use log::info;
use pyo3::{exceptions::PyTypeError, prelude::*, types::PySequence};

use crate::{
    point::{py_any_to_point, Point},
    utils::general::check_points_vec_not_empty,
};

pub fn input_polygon_points_to_correct_format(points: &Bound<'_, PyAny>) -> PyResult<Vec<Point>> {
    if let Ok(points) = points.downcast::<PySequence>() {
        let mut points_list = Vec::new();
        for item in points.iter()? {
            let point = py_any_to_point(&item?)?;
            points_list.push(point);
        }
        check_points_vec_not_empty(&points_list)?;
        Ok(polygon_points_to_correct_format(points_list))
    } else {
        Err(PyTypeError::new_err(
            "Invalid points format: not a sequence",
        ))
    }
}

fn are_points_closed(points: &[Point]) -> bool {
    points.first() == points.last()
}

pub fn close_points(points: &[Point]) -> Vec<Point> {
    if are_points_closed(points) {
        points.to_vec()
    } else {
        info!("The points {:?} are not closed, closing them", points);
        let mut closed_points = points.to_vec();
        closed_points.push(points[0]);
        closed_points
    }
}

pub fn polygon_points_to_correct_format(points: Vec<Point>) -> Vec<Point> {
    close_points(&points)
}
