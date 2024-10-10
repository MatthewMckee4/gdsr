use crate::point::Point;

pub fn get_points_from_i32_vec(vec: Vec<i32>) -> Vec<Point> {
    vec.chunks(2)
        .map(|chunk| Point::new(chunk[0] as f64, chunk[1] as f64))
        .collect()
}

pub fn points_are_close(points1: &[Point], points2: &[Point]) -> bool {
    if points1.len() != points2.len() {
        return false;
    }
    for (p1, p2) in points1.iter().zip(points2.iter()) {
        if !p1.epsilon_is_close(*p2) {
            return false;
        }
    }
    true
}
