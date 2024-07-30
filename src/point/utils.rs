use crate::point::Point;

pub fn get_points_from_i32_vec(vec: Vec<i32>) -> Vec<Point> {
    vec.chunks(2)
        .map(|chunk| Point::new(chunk[0] as f64, chunk[1] as f64))
        .collect()
}
