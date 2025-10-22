use crate::Point;

fn are_points_closed(points: &[Point]) -> bool {
    let points_vec: Vec<Point> = points.to_vec();
    if points_vec.is_empty() {
        return true;
    }
    points_vec.first() == points_vec.last()
}

pub fn close_points(
    points: impl IntoIterator<Item = impl Into<Point>>,
) -> Vec<Point> {
    let mut points_vec = points.into_iter().map(Into::into).collect::<Vec<_>>();
    if !are_points_closed(&points_vec) {
        if let Some(first) = points_vec.first().copied() {
            points_vec.push(first);
        }
    }
    points_vec
}

pub fn get_correct_polygon_points_format(
    points: impl IntoIterator<Item = impl Into<Point>>,
) -> Vec<Point> {
    close_points(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_points_not_closed() {
        let points = vec![
            Point::from([0, 0]),
            Point::from([5, 0]),
            Point::from([5, 5]),
        ];
        let closed = close_points(points.clone());

        // Should add closing point
        assert_eq!(closed.len(), 4);
        assert_eq!(closed[0], points[0]);
        assert_eq!(closed[closed.len() - 1], points[0]); // Last point should match first
    }

    #[test]
    fn test_close_points_already_closed() {
        let points = vec![
            Point::from([0, 0]),
            Point::from([5, 0]),
            Point::from([5, 5]),
            Point::from([0, 0]), // Already closed
        ];
        let closed = close_points(points.clone());

        // Should return same points since already closed
        assert_eq!(closed.len(), points.len());
    }

    #[test]
    fn test_close_points_empty() {
        let points: Vec<Point> = vec![];
        let closed = close_points(points);

        // Empty should remain empty
        assert_eq!(closed.len(), 0);
    }
}
