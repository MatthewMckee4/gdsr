use crate::PathType;

/// Expands a centerline path into a closed polygon outline.
///
/// Returns the polygon vertices that represent the filled area of the path,
/// accounting for the path width and end-cap type. Returns an empty vec if
/// fewer than 2 centerline points or non-positive `half_width`.
pub fn expand_path_to_polygon(
    centerline: &[(f64, f64)],
    half_width: f64,
    path_type: PathType,
    begin_extension: f64,
    end_extension: f64,
    arc_segments: usize,
) -> Vec<(f64, f64)> {
    if centerline.len() < 2 || half_width <= 0.0 {
        return Vec::new();
    }

    let pts = apply_extensions(centerline, path_type, begin_extension, end_extension);
    let n = pts.len();

    let normals: Vec<(f64, f64)> = (0..n - 1)
        .map(|i| segment_normal(pts[i], pts[i + 1]))
        .collect();

    let mut left = Vec::with_capacity(n);
    let mut right = Vec::with_capacity(n);

    // First point
    left.push(offset(pts[0], normals[0], half_width));
    right.push(offset(pts[0], normals[0], -half_width));

    // Interior points: miter join with bevel fallback
    for i in 1..n - 1 {
        miter_or_bevel(
            pts[i],
            normals[i - 1],
            normals[i],
            half_width,
            &mut left,
            &mut right,
        );
    }

    // Last point
    left.push(offset(pts[n - 1], normals[n - 2], half_width));
    right.push(offset(pts[n - 1], normals[n - 2], -half_width));

    // Build closed polygon: left forward + end cap + right reversed + begin cap
    let mut polygon = Vec::with_capacity(left.len() + right.len() + 2 * arc_segments);

    match path_type {
        PathType::Round => {
            polygon.extend_from_slice(&left);
            polygon.extend(semicircle_cap(
                pts[n - 1],
                normals[n - 2],
                half_width,
                false,
                arc_segments,
            ));
            polygon.extend(right.into_iter().rev());
            polygon.extend(semicircle_cap(
                pts[0],
                normals[0],
                half_width,
                true,
                arc_segments,
            ));
        }
        PathType::Square | PathType::Overlap => {
            polygon.extend_from_slice(&left);
            right.reverse();
            polygon.extend_from_slice(&right);
        }
    }

    polygon
}

fn apply_extensions(
    centerline: &[(f64, f64)],
    path_type: PathType,
    begin_ext: f64,
    end_ext: f64,
) -> Vec<(f64, f64)> {
    let mut pts: Vec<(f64, f64)> = centerline.to_vec();
    if path_type != PathType::Overlap {
        return pts;
    }
    let n = pts.len();

    // Extend first point backward along first segment
    if begin_ext.abs() > f64::EPSILON {
        let (dx, dy) = direction(pts[0], pts[1]);
        pts[0].0 -= dx * begin_ext;
        pts[0].1 -= dy * begin_ext;
    }

    // Extend last point forward along last segment
    if end_ext.abs() > f64::EPSILON {
        let (dx, dy) = direction(pts[n - 2], pts[n - 1]);
        pts[n - 1].0 += dx * end_ext;
        pts[n - 1].1 += dy * end_ext;
    }

    pts
}

fn direction(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        return (0.0, 0.0);
    }
    (dx / len, dy / len)
}

fn segment_normal(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (dx, dy) = direction(a, b);
    (-dy, dx)
}

fn offset(p: (f64, f64), normal: (f64, f64), dist: f64) -> (f64, f64) {
    (p.0 + normal.0 * dist, p.1 + normal.1 * dist)
}

/// Computes miter join at an interior vertex, pushing results into left/right buffers.
/// Falls back to bevel when the miter ratio exceeds 2.0.
fn miter_or_bevel(
    p: (f64, f64),
    n0: (f64, f64),
    n1: (f64, f64),
    half_width: f64,
    left: &mut Vec<(f64, f64)>,
    right: &mut Vec<(f64, f64)>,
) {
    let avg_x = n0.0 + n1.0;
    let avg_y = n0.1 + n1.1;
    let dot = avg_x * avg_x + avg_y * avg_y;

    if dot < 1e-10 || 1.0 / dot.sqrt() > 2.0 {
        // Nearly opposite normals or miter too long — use bevel
        left.push(offset(p, n0, half_width));
        left.push(offset(p, n1, half_width));
        right.push(offset(p, n1, -half_width));
        right.push(offset(p, n0, -half_width));
        return;
    }

    let scale = half_width * 2.0 / dot;
    let miter = (avg_x * scale, avg_y * scale);
    let s = dot.sqrt();

    left.push((p.0 + miter.0 / 2.0 * s, p.1 + miter.1 / 2.0 * s));
    right.push((p.0 - miter.0 / 2.0 * s, p.1 - miter.1 / 2.0 * s));
}

fn semicircle_cap(
    center: (f64, f64),
    normal: (f64, f64),
    half_width: f64,
    is_begin: bool,
    segments: usize,
) -> Vec<(f64, f64)> {
    let segments = segments.max(4);
    let mut pts = Vec::with_capacity(segments + 1);

    // The cap goes from one side of the normal to the other, sweeping π radians
    let start_angle = normal.1.atan2(normal.0);
    let sweep = if is_begin {
        std::f64::consts::PI
    } else {
        -std::f64::consts::PI
    };

    for i in 0..=segments {
        let t = i as f64 / segments as f64;
        let angle = start_angle + sweep * t;
        pts.push((
            center.0 + half_width * angle.cos(),
            center.1 + half_width * angle.sin(),
        ));
    }

    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_segment_square() {
        let pts = expand_path_to_polygon(
            &[(0.0, 0.0), (10.0, 0.0)],
            1.0,
            PathType::Square,
            0.0,
            0.0,
            8,
        );
        insta::assert_debug_snapshot!(pts, @r"
        [
            (
                0.0,
                1.0,
            ),
            (
                10.0,
                1.0,
            ),
            (
                10.0,
                -1.0,
            ),
            (
                0.0,
                -1.0,
            ),
        ]
        ");
    }

    #[test]
    fn horizontal_segment_overlap() {
        let pts = expand_path_to_polygon(
            &[(0.0, 0.0), (10.0, 0.0)],
            1.0,
            PathType::Overlap,
            2.0,
            3.0,
            8,
        );
        insta::assert_debug_snapshot!(pts, @r"
        [
            (
                -2.0,
                1.0,
            ),
            (
                13.0,
                1.0,
            ),
            (
                13.0,
                -1.0,
            ),
            (
                -2.0,
                -1.0,
            ),
        ]
        ");
    }

    #[test]
    fn horizontal_segment_round() {
        let pts = expand_path_to_polygon(
            &[(0.0, 0.0), (10.0, 0.0)],
            1.0,
            PathType::Round,
            0.0,
            0.0,
            4,
        );
        // Left side + end semicircle + right side reversed + begin semicircle
        assert!(pts.len() > 4, "Round cap should produce more points");
        // First and last left-side points
        assert!((pts[0].0 - 0.0).abs() < 1e-10);
        assert!((pts[0].1 - 1.0).abs() < 1e-10);
        assert!((pts[1].0 - 10.0).abs() < 1e-10);
        assert!((pts[1].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn ninety_degree_corner() {
        let pts = expand_path_to_polygon(
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)],
            1.0,
            PathType::Square,
            0.0,
            0.0,
            8,
        );
        // Should produce a closed polygon with miter at the corner
        assert!(pts.len() >= 6);
    }

    #[test]
    fn collinear_segments() {
        let pts = expand_path_to_polygon(
            &[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)],
            1.0,
            PathType::Square,
            0.0,
            0.0,
            8,
        );
        insta::assert_debug_snapshot!(pts, @r"
        [
            (
                0.0,
                1.0,
            ),
            (
                5.0,
                1.0,
            ),
            (
                10.0,
                1.0,
            ),
            (
                10.0,
                -1.0,
            ),
            (
                5.0,
                -1.0,
            ),
            (
                0.0,
                -1.0,
            ),
        ]
        ");
    }

    #[test]
    fn too_few_points_returns_empty() {
        assert!(
            expand_path_to_polygon(&[(0.0, 0.0)], 1.0, PathType::Square, 0.0, 0.0, 8).is_empty()
        );
        assert!(expand_path_to_polygon(&[], 1.0, PathType::Square, 0.0, 0.0, 8).is_empty());
    }

    #[test]
    fn zero_width_returns_empty() {
        assert!(
            expand_path_to_polygon(
                &[(0.0, 0.0), (10.0, 0.0)],
                0.0,
                PathType::Square,
                0.0,
                0.0,
                8
            )
            .is_empty()
        );
    }
}
