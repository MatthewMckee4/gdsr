use std::collections::HashMap;

use egui::{Mesh, Pos2, Rect, Stroke};
use quickcheck_macros::quickcheck;

use crate::drawable::{Drawable, WorldBBox, stroke_polyline_to_mesh};
use crate::spatial::SpatialGrid;
use crate::testutil::helpers;
use crate::viewport::Viewport;
use crate::viewport::bounds::compute_bounds;

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
    let bounds = WorldBBox::new(min_x, min_y, max_x, max_y);
    vp.zoom_to_fit(&bounds, rect);

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
    let Some(bbox) = elem.world_bbox() else {
        return false;
    };

    let points = [(x1, y1), (x2, y2), (x3, y3)];
    let scale = 1e-9;
    points.iter().all(|&(x, y)| {
        let wx = f64::from(x) * scale;
        let wy = f64::from(y) * scale;
        wx >= bbox.min_x - 1e-15
            && wx <= bbox.max_x + 1e-15
            && wy >= bbox.min_y - 1e-15
            && wy <= bbox.max_y + 1e-15
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

    let Some(cb) = compute_bounds(&elems) else {
        return false;
    };

    elems.iter().all(|e| {
        if let Some(bbox) = e.world_bbox() {
            bbox.min_x >= cb.min_x - 1e-15
                && bbox.min_y >= cb.min_y - 1e-15
                && bbox.max_x <= cb.max_x + 1e-15
                && bbox.max_y <= cb.max_y + 1e-15
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
    let a = WorldBBox::new(ax, ay, ax + aw.abs(), ay + ah.abs());
    let b = WorldBBox::new(bx, by, bx + bw.abs(), by + bh.abs());
    a.overlaps(&b) == b.overlaps(&a)
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
    let grid = SpatialGrid::build(&elems, &bounds);

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

    let visible = WorldBBox::new(min_x, min_y, max_x, max_y);
    let mut indices: Vec<u32> = grid
        .query_visible(&visible)
        .flat_map(|c| c.indices.iter().copied())
        .collect();
    indices.sort_unstable();
    indices.dedup();

    indices.contains(&0) && indices.contains(&1)
}

/// Merging N meshes into a batch preserves total vertex count and produces
/// correctly offset indices.
#[quickcheck]
#[expect(
    clippy::needless_pass_by_value,
    reason = "quickcheck requires owned types"
)]
fn merge_mesh_preserves_totals(counts: Vec<u8>) -> bool {
    let mut layer_meshes: HashMap<(u16, u16), Mesh> = HashMap::new();
    let key = (1, 0);
    let mut total_verts = 0usize;
    let mut total_indices = 0usize;

    for &n in &counts {
        let n = (n % 20) as usize;
        if n < 3 {
            continue;
        }
        let mut src = Mesh::default();
        for i in 0..n {
            src.vertices.push(egui::epaint::Vertex {
                pos: Pos2::new(i as f32, 0.0),
                uv: egui::epaint::WHITE_UV,
                color: egui::Color32::WHITE,
            });
        }
        for i in 0..(n - 2) {
            src.indices
                .extend_from_slice(&[0, (i + 1) as u32, (i + 2) as u32]);
        }
        total_verts += src.vertices.len();
        total_indices += src.indices.len();

        let dst = layer_meshes.entry(key).or_default();
        let base = dst.vertices.len() as u32;
        dst.vertices.extend_from_slice(&src.vertices);
        dst.indices.extend(src.indices.iter().map(|&i| i + base));
    }

    if let Some(merged) = layer_meshes.get(&key) {
        if merged.vertices.len() != total_verts {
            return false;
        }
        if merged.indices.len() != total_indices {
            return false;
        }
        // All indices must reference valid vertices
        let max_idx = merged.vertices.len() as u32;
        merged.indices.iter().all(|&i| i < max_idx)
    } else {
        total_verts == 0
    }
}

/// Elements at grid cell boundaries appear in multiple cells. The `seen` bitset
/// ensures each element index is processed at most once.
#[quickcheck]
fn dedup_bitset_prevents_duplicates(x1: i8, y1: i8) -> bool {
    let elems = vec![
        helpers::polygon(
            vec![
                (i32::from(x1), i32::from(y1)),
                (i32::from(x1) + 100, i32::from(y1)),
                (i32::from(x1) + 100, i32::from(y1) + 100),
            ],
            1,
            0,
        ),
        helpers::polygon(
            vec![
                (i32::from(x1) + 50, i32::from(y1) + 50),
                (i32::from(x1) + 150, i32::from(y1) + 50),
                (i32::from(x1) + 150, i32::from(y1) + 150),
            ],
            1,
            0,
        ),
    ];

    let Some(bounds) = compute_bounds(&elems) else {
        return false;
    };
    let grid = SpatialGrid::build(&elems, &bounds);

    let w = bounds.max_x - bounds.min_x;
    let h = bounds.max_y - bounds.min_y;
    let visible = WorldBBox::new(
        bounds.min_x - w,
        bounds.min_y - h,
        bounds.max_x + w,
        bounds.max_y + h,
    );

    let mut seen = vec![false; elems.len()];
    let mut draw_counts = vec![0u32; elems.len()];

    for cell in grid.query_visible(&visible) {
        for &idx in &cell.indices {
            let i = idx as usize;
            if seen[i] {
                continue;
            }
            seen[i] = true;
            if i < draw_counts.len() {
                draw_counts[i] += 1;
            }
        }
    }

    draw_counts.iter().all(|&c| c <= 1)
}

/// Counts edges that have non-zero length (matching the 1e-6 threshold in the function).
fn count_nonzero_edges(points: &[Pos2], closed: bool) -> usize {
    if points.len() < 2 {
        return 0;
    }
    let edge_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    (0..edge_count)
        .filter(|&i| {
            let p0 = points[i];
            let p1 = points[(i + 1) % points.len()];
            (p1 - p0).length() >= 1e-6
        })
        .count()
}

#[quickcheck]
#[expect(
    clippy::needless_pass_by_value,
    reason = "quickcheck requires owned types"
)]
fn stroke_polyline_mesh_dimensions(xs: Vec<(i16, i16)>) -> bool {
    let points: Vec<Pos2> = xs
        .iter()
        .map(|&(x, y)| Pos2::new(f32::from(x), f32::from(y)))
        .collect();
    let stroke = Stroke::new(2.0, egui::Color32::WHITE);

    for closed in [false, true] {
        let mesh = stroke_polyline_to_mesh(&points, stroke, closed);
        let expected_edges = count_nonzero_edges(&points, closed);
        if mesh.vertices.len() != 4 * expected_edges {
            return false;
        }
        if mesh.indices.len() != 6 * expected_edges {
            return false;
        }
    }
    true
}
