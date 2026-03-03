use quickcheck_macros::quickcheck;

use crate::*;

#[quickcheck]
fn double_reflection_cancels(text: Text) -> bool {
    let centre = Point::integer(0, 0, text.origin().units().0);
    let reflected_twice = text.clone().reflect(0.0, centre).reflect(0.0, centre);
    reflected_twice.x_reflection() == text.x_reflection()
}

#[quickcheck]
fn scale_multiplies_magnification(text: Text) -> bool {
    let centre = Point::integer(0, 0, text.origin().units().0);
    let scaled = text.clone().scale(2.0, centre);
    (scaled.magnification() - text.magnification() * 2.0).abs() < 1e-10
}
