use quickcheck_macros::quickcheck;

use crate::*;

const MAX_COORD: i32 = 10_000;

fn point_approx_eq(a: &Point, b: &Point, epsilon: f64) -> bool {
    (a.x() - b.x()).absolute_value().abs() < epsilon
        && (a.y() - b.y()).absolute_value().abs() < epsilon
}

/// Identity transformation does not change a point.
#[quickcheck]
fn identity_does_not_change_point(x: i32, y: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let identity = Transformation::default();
    identity.apply_to_point(&point) == point
}

/// Double reflection with the same axis cancels out.
#[quickcheck]
fn double_reflection_cancels(reflection: Reflection, x: i32, y: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let once = reflection.apply_to_point(&point);
    let twice = reflection.apply_to_point(&once);
    point_approx_eq(&twice, &point, 1e-6)
}

/// Rotation by 0 is identity.
#[quickcheck]
fn rotation_by_zero_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let centre = Point::integer(cx, cy, 1e-9);
    let rotation = Rotation::new(0.0, centre);
    let result = rotation.apply_to_point(&point);
    point_approx_eq(&result, &point, 1e-6)
}

/// Rotation by 2*pi is approximately identity.
#[quickcheck]
fn rotation_by_two_pi_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let centre = Point::integer(cx, cy, 1e-9);
    let rotation = Rotation::new(std::f64::consts::TAU, centre);
    let result = rotation.apply_to_point(&point);
    point_approx_eq(&result, &point, 1e-6)
}

/// Scale by 1 is identity.
#[quickcheck]
fn scale_by_one_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let centre = Point::integer(cx, cy, 1e-9);
    let scale = Scale::new(1.0, centre);
    let result = scale.apply_to_point(&point);
    point_approx_eq(&result, &point, 1e-6)
}

/// Translation composition: translate(a) then translate(b) == translate(a + b).
#[quickcheck]
fn translation_composition(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> bool {
    let ax = (ax % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let ay = (ay % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let bx = (bx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let by = (by % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let px = (px % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let py = (py % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);

    let point = Point::integer(px, py, 1e-9);
    let t_a = Translation::new(Point::integer(ax, ay, 1e-9));
    let t_b = Translation::new(Point::integer(bx, by, 1e-9));
    let t_ab = Translation::new(Point::integer(ax + bx, ay + by, 1e-9));

    let sequential = t_b.apply_to_point(&t_a.apply_to_point(&point));
    let composed = t_ab.apply_to_point(&point);
    point_approx_eq(&sequential, &composed, 1e-6)
}

/// Scale then inverse scale returns to the original point.
#[quickcheck]
fn scale_then_inverse_is_identity(x: i32, y: i32, cx: i32, cy: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cx = (cx % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let cy = (cy % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let centre = Point::integer(cx, cy, 1e-9);

    let factor = 2.5;
    let scale = Scale::new(factor, centre);
    let inverse_scale = Scale::new(1.0 / factor, centre);

    let result = inverse_scale.apply_to_point(&scale.apply_to_point(&point));
    point_approx_eq(&result, &point, 1e-6)
}

/// Rotation then inverse rotation returns to the original point.
#[quickcheck]
fn rotation_then_inverse_is_identity(rotation: Rotation, x: i32, y: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let inverse = Rotation::new(-rotation.angle(), *rotation.centre());

    let result = inverse.apply_to_point(&rotation.apply_to_point(&point));
    point_approx_eq(&result, &point, 1e-6)
}

/// Translation then inverse translation returns to the original point.
#[quickcheck]
fn translation_then_inverse_is_identity(translation: Translation, x: i32, y: i32) -> bool {
    let x = (x % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let y = (y % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
    let point = Point::integer(x, y, 1e-9);
    let applied = translation.apply_to_point(&point);
    let delta = applied - point;
    let inverse = Translation::new(delta * -1);

    let result = inverse.apply_to_point(&applied);
    point_approx_eq(&result, &point, 1e-6)
}
