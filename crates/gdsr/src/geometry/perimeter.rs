use crate::{Point, Unit};

use super::ensure_points_same_units;

/// Calculate the perimeter (total path length) of a sequence of points.
///
/// Sums [Euclidean distances](https://en.wikipedia.org/wiki/Euclidean_distance)
/// between consecutive points. For closed polygons the caller should include
/// the closing segment by repeating the first point at the end.
pub fn perimeter(points: &[Point]) -> Unit {
    if points.len() < 2 {
        return Unit::default_integer(0);
    }

    let first_point = points[0];
    let units = first_point.units().0;

    let points = ensure_points_same_units(points, units);

    let mut length = 0.0;
    for i in 0..points.len() - 1 {
        let dx = points[i + 1].x().float_value() - points[i].x().float_value();
        let dy = points[i + 1].y().float_value() - points[i].y().float_value();
        length += (dx * dx + dy * dy).sqrt();
    }

    Unit::float(length, units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perimeter_closed_square() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
            Point::float(0.0, 0.0, 1e-6),
        ];
        assert_eq!(perimeter(&square), Unit::float(8.0, 1e-6));
    }

    #[test]
    fn test_perimeter_open_polygon() {
        let line = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(3.0, 0.0, 1e-6),
            Point::float(3.0, 4.0, 1e-6),
        ];
        assert_eq!(perimeter(&line), Unit::float(7.0, 1e-6));
    }

    #[test]
    fn test_perimeter_triangle() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(3.0, 0.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
            Point::float(0.0, 0.0, 1e-6),
        ];
        assert_eq!(perimeter(&triangle), Unit::float(12.0, 1e-6));
    }

    #[test]
    fn test_perimeter_single_point() {
        let points = [Point::integer(1, 1, 1e-9)];
        assert_eq!(perimeter(&points), Unit::float(0.0, 1e-6));
    }

    #[test]
    fn test_perimeter_two_points() {
        let points = [Point::float(0.0, 0.0, 1e-6), Point::float(3.0, 4.0, 1e-6)];
        assert_eq!(perimeter(&points), Unit::float(5.0, 1e-6));
    }
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn perimeter_is_non_negative(coords: Vec<(i16, i16)>) -> bool {
        let points: Vec<Point> = coords
            .iter()
            .map(|&(x, y)| Point::float(f64::from(x), f64::from(y), 1e-6))
            .collect();
        perimeter(&points).float_value() >= 0.0
    }

    /// A single segment's perimeter equals its Euclidean length.
    #[quickcheck]
    fn perimeter_of_segment_equals_distance(x1: i16, y1: i16, x2: i16, y2: i16) -> bool {
        let a = Point::float(f64::from(x1), f64::from(y1), 1e-6);
        let b = Point::float(f64::from(x2), f64::from(y2), 1e-6);
        let dx = f64::from(x2) - f64::from(x1);
        let dy = f64::from(y2) - f64::from(y1);
        let expected = (dx * dx + dy * dy).sqrt();
        (perimeter(&[a, b]).float_value() - expected).abs() < 1e-10
    }
}
