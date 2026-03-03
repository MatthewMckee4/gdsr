use quickcheck_macros::quickcheck;

use crate::*;

#[quickcheck]
fn double_reflection_cancels(grid: Grid) -> bool {
    let centre = Point::integer(0, 0, 1e-9);
    let transformed = grid.reflect(0.0, centre).reflect(0.0, centre);
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
