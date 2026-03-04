use egui::{Pos2, Rect};
use quickcheck_macros::quickcheck;

use crate::spatial::SpatialGrid;
use crate::testutil::helpers;
use crate::viewport::Viewport;
use crate::viewport::bounds::{bbox_overlaps, compute_bounds, element_bbox};

fn test_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
}

fn is_finite(v: f64) -> bool {
    v.is_finite()
}

/// Clamp to reasonable ranges to avoid f32 precision loss during conversion.
fn clamp_viewport(cx: f64, cy: f64, zoom: f64) -> Option<Viewport> {
    if !is_finite(cx) || !is_finite(cy) || !is_finite(zoom) {
        return None;
    }
    Some(Viewport {
        center_x: cx.clamp(-1e6, 1e6),
        center_y: cy.clamp(-1e6, 1e6),
        zoom: zoom.abs().clamp(1e-3, 1e6),
    })
}

#[quickcheck]
fn world_to_screen_roundtrip(cx: f64, cy: f64, zoom: f64, wx: f64, wy: f64) -> bool {
    let Some(vp) = clamp_viewport(cx, cy, zoom) else {
        return true;
    };
    if !is_finite(wx) || !is_finite(wy) {
        return true;
    }
    let rect = test_rect();
    let wx = wx.clamp(-1e6, 1e6);
    let wy = wy.clamp(-1e6, 1e6);

    let screen = vp.world_to_screen(wx, wy, rect);
    let (rx, ry) = vp.screen_to_world(screen.x, screen.y, rect);

    (rx - wx).abs() < 1.0 && (ry - wy).abs() < 1.0
}

#[quickcheck]
fn zoom_preserves_anchor(cx: f64, cy: f64, zoom: f64, sx: f32, sy: f32, factor: f64) -> bool {
    let Some(mut vp) = clamp_viewport(cx, cy, zoom) else {
        return true;
    };
    if !sx.is_finite() || !sy.is_finite() || !is_finite(factor) {
        return true;
    }
    let rect = test_rect();
    let sx = sx.clamp(0.0, 800.0);
    let sy = sy.clamp(0.0, 600.0);
    let factor = factor.clamp(0.5, 2.0);

    let (wx, wy) = vp.screen_to_world(sx, sy, rect);
    let new_zoom = (vp.zoom * factor).clamp(1e-3, 1e6);
    let screen_cx = f64::from(rect.center().x);
    let screen_cy = f64::from(rect.center().y);
    vp.center_x = wx - (f64::from(sx) - screen_cx) / new_zoom;
    vp.center_y = wy + (f64::from(sy) - screen_cy) / new_zoom;
    vp.zoom = new_zoom;

    let after = vp.world_to_screen(wx, wy, rect);
    (f64::from(after.x) - f64::from(sx)).abs() < 1.0
        && (f64::from(after.y) - f64::from(sy)).abs() < 1.0
}

#[quickcheck]
fn zoom_to_fit_makes_bounds_visible(min_x: f64, min_y: f64, width: f64, height: f64) -> bool {
    if !is_finite(min_x) || !is_finite(min_y) || !is_finite(width) || !is_finite(height) {
        return true;
    }
    let min_x = min_x.clamp(-1e6, 1e6);
    let min_y = min_y.clamp(-1e6, 1e6);
    let width = width.abs().clamp(1.0, 1e6);
    let height = height.abs().clamp(1.0, 1e6);
    let max_x = min_x + width;
    let max_y = min_y + height;

    let mut vp = Viewport::default();
    let rect = test_rect();
    vp.zoom_to_fit(min_x, min_y, max_x, max_y, rect);

    let s_min = vp.world_to_screen(min_x, min_y, rect);
    let s_max = vp.world_to_screen(max_x, max_y, rect);

    s_min.x >= rect.min.x - 1.0
        && s_max.x <= rect.max.x + 1.0
        && s_max.y >= rect.min.y - 1.0
        && s_min.y <= rect.max.y + 1.0
}

#[quickcheck]
fn element_bbox_contains_all_points(x1: i16, y1: i16, x2: i16, y2: i16, x3: i16, y3: i16) -> bool {
    let elem = helpers::polygon(
        vec![
            (i32::from(x1), i32::from(y1)),
            (i32::from(x2), i32::from(y2)),
            (i32::from(x3), i32::from(y3)),
        ],
        1,
        0,
    );
    let Some(bbox) = element_bbox(&elem) else {
        return false;
    };

    let points = [(x1, y1), (x2, y2), (x3, y3)];
    let scale = 1e-9;
    points.iter().all(|&(x, y)| {
        let wx = f64::from(x) * scale;
        let wy = f64::from(y) * scale;
        wx >= bbox[0] - 1e-15
            && wx <= bbox[2] + 1e-15
            && wy >= bbox[1] - 1e-15
            && wy <= bbox[3] + 1e-15
    })
}

#[quickcheck]
fn compute_bounds_is_superset(
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    x3: i16,
    y3: i16,
    x4: i16,
    y4: i16,
) -> bool {
    let elems = vec![
        helpers::polygon(
            vec![
                (i32::from(x1), i32::from(y1)),
                (i32::from(x2), i32::from(y2)),
                (i32::from(x3), i32::from(y3)),
            ],
            1,
            0,
        ),
        helpers::polygon(
            vec![
                (i32::from(x2), i32::from(y2)),
                (i32::from(x3), i32::from(y3)),
                (i32::from(x4), i32::from(y4)),
            ],
            2,
            0,
        ),
    ];

    let Some((cb_min_x, cb_min_y, cb_max_x, cb_max_y)) = compute_bounds(&elems) else {
        return false;
    };

    elems.iter().all(|e| {
        if let Some(bbox) = element_bbox(e) {
            bbox[0] >= cb_min_x - 1e-15
                && bbox[1] >= cb_min_y - 1e-15
                && bbox[2] <= cb_max_x + 1e-15
                && bbox[3] <= cb_max_y + 1e-15
        } else {
            true
        }
    })
}

#[quickcheck]
fn bbox_overlaps_is_symmetric(
    ax: f64,
    ay: f64,
    aw: f64,
    ah: f64,
    bx: f64,
    by: f64,
    bw: f64,
    bh: f64,
) -> bool {
    let vals = [ax, ay, aw, ah, bx, by, bw, bh];
    if vals.iter().any(|v| !v.is_finite()) {
        return true;
    }
    let a = [ax, ay, ax + aw.abs(), ay + ah.abs()];
    let b = [bx, by, bx + bw.abs(), by + bh.abs()];
    bbox_overlaps(&a, &b) == bbox_overlaps(&b, &a)
}

#[quickcheck]
fn spatial_grid_full_query_finds_all(x1: i8, y1: i8, x2: i8, y2: i8) -> bool {
    let scale = 1e-9;
    let elems = vec![
        helpers::polygon(
            vec![
                (i32::from(x1), i32::from(y1)),
                (i32::from(x1) + 10, i32::from(y1)),
                (i32::from(x1) + 10, i32::from(y1) + 10),
            ],
            1,
            0,
        ),
        helpers::polygon(
            vec![
                (i32::from(x2), i32::from(y2)),
                (i32::from(x2) + 10, i32::from(y2)),
                (i32::from(x2) + 10, i32::from(y2) + 10),
            ],
            2,
            0,
        ),
    ];

    let Some(bounds) = compute_bounds(&elems) else {
        return false;
    };
    let grid = SpatialGrid::build(&elems, bounds);

    let all_x: Vec<f64> = [x1, x2]
        .iter()
        .flat_map(|&x| [f64::from(x) * scale, (f64::from(x) + 10.0) * scale])
        .collect();
    let all_y: Vec<f64> = [y1, y2]
        .iter()
        .flat_map(|&y| [f64::from(y) * scale, (f64::from(y) + 10.0) * scale])
        .collect();

    let min_x = all_x.iter().copied().fold(f64::MAX, f64::min);
    let max_x = all_x.iter().copied().fold(f64::MIN, f64::max);
    let min_y = all_y.iter().copied().fold(f64::MAX, f64::min);
    let max_y = all_y.iter().copied().fold(f64::MIN, f64::max);

    let visible = [min_x, min_y, max_x, max_y];
    let mut indices: Vec<u32> = grid
        .query_visible(&visible)
        .flat_map(|c| c.indices.iter().copied())
        .collect();
    indices.sort_unstable();
    indices.dedup();

    indices.contains(&0) && indices.contains(&1)
}
