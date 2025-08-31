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

pub fn get_correct_polygon_points_format<T: CoordNum>(points: Vec<Point<T>>) -> Vec<Point<T>> {
    close_points(&points)
}
