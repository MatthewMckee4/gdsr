use crate::Point;

fn are_points_closed(points: &[Point]) -> bool {
    let points_vec: Vec<Point> = points.to_vec();
    if points_vec.is_empty() {
        return true;
    }
    points_vec.first() == points_vec.last()
}

pub fn close_points(points: impl IntoIterator<Item = Point>) -> Vec<Point> {
    let mut points_vec = points.into_iter().collect::<Vec<_>>();
    if !are_points_closed(&points_vec) {
        if let Some(first) = points_vec.first().copied() {
            points_vec.push(first);
        }
    }
    points_vec
}

pub fn get_correct_polygon_points_format(points: impl IntoIterator<Item = Point>) -> Vec<Point> {
    close_points(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_points_not_closed() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(5, 0, 1e-9),
            Point::integer(5, 5, 1e-9),
        ];
        let closed = close_points(points.clone());

        assert_eq!(closed.len(), 4);
        assert_eq!(closed[0], points[0]);
        assert_eq!(closed[closed.len() - 1], points[0]);
    }

    #[test]
    fn test_close_points_already_closed() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(5, 0, 1e-9),
            Point::integer(5, 5, 1e-9),
            Point::integer(0, 0, 1e-9),
        ];
        let closed = close_points(points.clone());

        assert_eq!(closed.len(), points.len());
    }

    #[test]
    fn test_close_points_empty() {
        let points: Vec<Point> = vec![];
        let closed = close_points(points);

        assert_eq!(closed.len(), 0);
    }
}
