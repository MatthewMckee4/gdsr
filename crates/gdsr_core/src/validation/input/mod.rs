use crate::Point;

pub fn check_layer_valid(layer: i32) -> Result<(), String> {
    if !(0..=255).contains(&layer) {
        return Err("Layer must be in the range 0-255".to_string());
    }
    Ok(())
}

pub fn check_data_type_valid(_: i32) -> Result<(), String> {
    Ok(())
}

pub fn check_points_vec_has_at_least_two_points(points: &[Point]) -> Result<(), String> {
    if points.len() < 2 {
        return Err("Path must have at least two points".to_string());
    }
    Ok(())
}
