use quickcheck_macros::quickcheck;

use crate::*;

#[quickcheck]
fn double_reflection_cancels(grid: Grid) -> bool {
    let centre = Point::integer(0, 0, 1e-9);
    let transformed = grid
        .reflect(Radians::new(0.0), centre)
        .reflect(Radians::new(0.0), centre);
    !transformed.x_reflection()
}

#[quickcheck]
fn translation_preserves_dimensions(grid: Grid) -> bool {
    let delta = Point::integer(42, -17, 1e-9);
    let translated = grid.clone().translate(delta);
    translated.columns() == grid.columns()
        && translated.rows() == grid.rows()
        && translated.magnification() == grid.magnification()
}

#[quickcheck]
fn translation_preserves_spacing(grid: Grid) -> bool {
    let delta = Point::integer(42, -17, 1e-9);
    let translated = grid.clone().translate(delta);
    translated.spacing_x() == grid.spacing_x() && translated.spacing_y() == grid.spacing_y()
}

#[quickcheck]
fn move_by_preserves_spacing(grid: Grid) -> bool {
    let delta = Point::integer(99, -33, 1e-9);
    let moved = grid.clone().move_by(delta);
    moved.spacing_x() == grid.spacing_x() && moved.spacing_y() == grid.spacing_y()
}
