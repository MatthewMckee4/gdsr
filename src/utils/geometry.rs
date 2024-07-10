use pyo3::prelude::*;

use super::points::check_vec_not_empty;

pub fn bounding_box(points: &Vec<(f64, f64)>) -> PyResult<((f64, f64), (f64, f64))> {
    check_vec_not_empty(points)?;

    let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
    let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);

    for &(x, y) in points {
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

    Ok(((min_x, min_y), (max_x, max_y)))
}
