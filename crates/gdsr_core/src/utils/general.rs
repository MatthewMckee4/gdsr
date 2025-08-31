use crate::Point;

pub fn check_points_vec_not_empty(vec: &[Point]) -> Result<(), String> {
    if vec.is_empty() {
        Err("Points cannot be empty".to_string())
    } else {
        Ok(())
    }
}
