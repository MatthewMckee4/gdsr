use crate::{Point, Unit, elements::polygon::get_correct_polygon_points_format};

use super::ensure_points_same_units;

/// Calculate the area of a polygon defined by points.
///
/// Uses the [shoelace formula](https://en.wikipedia.org/wiki/Shoelace_formula).
/// Points should be in order (clockwise or counter-clockwise).
pub fn area(points: &[Point]) -> Unit {
    if points.len() < 3 {
        return Unit::default_integer(0);
    }

    let first_point = points[0];
    let units = first_point.units().0;

    let points = ensure_points_same_units(points, units);
    let closed = get_correct_polygon_points_format(points.clone());

    let mut sum = 0.0;
    for i in 0..closed.len() - 1 {
        let x_i = closed[i].x().float_value();
        let y_i = closed[i].y().float_value();
        let x_next = closed[i + 1].x().float_value();
        let y_next = closed[i + 1].y().float_value();
        sum += x_i * y_next - x_next * y_i;
    }

    Unit::float(sum.abs() * 0.5, units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_square() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];
        assert_eq!(area(&square), Unit::float(4.0, 1e-6));
    }

    #[test]
    fn test_area_triangle() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];
        assert_eq!(area(&triangle), Unit::float(2.0, 1e-6));
    }

    #[test]
    fn test_area_empty_polygon() {
        let empty: [Point; 0] = [];
        assert_eq!(area(&empty), Unit::float(0.0, 1e-6));

        let single_point = [Point::float(1.0, 1.0, 1e-6)];
        assert_eq!(area(&single_point), Unit::float(0.0, 1e-6));

        let two_points = [Point::float(0.0, 0.0, 1e-6), Point::float(1.0, 1.0, 1e-6)];
        assert_eq!(area(&two_points), Unit::float(0.0, 1e-6));
    }

    #[test]
    fn test_area_concave_polygon() {
        let concave = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(4.0, 0.0, 1e-6),
            Point::float(4.0, 4.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
        ];
        assert_eq!(area(&concave), Unit::float(12.0, 1e-6));
    }
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn area_is_non_negative(coords: Vec<(i16, i16)>) -> bool {
        if coords.len() < 3 {
            return true;
        }
        let points: Vec<Point> = coords
            .iter()
            .map(|&(x, y)| Point::float(f64::from(x), f64::from(y), 1e-6))
            .collect();
        area(&points).float_value() >= 0.0
    }

    /// Reversing the winding order should not change the unsigned area.
    #[quickcheck]
    fn area_invariant_under_reversal(coords: Vec<(i16, i16)>) -> bool {
        if coords.len() < 3 {
            return true;
        }
        let points: Vec<Point> = coords
            .iter()
            .map(|&(x, y)| Point::float(f64::from(x), f64::from(y), 1e-6))
            .collect();
        let mut reversed = points.clone();
        reversed.reverse();
        (area(&points).float_value() - area(&reversed).float_value()).abs() < 1e-10
    }
}
