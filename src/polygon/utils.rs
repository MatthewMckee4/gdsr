use log::info;

use crate::point::Point;

fn are_points_closed(points: &[Point]) -> bool {
    points.first() == points.last()
}

pub fn close_points(points: &[Point]) -> Vec<Point> {
    if are_points_closed(points) {
        points.to_vec()
    } else {
        info!("The points {:?} are not closed, closing them", points);
        let mut closed_points = points.to_vec();
        closed_points.push(points[0]);
        closed_points
    }
}

pub fn get_correct_polygon_points_format(points: Vec<Point>) -> Vec<Point> {
    close_points(&points)
}
