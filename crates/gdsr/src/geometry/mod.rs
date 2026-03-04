mod area;
mod bounding_box;
mod is_point_inside;
mod is_point_on_edge;
mod perimeter;
mod round;

pub use area::area;
pub use bounding_box::bounding_box;
pub use is_point_inside::is_point_inside;
pub use is_point_on_edge::is_point_on_edge;
pub use perimeter::perimeter;
pub use round::round_to_decimals;

/// Ensure all points have the same units
fn ensure_points_same_units(points: &[crate::Point], new_units: f64) -> Vec<crate::Point> {
    points
        .iter()
        .map(|point| point.scale_units(new_units))
        .collect()
}
