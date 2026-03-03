use quickcheck_macros::quickcheck;

use crate::*;

#[quickcheck]
fn bounding_box_contains_all_points(path: Path) -> bool {
    if path.points().is_empty() {
        return true;
    }
    let (min, max) = path.bounding_box();
    path.points().iter().all(|p| {
        p.x().float_value() >= min.x().float_value()
            && p.x().float_value() <= max.x().float_value()
            && p.y().float_value() >= min.y().float_value()
            && p.y().float_value() <= max.y().float_value()
    })
}

#[quickcheck]
fn translation_preserves_point_count(path: Path, dx: i32, dy: i32) -> bool {
    let units = path.points()[0].units().0;
    let dx = (dx % 10_000).clamp(-10_000, 10_000);
    let dy = (dy % 10_000).clamp(-10_000, 10_000);
    let delta = Point::integer(dx, dy, units);
    let translated = path.clone().translate(delta);
    translated.points().len() == path.points().len()
}
