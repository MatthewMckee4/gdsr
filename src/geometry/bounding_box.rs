use crate::Point;

/// Calculate the bounding box of a collection of points.
///
/// Iterates all points and tracks min/max x/y via `f64::min`/`f64::max`.
/// Returns (`min_point`, `max_point`) representing the bottom-left and top-right corners.
pub fn bounding_box(points: &[Point]) -> (Point, Point) {
    if points.is_empty() {
        return (Point::default(), Point::default());
    }

    let first_point = points[0];
    let (x_units, y_units) = first_point.units();

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for p in points {
        let x = p.x().float_value();
        let y = p.y().float_value();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    let min_point = Point::float(min_x, min_y, x_units);
    let max_point = Point::float(max_x, max_y, y_units);
    (min_point, max_point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point::integer(1, 2, 1e-9),
            Point::integer(4, 6, 1e-9),
            Point::integer(-1, 3, 1e-9),
            Point::integer(2, -1, 1e-9),
        ];

        let (min_point, max_point) = bounding_box(&points);
        assert_eq!(min_point, Point::integer(-1, -1, 1e-9));
        assert_eq!(max_point, Point::integer(4, 6, 1e-9));
    }

    #[test]
    fn test_bounding_box_empty() {
        let empty: [Point; 0] = [];
        let (min, max) = bounding_box(&empty);
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }

    #[test]
    fn test_bounding_box_single_point() {
        let points = [Point::integer(5, 3, 1e-9)];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(5, 3, 1e-9));
        assert_eq!(max, Point::integer(5, 3, 1e-9));
    }

    #[test]
    fn test_bounding_box_collinear_points() {
        let points = vec![
            Point::integer(1, 5, 1e-9),
            Point::integer(3, 5, 1e-9),
            Point::integer(7, 5, 1e-9),
        ];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(1, 5, 1e-9));
        assert_eq!(max, Point::integer(7, 5, 1e-9));
    }

    #[test]
    fn test_bounding_box_large_coordinates() {
        let points = vec![
            Point::integer(1_000_000, -2_000_000, 1e-9),
            Point::integer(-3_000_000, 4_000_000, 1e-9),
        ];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(-3_000_000, -2_000_000, 1e-9));
        assert_eq!(max, Point::integer(1_000_000, 4_000_000, 1e-9));
    }
}

#[cfg(test)]
#[allow(clippy::needless_pass_by_value)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn bounding_box_contains_all_points(coords: Vec<(i32, i32)>) -> bool {
        if coords.is_empty() {
            return true;
        }
        let points: Vec<Point> = coords
            .iter()
            .map(|&(x, y)| Point::integer(x, y, 1e-9))
            .collect();
        let (min, max) = bounding_box(&points);
        points.iter().all(|p| {
            p.x().float_value() >= min.x().float_value()
                && p.x().float_value() <= max.x().float_value()
                && p.y().float_value() >= min.y().float_value()
                && p.y().float_value() <= max.y().float_value()
        })
    }

    #[quickcheck]
    fn single_point_bounding_box_is_point_itself(x: i32, y: i32) -> bool {
        let p = Point::integer(x, y, 1e-9);
        let (min, max) = bounding_box(&[p]);
        min == p && max == p
    }
}
