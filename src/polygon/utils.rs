use log::info;
use pyo3::prelude::*;

use crate::{point::Point, validation::input::input_points_like_to_points_vec};

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

pub fn input_polygon_points_to_correct_format(points: &Bound<'_, PyAny>) -> PyResult<Vec<Point>> {
    let points = input_points_like_to_points_vec(points)?;
    Ok(polygon_points_to_correct_format(points))
}
