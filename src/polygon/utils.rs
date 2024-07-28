use log::info;
use pyo3::prelude::*;

use crate::{point::Point, validation::input::py_any_to_points_vec};

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

pub fn get_correct_polygon_points_format(points: Vec<Point>) -> Vec<Point> {
    close_points(&points)
}

pub fn py_any_to_correct_polygon_points_format(points: &Bound<'_, PyAny>) -> PyResult<Vec<Point>> {
    let points = py_any_to_points_vec(points)?;
    Ok(get_correct_polygon_points_format(points))
}
