use crate::{CoordNum, Point};

fn are_points_closed<T: CoordNum>(points: &[Point<T>]) -> bool {
    points.first() == points.last()
}

pub fn close_points<T: CoordNum>(points: &[Point<T>]) -> Vec<Point<T>> {
    if are_points_closed(points) {
        points.to_vec()
    } else {
        let mut closed_points = points.to_vec();
        closed_points.push(points[0]);
        closed_points
    }
}

pub fn get_correct_polygon_points_format<T: CoordNum>(points: &[Point<T>]) -> Vec<Point<T>> {
    close_points(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseIntegerUnit;

    #[test]
    fn test_are_points_closed_true() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(1)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)), // Same as first
        ];
        assert!(are_points_closed(&points));
    }

    #[test]
    fn test_are_points_closed_false() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(1)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(1)), // Different from first
        ];
        assert!(!are_points_closed(&points));
    }

    #[test]
    fn test_are_points_closed_single_point() {
        let points = vec![Point::new(
            DatabaseIntegerUnit::from(5),
            DatabaseIntegerUnit::from(5),
        )];
        // Single point is considered closed (first == last)
        assert!(are_points_closed(&points));
    }

    #[test]
    fn test_are_points_closed_two_same_points() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(3), DatabaseIntegerUnit::from(4)),
            Point::new(DatabaseIntegerUnit::from(3), DatabaseIntegerUnit::from(4)),
        ];
        assert!(are_points_closed(&points));
    }

    #[test]
    fn test_are_points_closed_empty() {
        let points: Vec<Point<DatabaseIntegerUnit>> = vec![];
        // Empty vector: first() and last() both return None, so None == None is true
        assert!(are_points_closed(&points));
    }

    #[test]
    fn test_close_points_already_closed() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)), // Already closed
        ];
        let closed = close_points(&points);

        // Should return same points since already closed
        assert_eq!(closed.len(), points.len());
        assert_eq!(closed, points);
    }

    #[test]
    fn test_close_points_not_closed() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(5)), // Not closed
        ];
        let closed = close_points(&points);

        // Should add closing point
        assert_eq!(closed.len(), points.len() + 1);
        assert_eq!(closed[0], points[0]); // First point
        assert_eq!(closed[closed.len() - 1], points[0]); // Last point should match first

        // All original points should be preserved
        for (i, original_point) in points.iter().enumerate() {
            assert_eq!(closed[i], *original_point);
        }
    }

    #[test]
    fn test_close_points_single_point() {
        let points = vec![Point::new(
            DatabaseIntegerUnit::from(7),
            DatabaseIntegerUnit::from(3),
        )];
        let closed = close_points(&points);

        // Single point is already "closed", should return unchanged
        assert_eq!(closed.len(), 1);
        assert_eq!(closed, points);
    }

    #[test]
    fn test_close_points_two_different_points() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(2)),
            Point::new(DatabaseIntegerUnit::from(3), DatabaseIntegerUnit::from(4)),
        ];
        let closed = close_points(&points);

        // Should add closing point
        assert_eq!(closed.len(), 3);
        assert_eq!(closed[0], points[0]);
        assert_eq!(closed[1], points[1]);
        assert_eq!(closed[2], points[0]); // Closing point
    }

    #[test]
    fn test_close_points_empty() {
        let points: Vec<Point<DatabaseIntegerUnit>> = vec![];
        let closed = close_points(&points);

        // Empty should remain empty
        assert_eq!(closed.len(), 0);
        assert_eq!(closed, points);
    }

    #[test]
    fn test_get_correct_polygon_points_format_open() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(10), DatabaseIntegerUnit::from(20)),
            Point::new(DatabaseIntegerUnit::from(30), DatabaseIntegerUnit::from(20)),
            Point::new(DatabaseIntegerUnit::from(30), DatabaseIntegerUnit::from(40)),
        ];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Should close the polygon
        assert_eq!(formatted.len(), points.len() + 1);
        assert_eq!(formatted[0], points[0]);
        assert_eq!(formatted[formatted.len() - 1], points[0]);
    }

    #[test]
    fn test_get_correct_polygon_points_format_closed() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(10), DatabaseIntegerUnit::from(20)),
            Point::new(DatabaseIntegerUnit::from(30), DatabaseIntegerUnit::from(20)),
            Point::new(DatabaseIntegerUnit::from(30), DatabaseIntegerUnit::from(40)),
            Point::new(DatabaseIntegerUnit::from(10), DatabaseIntegerUnit::from(20)), // Already closed
        ];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Should remain unchanged
        assert_eq!(formatted.len(), points.len());
        assert_eq!(formatted, points);
    }

    #[test]
    fn test_get_correct_polygon_points_format_empty() {
        let points: Vec<Point<DatabaseIntegerUnit>> = vec![];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Empty should remain empty
        assert_eq!(formatted.len(), 0);
        assert_eq!(formatted, points);
    }

    #[test]
    fn test_get_correct_polygon_points_format_single() {
        let points = vec![Point::new(
            DatabaseIntegerUnit::from(100),
            DatabaseIntegerUnit::from(200),
        )];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Single point should remain single (already "closed")
        assert_eq!(formatted.len(), 1);
        assert_eq!(formatted, points);
    }

    #[test]
    fn test_polygon_formats_preserve_order() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(1)),
            Point::new(DatabaseIntegerUnit::from(2), DatabaseIntegerUnit::from(1)),
            Point::new(DatabaseIntegerUnit::from(3), DatabaseIntegerUnit::from(2)),
            Point::new(DatabaseIntegerUnit::from(2), DatabaseIntegerUnit::from(3)),
            Point::new(DatabaseIntegerUnit::from(1), DatabaseIntegerUnit::from(2)),
        ];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Should preserve order and add closing point
        assert_eq!(formatted.len(), points.len() + 1);
        for (i, point) in points.iter().enumerate() {
            assert_eq!(formatted[i], *point);
        }
        assert_eq!(formatted[formatted.len() - 1], points[0]); // Closing point
    }

    #[test]
    fn test_floating_point_coordinates() {
        let points = vec![
            Point::new(1.5f64, 2.5f64),
            Point::new(3.7f64, 2.5f64),
            Point::new(3.7f64, 4.1f64),
        ];
        let formatted = get_correct_polygon_points_format(points.clone());

        // Should work with floating point coordinates
        assert_eq!(formatted.len(), points.len() + 1);
        assert_eq!(formatted[0], points[0]);
        assert_eq!(formatted[formatted.len() - 1], points[0]);
    }
}
