use pyo3::prelude::*;

use crate::point::Point;

use super::general::check_points_vec_not_empty;

pub fn bounding_box(points: &Vec<Point>) -> (Point, Point) {
    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);

    for &point in points {
        let (x, y) = (point.x, point.y);
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }

    (Point::new(min_x, min_y), Point::new(max_x, max_y))
}

pub fn area(points: &[Point]) -> PyResult<f64> {
    check_points_vec_not_empty(points)?;

    let mut area = 0.0;
    let length = points.len();

    for index in 0..length {
        let next_index = (index + 1) % length;
        area += points[index].cross(points[next_index])?;
    }

    Ok(area.abs() / 2.0)
}

pub fn perimeter(points: &[Point]) -> PyResult<f64> {
    check_points_vec_not_empty(points)?;

    let mut perimeter = 0.0;
    let length = points.len();

    for index in 0..length - 1 {
        let next_index = (index + 1) % length;
        perimeter += points[index].distance_to(points[next_index])?;
    }

    Ok(perimeter)
}

pub fn distance_between_points(point1: &Point, point2: &Point) -> PyResult<f64> {
    Ok(((point1.x - point2.x).powi(2) + (point1.y - point2.y).powi(2)).sqrt())
}
pub fn is_point_inside(point: &Point, points: &[Point]) -> bool {
    if is_point_on_edge(point, points) {
        return true;
    }

    let mut inside = false;
    let n = points.len();
    let mut j = n - 1;

    for i in 0..n {
        let xi = points[i].x;
        let yi = points[i].y;
        let xj = points[j].x;
        let yj = points[j].y;

        let intersect = ((yi > point.y) != (yj > point.y))
            && (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }

    inside
}

pub fn is_point_on_edge(point: &Point, polygon_points: &[Point]) -> bool {
    let num_points = polygon_points.len();

    for i in 0..num_points {
        let start = &polygon_points[i];
        let end = &polygon_points[(i + 1) % num_points];

        if is_point_on_line_segment(point, start, end) {
            return true;
        }
    }

    false
}

fn is_point_on_line_segment(point: &Point, a: &Point, b: &Point) -> bool {
    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);

    if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
        return false;
    }

    let cross_product = (point.y - a.y) * (b.x - a.x) - (point.x - a.x) * (b.y - a.y);

    if cross_product.abs() > f64::EPSILON {
        return false;
    }

    true
}

pub fn round_to_decimals(value: f64, ndigits: u32) -> f64 {
    let factor = 10f64.powi(ndigits as i32);
    (value * factor).round() / factor
}

pub fn rotate_points_to_minimum(points: &mut [Point]) {
    if let Some((min_index, _)) = points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    {
        points.rotate_left(min_index);
    }
}
