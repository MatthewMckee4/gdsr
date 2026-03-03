use crate::Point;

/// Check if a point lies on the edge of a polygon.
///
/// Iterates each edge and delegates to [`is_point_on_line_segment`].
pub fn is_point_on_edge(point: &Point, points: &[Point]) -> bool {
    if points.len() < 2 {
        return false;
    }

    let num_points = points.len();
    for i in 0..num_points {
        let start = &points[i];
        let end = &points[(i + 1) % num_points];

        if is_point_on_line_segment(point, start, end) {
            return true;
        }
    }
    false
}

/// Check if a point lies on a line segment.
///
/// Tests collinearity via the [cross product](https://en.wikipedia.org/wiki/Cross_product)
/// and containment via the dot product.
/// For non-degenerate segments, endpoints are excluded.
/// For degenerate segments (a == b), returns true if the point equals the endpoint.
pub fn is_point_on_line_segment(point: &Point, a: &Point, b: &Point) -> bool {
    let px = point.x().float_value();
    let py = point.y().float_value();
    let ax = a.x().float_value();
    let ay = a.y().float_value();
    let bx = b.x().float_value();
    let by = b.y().float_value();

    let len_sq = (bx - ax) * (bx - ax) + (by - ay) * (by - ay);

    if len_sq == 0.0 {
        return px == ax && py == ay;
    }

    let cross = (px - ax) * (by - ay) - (py - ay) * (bx - ax);
    if cross.abs() > 1e-10 {
        return false;
    }

    let dot = (px - ax) * (bx - ax) + (py - ay) * (by - ay);
    dot > 0.0 && dot < len_sq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_on_edge() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];

        assert!(is_point_on_edge(&Point::integer(1, 0, 1e-9), &triangle));
        assert!(!is_point_on_edge(&Point::integer(1, 1, 1e-9), &triangle));
    }

    #[test]
    fn test_point_on_edge_too_few_points() {
        let triangle = [Point::float(0.0, 0.0, 1e-6)];
        assert!(!is_point_on_edge(&Point::integer(0, 0, 1e-9), &triangle));
    }

    #[test]
    fn test_point_on_edge_midpoint() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];

        assert!(is_point_on_edge(&Point::float(1.0, 0.0, 1e-6), &triangle));
        assert!(!is_point_on_edge(&Point::float(0.5, 0.5, 1e-6), &triangle));
    }

    /// Vertices are not considered "on edge" because `is_point_on_line_segment` excludes endpoints.
    #[test]
    fn test_is_point_on_edge_vertex() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];
        assert!(!is_point_on_edge(&Point::float(0.0, 0.0, 1e-6), &triangle));
        assert!(!is_point_on_edge(&Point::float(2.0, 0.0, 1e-6), &triangle));
        assert!(!is_point_on_edge(&Point::float(1.0, 2.0, 1e-6), &triangle));
    }

    #[test]
    fn test_is_point_on_line_segment() {
        let a = Point::float(0.0, 0.0, 1e-6);
        let b = Point::float(2.0, 2.0, 1e-6);

        assert!(is_point_on_line_segment(
            &Point::float(1.0, 1.0, 1e-6),
            &a,
            &b
        ));

        assert!(!is_point_on_line_segment(
            &Point::float(1.0, 0.0, 1e-6),
            &a,
            &b
        ));

        assert!(!is_point_on_line_segment(
            &Point::float(3.0, 3.0, 1e-6),
            &a,
            &b
        ));
    }

    #[test]
    fn test_degenerate_segment() {
        let a = Point::float(1.0, 1.0, 1e-6);
        assert!(is_point_on_line_segment(
            &Point::float(1.0, 1.0, 1e-6),
            &a,
            &a
        ));
        assert!(!is_point_on_line_segment(
            &Point::float(2.0, 2.0, 1e-6),
            &a,
            &a
        ));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    /// The midpoint of a non-degenerate segment should always be on the segment.
    #[quickcheck]
    fn midpoint_is_on_segment(x1: i16, y1: i16, x2: i16, y2: i16) -> bool {
        if x1 == x2 && y1 == y2 {
            return true;
        }
        let a = Point::float(f64::from(x1), f64::from(y1), 1e-6);
        let b = Point::float(f64::from(x2), f64::from(y2), 1e-6);
        let mid = Point::float(
            f64::midpoint(f64::from(x1), f64::from(x2)),
            f64::midpoint(f64::from(y1), f64::from(y2)),
            1e-6,
        );
        is_point_on_line_segment(&mid, &a, &b)
    }

    /// Endpoints should never be considered on a non-degenerate segment.
    #[quickcheck]
    fn endpoints_excluded_from_segment(x1: i16, y1: i16, x2: i16, y2: i16) -> bool {
        if x1 == x2 && y1 == y2 {
            return true;
        }
        let a = Point::float(f64::from(x1), f64::from(y1), 1e-6);
        let b = Point::float(f64::from(x2), f64::from(y2), 1e-6);
        !is_point_on_line_segment(&a, &a, &b) && !is_point_on_line_segment(&b, &a, &b)
    }
}
