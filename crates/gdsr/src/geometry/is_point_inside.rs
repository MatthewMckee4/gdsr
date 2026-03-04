use crate::{Point, elements::polygon::get_correct_polygon_points_format};

/// Check if a point is strictly inside a polygon.
///
/// Uses the [ray casting algorithm](https://en.wikipedia.org/wiki/Point_in_polygon#Ray_casting_algorithm).
/// Points on the boundary are not considered inside.
pub fn is_point_inside(point: &Point, points: &[Point]) -> bool {
    if points.len() < 3 {
        return false;
    }

    let closed = get_correct_polygon_points_format(points.to_vec());

    let px = point.x().float_value();
    let py = point.y().float_value();
    let n = closed.len() - 1;

    // Exclude points on the boundary (edges and vertices)
    for i in 0..n {
        let xi = closed[i].x().float_value();
        let yi = closed[i].y().float_value();
        let xn = closed[i + 1].x().float_value();
        let yn = closed[i + 1].y().float_value();

        let cross = (px - xi) * (yn - yi) - (py - yi) * (xn - xi);
        if cross.abs() <= 1e-10 {
            let in_x = px >= xi.min(xn) && px <= xi.max(xn);
            let in_y = py >= yi.min(yn) && py <= yi.max(yn);
            if in_x && in_y {
                return false;
            }
        }
    }

    // Ray casting
    let mut inside = false;
    let mut j = n - 1;

    for i in 0..n {
        let xi = closed[i].x().float_value();
        let yi = closed[i].y().float_value();
        let xj = closed[j].x().float_value();
        let yj = closed[j].y().float_value();

        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_inside() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];

        assert!(is_point_inside(&Point::integer(1, 1, 1e-9), &square));
        assert!(!is_point_inside(&Point::integer(3, 3, 1e-9), &square));
    }

    #[test]
    fn test_point_inside_too_few_points() {
        let square = [Point::float(0.0, 0.0, 1e-6), Point::float(2.0, 0.0, 1e-6)];
        assert!(!is_point_inside(&Point::integer(0, 0, 1e-9), &square));
    }

    #[test]
    fn test_point_inside_edge_cases() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];

        assert!(!is_point_inside(&Point::float(0.0, 0.0, 1e-6), &square));
        assert!(!is_point_inside(&Point::float(1.0, 0.0, 1e-6), &square));
        assert!(is_point_inside(&Point::float(1.0, 1.0, 1e-6), &square));
        assert!(!is_point_inside(&Point::float(-1.0, 1.0, 1e-6), &square));
        assert!(!is_point_inside(&Point::float(3.0, 1.0, 1e-6), &square));
    }

    #[test]
    fn test_is_point_inside_concave_polygon() {
        let concave = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(4.0, 0.0, 1e-6),
            Point::float(4.0, 4.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
        ];
        assert!(is_point_inside(&Point::float(1.0, 1.0, 1e-6), &concave));
        assert!(!is_point_inside(&Point::float(2.0, 3.0, 1e-6), &concave));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    /// Points far outside a polygon centered at the origin should never be inside.
    #[quickcheck]
    fn far_away_point_is_outside(offset: u16) -> bool {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(1.0, 0.0, 1e-6),
            Point::float(1.0, 1.0, 1e-6),
            Point::float(0.0, 1.0, 1e-6),
        ];
        let far = f64::from(offset) + 2.0;
        !is_point_inside(&Point::float(far, far, 1e-6), &square)
    }

    /// The centroid of a convex axis-aligned rectangle should be inside.
    #[quickcheck]
    fn centroid_of_rectangle_is_inside(w: u8, h: u8) -> bool {
        let w = f64::from(w.max(1));
        let h = f64::from(h.max(1));
        let rect = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(w, 0.0, 1e-6),
            Point::float(w, h, 1e-6),
            Point::float(0.0, h, 1e-6),
        ];
        is_point_inside(&Point::float(w / 2.0, h / 2.0, 1e-6), &rect)
    }
}
