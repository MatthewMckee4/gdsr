use egui::{Color32, Painter, Rect, Stroke};

use crate::drawable::WorldBBox;
use crate::viewport::Viewport;

/// Grid line color — subtle against the dark background.
const GRID_COLOR: Color32 = Color32::from_rgb(60, 60, 60);

/// Target screen-space distance between grid lines in pixels.
const TARGET_SPACING_PX: f64 = 80.0;

/// 1-2-5 sequence multipliers used for "nice" grid spacing.
const STEPS: [f64; 3] = [1.0, 2.0, 5.0];

/// Computes a "nice" grid spacing in world units so that lines are approximately
/// `TARGET_SPACING_PX` apart on screen given the current zoom level.
pub fn grid_spacing(zoom: f64) -> f64 {
    let raw = TARGET_SPACING_PX / zoom;
    let exponent = raw.log10().floor();
    let mag = 10_f64.powf(exponent);

    for &step in &STEPS {
        if step * mag >= raw {
            return step * mag;
        }
    }
    10.0 * mag
}

/// Returns the first grid line position at or before `min` aligned to `spacing`.
pub fn first_line(min: f64, spacing: f64) -> f64 {
    (min / spacing).floor() * spacing
}

/// Draws a background grid on the painter.
pub fn draw_grid(painter: &Painter, viewport: &Viewport, rect: Rect) {
    let visible = viewport.visible_world_rect(rect);
    let spacing = grid_spacing(viewport.zoom);
    let stroke = Stroke::new(1.0, GRID_COLOR);

    draw_grid_lines(painter, viewport, rect, &visible, spacing, stroke);
}

fn draw_grid_lines(
    painter: &Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &WorldBBox,
    spacing: f64,
    stroke: Stroke,
) {
    let mut x = first_line(visible.min_x, spacing);
    while x <= visible.max_x {
        let top = viewport.world_to_screen(x, visible.max_y, rect);
        let bottom = viewport.world_to_screen(x, visible.min_y, rect);
        painter.line_segment([top, bottom], stroke);
        x += spacing;
    }

    let mut y = first_line(visible.min_y, spacing);
    while y <= visible.max_y {
        let left = viewport.world_to_screen(visible.min_x, y, rect);
        let right = viewport.world_to_screen(visible.max_x, y, rect);
        painter.line_segment([left, right], stroke);
        y += spacing;
    }
}

/// Draws the origin axes with slightly brighter lines.
pub fn draw_origin_axes(painter: &Painter, viewport: &Viewport, rect: Rect) {
    let visible = viewport.visible_world_rect(rect);
    let stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 100));

    if visible.min_x <= 0.0 && visible.max_x >= 0.0 {
        let top = viewport.world_to_screen(0.0, visible.max_y, rect);
        let bottom = viewport.world_to_screen(0.0, visible.min_y, rect);
        painter.line_segment([top, bottom], stroke);
    }
    if visible.min_y <= 0.0 && visible.max_y >= 0.0 {
        let left = viewport.world_to_screen(visible.min_x, 0.0, rect);
        let right = viewport.world_to_screen(visible.max_x, 0.0, rect);
        painter.line_segment([left, right], stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_spacing_follows_1_2_5_sequence() {
        let cases: &[(f64, f64)] = &[
            (1.0, 100.0),
            (10.0, 10.0),
            (100.0, 1.0),
            (1000.0, 0.1),
            (0.1, 1000.0),
        ];
        for &(zoom, expected) in cases {
            let spacing = grid_spacing(zoom);
            assert!(
                (spacing - expected).abs() < expected * 0.01,
                "zoom={zoom}, spacing={spacing}, expected={expected}"
            );
        }
    }

    #[test]
    fn grid_spacing_is_always_positive() {
        for exp in -5..15 {
            let zoom = 10_f64.powi(exp);
            let spacing = grid_spacing(zoom);
            assert!(spacing > 0.0, "zoom={zoom}, spacing={spacing}");
        }
    }

    #[test]
    fn grid_spacing_screen_distance_in_reasonable_range() {
        for exp in -3..10 {
            let zoom = 10_f64.powi(exp);
            let spacing = grid_spacing(zoom);
            let screen_px = spacing * zoom;
            assert!(
                (40.0..=200.0).contains(&screen_px),
                "zoom={zoom}, spacing={spacing}, screen_px={screen_px}"
            );
        }
    }

    #[test]
    fn first_line_aligns_to_spacing() {
        assert!((first_line(0.0, 10.0) - 0.0).abs() < 1e-10);
        assert!((first_line(5.0, 10.0) - 0.0).abs() < 1e-10);
        assert!((first_line(15.0, 10.0) - 10.0).abs() < 1e-10);
        assert!((first_line(-5.0, 10.0) - (-10.0)).abs() < 1e-10);
        assert!((first_line(-15.0, 10.0) - (-20.0)).abs() < 1e-10);
    }

    #[test]
    fn grid_spacing_at_min_zoom() {
        let spacing = grid_spacing(1e-3);
        assert!(spacing > 0.0);
        assert!(spacing.is_finite());
        let screen_px = spacing * 1e-3;
        assert!((40.0..=200.0).contains(&screen_px));
    }

    #[test]
    fn grid_spacing_at_max_zoom() {
        let spacing = grid_spacing(1e15);
        assert!(spacing > 0.0);
        assert!(spacing.is_finite());
        let screen_px = spacing * 1e15;
        assert!((40.0..=200.0).contains(&screen_px));
    }

    #[test]
    fn grid_spacing_very_small_zoom() {
        let spacing = grid_spacing(1e-10);
        assert!(spacing > 0.0);
        assert!(spacing.is_finite());
    }

    #[test]
    fn grid_spacing_very_large_zoom() {
        let spacing = grid_spacing(1e12);
        assert!(spacing > 0.0);
        assert!(spacing.is_finite());
    }

    #[test]
    fn first_line_large_negative() {
        let result = first_line(-1e15, 100.0);
        assert!(result.is_finite());
        assert!(result <= -1e15);
    }

    #[test]
    fn first_line_large_positive() {
        let result = first_line(1e15, 100.0);
        assert!(result.is_finite());
        assert!(result <= 1e15);
    }

    #[test]
    fn first_line_tiny_spacing() {
        let result = first_line(1.0, 1e-12);
        assert!(result.is_finite());
    }

    #[test]
    fn first_line_exact_multiple() {
        assert!((first_line(20.0, 10.0) - 20.0).abs() < 1e-10);
        assert!((first_line(-20.0, 10.0) - (-20.0)).abs() < 1e-10);
    }
}
