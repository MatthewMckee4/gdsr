pub mod bounds;

pub use bounds::compute_bounds;

use std::collections::HashMap;

use egui::{Color32, Pos2, Rect, Sense};
use gdsr::{Element, Library, Point};

use crate::drawable::{WorldBBox, draw_highlight};
use crate::grid;
use crate::ruler::RulerState;
use crate::state::{GridSpacing, LayerState};

/// Camera state for the 2D viewport: center position in world coordinates and zoom level.
pub struct Viewport {
    pub center_x: f64,
    pub center_y: f64,
    /// Pixels per world-unit.
    pub zoom: f64,
}

pub struct ViewportInteraction {
    pub rect: Rect,
    pub mouse_world: Option<(f64, f64)>,
    pub clicked: bool,
    pub double_clicked: bool,
    pub selected_element_drag_delta: Option<(f64, f64)>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// Converts a world-space point to screen-space. Y is flipped because GDS uses
    /// Y-up while screen coordinates are Y-down.
    pub fn world_to_screen(&self, wx: f64, wy: f64, rect: Rect) -> Pos2 {
        let cx = f64::from(rect.center().x);
        let cy = f64::from(rect.center().y);
        let sx = cx + (wx - self.center_x) * self.zoom;
        let sy = cy - (wy - self.center_y) * self.zoom;
        Pos2::new(sx as f32, sy as f32)
    }

    /// Converts a screen-space point back to world-space.
    pub fn screen_to_world(&self, sx: f32, sy: f32, rect: Rect) -> (f64, f64) {
        let cx = f64::from(rect.center().x);
        let cy = f64::from(rect.center().y);
        let wx = (f64::from(sx) - cx) / self.zoom + self.center_x;
        let wy = -(f64::from(sy) - cy) / self.zoom + self.center_y;
        (wx, wy)
    }

    /// Returns the visible world-space rectangle.
    pub fn visible_world_rect(&self, rect: Rect) -> WorldBBox {
        let (min_x, max_y) = self.screen_to_world(rect.min.x, rect.min.y, rect);
        let (max_x, min_y) = self.screen_to_world(rect.max.x, rect.max.y, rect);
        WorldBBox::new(min_x, min_y, max_x, max_y)
    }

    /// Pans the viewport by the given world-space deltas.
    pub fn pan(&mut self, dx_world: f64, dy_world: f64) {
        self.center_x += dx_world;
        self.center_y += dy_world;
    }

    /// Zooms by `factor` anchored at the center of the viewport.
    pub fn zoom_at_center(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).clamp(1e-3, 1e15);
    }

    /// Adjusts center and zoom to fit the given bounding box in the viewport rect.
    pub fn zoom_to_fit(&mut self, bounds: &WorldBBox, rect: Rect) {
        self.center_x = f64::midpoint(bounds.min_x, bounds.max_x);
        self.center_y = f64::midpoint(bounds.min_y, bounds.max_y);

        let world_w = bounds.max_x - bounds.min_x;
        let world_h = bounds.max_y - bounds.min_y;

        if world_w > 0.0 && world_h > 0.0 {
            let zoom_x = f64::from(rect.width()) / world_w;
            let zoom_y = f64::from(rect.height()) / world_h;
            self.zoom = zoom_x.min(zoom_y) * 0.9; // 10% margin
        }
    }
}

impl Viewport {
    /// Draws viewport interaction and egui overlays over the Bevy scene.
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        elements: &[Element],
        layer_state: &mut LayerState,
        library: Option<&Library>,
        tessellation_cache: &mut HashMap<u32, Vec<usize>>,
        ruler: &mut RulerState,
        show_grid: bool,
        grid_spacing: GridSpacing,
        hovered_element: Option<usize>,
        selected_element: Option<usize>,
        drawing_preview_points: Option<&[Point]>,
    ) -> ViewportInteraction {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;

        if show_grid {
            grid::draw_grid(&painter, self, rect, grid_spacing);
            grid::draw_origin_axes(&painter, self, rect);
        }

        let ruler_was_active = ruler.active;
        if ruler_was_active
            && response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let (wx, wy) = self.screen_to_world(pos.x, pos.y, rect);
            ruler.handle_click(wx, wy);
        }
        let clicked = response.clicked() && !ruler_was_active;
        let double_clicked = response.double_clicked() && !ruler_was_active;

        let move_selected_element = selected_element.is_some()
            && ui.input(|input| input.modifiers.shift)
            && response.dragged();
        let selected_element_drag_delta = if move_selected_element {
            let delta = response.drag_delta();
            Some((
                f64::from(delta.x) / self.zoom,
                -f64::from(delta.y) / self.zoom,
            ))
        } else {
            None
        };

        if response.dragged() && !move_selected_element {
            let delta = response.drag_delta();
            self.center_x -= f64::from(delta.x) / self.zoom;
            self.center_y += f64::from(delta.y) / self.zoom;
        }

        if let Some(hover_pos) = response.hover_pos() {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let (wx, wy) = self.screen_to_world(hover_pos.x, hover_pos.y, rect);
                let factor = 1.0 + f64::from(scroll) * 0.002;
                let new_zoom = (self.zoom * factor).clamp(1e-3, 1e15);
                let center = rect.center();
                self.center_x = wx - f64::from(hover_pos.x - center.x) / new_zoom;
                self.center_y = wy + f64::from(hover_pos.y - center.y) / new_zoom;
                self.zoom = new_zoom;
            }
        }

        let pan_step_x = f64::from(rect.width()) * 0.1 / self.zoom;
        let pan_step_y = f64::from(rect.height()) * 0.1 / self.zoom;
        ui.input(|input| {
            if input.key_pressed(egui::Key::ArrowLeft) {
                self.pan(-pan_step_x, 0.0);
            }
            if input.key_pressed(egui::Key::ArrowRight) {
                self.pan(pan_step_x, 0.0);
            }
            if input.key_pressed(egui::Key::ArrowUp) {
                self.pan(0.0, pan_step_y);
            }
            if input.key_pressed(egui::Key::ArrowDown) {
                self.pan(0.0, -pan_step_y);
            }
            if input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals) {
                self.zoom_at_center(1.2);
            }
            if input.key_pressed(egui::Key::Minus) {
                self.zoom_at_center(1.0 / 1.2);
            }
        });

        draw_element_highlight(
            selected_element,
            elements,
            self,
            &painter,
            rect,
            layer_state,
            library,
            tessellation_cache,
        );
        if hovered_element != selected_element {
            draw_element_highlight(
                hovered_element,
                elements,
                self,
                &painter,
                rect,
                layer_state,
                library,
                tessellation_cache,
            );
        }

        let mouse_world = response
            .hover_pos()
            .map(|pos| self.screen_to_world(pos.x, pos.y, rect));
        draw_drawing_preview(drawing_preview_points, self, &painter, rect);
        ruler.draw(&painter, self, rect, mouse_world);

        ViewportInteraction {
            rect,
            mouse_world,
            clicked,
            double_clicked,
            selected_element_drag_delta,
        }
    }
}

fn draw_drawing_preview(
    points: Option<&[Point]>,
    viewport: &Viewport,
    painter: &egui::Painter,
    rect: Rect,
) {
    let Some(points) = points else {
        return;
    };
    if points.is_empty() {
        return;
    }

    let color = Color32::from_rgb(120, 220, 255);
    let stroke = egui::Stroke::new(1.5_f32, color);
    let screen_points: Vec<Pos2> = points
        .iter()
        .map(|point| {
            viewport.world_to_screen(point.x().absolute_value(), point.y().absolute_value(), rect)
        })
        .collect();

    for pair in screen_points.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
    for point in screen_points {
        painter.circle_filled(point, 3.5, color);
    }
}

fn draw_element_highlight(
    idx: Option<usize>,
    elements: &[Element],
    viewport: &Viewport,
    painter: &egui::Painter,
    rect: Rect,
    layer_state: &mut LayerState,
    library: Option<&Library>,
    tessellation_cache: &mut HashMap<u32, Vec<usize>>,
) {
    if let Some(el) = idx.and_then(|idx| elements.get(idx)) {
        draw_highlight(
            el,
            viewport,
            painter,
            rect,
            layer_state,
            library,
            tessellation_cache,
        );
    }
}

#[cfg(test)]
fn test_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn world_screen_roundtrip_at_origin() {
        let vp = Viewport::default();
        let rect = test_rect();
        let (wx, wy) = (0.0, 0.0);
        let screen = vp.world_to_screen(wx, wy, rect);
        let (rx, ry) = vp.screen_to_world(screen.x, screen.y, rect);
        assert!((rx - wx).abs() < EPSILON);
        assert!((ry - wy).abs() < EPSILON);
    }

    #[test]
    fn world_screen_roundtrip_off_center() {
        let vp = Viewport {
            center_x: 100.0,
            center_y: -50.0,
            zoom: 1000.0,
        };
        let rect = test_rect();
        let (wx, wy) = (123.456, -78.9);
        let screen = vp.world_to_screen(wx, wy, rect);
        let (rx, ry) = vp.screen_to_world(screen.x, screen.y, rect);
        assert!((rx - wx).abs() < EPSILON);
        assert!((ry - wy).abs() < EPSILON);
    }

    #[test]
    fn center_maps_to_screen_center() {
        let vp = Viewport {
            center_x: 42.0,
            center_y: 17.0,
            zoom: 500.0,
        };
        let rect = test_rect();
        let screen = vp.world_to_screen(42.0, 17.0, rect);
        assert!((f64::from(screen.x) - f64::from(rect.center().x)).abs() < EPSILON);
        assert!((f64::from(screen.y) - f64::from(rect.center().y)).abs() < EPSILON);
    }

    /// Y-axis is flipped: increasing world Y should decrease screen Y.
    #[test]
    fn y_axis_is_flipped() {
        let vp = Viewport::default();
        let rect = test_rect();
        let low = vp.world_to_screen(0.0, 0.0, rect);
        let high = vp.world_to_screen(0.0, 1.0, rect);
        assert!(high.y < low.y);
    }

    #[test]
    fn zoom_to_fit_centers_on_bounds() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(10.0, 20.0, 30.0, 40.0);
        vp.zoom_to_fit(&bounds, rect);
        assert!((vp.center_x - 20.0).abs() < EPSILON);
        assert!((vp.center_y - 30.0).abs() < EPSILON);
    }

    #[test]
    fn zoom_to_fit_bounds_are_within_viewport() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(-1.0, -2.0, 3.0, 4.0);
        vp.zoom_to_fit(&bounds, rect);

        let min_screen = vp.world_to_screen(-1.0, -2.0, rect);
        let max_screen = vp.world_to_screen(3.0, 4.0, rect);

        assert!(min_screen.x >= rect.min.x);
        assert!(max_screen.x <= rect.max.x);
        assert!(max_screen.y >= rect.min.y);
        assert!(min_screen.y <= rect.max.y);
    }

    #[test]
    fn visible_world_rect_matches_screen_corners() {
        let vp = Viewport {
            center_x: 5.0,
            center_y: 10.0,
            zoom: 100.0,
        };
        let rect = test_rect();
        let vis = vp.visible_world_rect(rect);

        let (wx_tl, wy_tl) = vp.screen_to_world(rect.min.x, rect.min.y, rect);
        assert!((vis.min_x - wx_tl).abs() < EPSILON);
        assert!((vis.max_y - wy_tl).abs() < EPSILON);

        let (wx_br, wy_br) = vp.screen_to_world(rect.max.x, rect.max.y, rect);
        assert!((vis.max_x - wx_br).abs() < EPSILON);
        assert!((vis.min_y - wy_br).abs() < EPSILON);
    }

    /// Simulates the zoom logic from `draw_viewport`: zoom by `factor` anchored at `cursor`.
    fn apply_zoom(vp: &mut Viewport, rect: Rect, cursor: Pos2, factor: f64) {
        let (wx, wy) = vp.screen_to_world(cursor.x, cursor.y, rect);
        let new_zoom = (vp.zoom * factor).clamp(1e-3, 1e15);
        let cx = f64::from(rect.center().x);
        let cy = f64::from(rect.center().y);
        vp.center_x = wx - (f64::from(cursor.x) - cx) / new_zoom;
        vp.center_y = wy + (f64::from(cursor.y) - cy) / new_zoom;
        vp.zoom = new_zoom;
    }

    #[test]
    fn zoom_preserves_world_point_under_cursor() {
        let rect = test_rect();
        let cursor = Pos2::new(200.0, 150.0);
        let mut vp = Viewport {
            center_x: 50.0,
            center_y: 30.0,
            zoom: 200.0,
        };

        let (wx, wy) = vp.screen_to_world(cursor.x, cursor.y, rect);
        apply_zoom(&mut vp, rect, cursor, 1.5);
        let after = vp.world_to_screen(wx, wy, rect);

        assert!((f64::from(after.x - cursor.x)).abs() < 0.01);
        assert!((f64::from(after.y - cursor.y)).abs() < 0.01);
    }

    #[test]
    fn pan_shifts_center() {
        let mut vp = Viewport::default();
        vp.pan(10.0, -5.0);
        assert!((vp.center_x - 10.0).abs() < EPSILON);
        assert!((vp.center_y - (-5.0)).abs() < EPSILON);
    }

    #[test]
    fn zoom_at_center_scales_zoom() {
        let mut vp = Viewport {
            center_x: 5.0,
            center_y: 10.0,
            zoom: 100.0,
        };
        let (orig_cx, orig_cy) = (vp.center_x, vp.center_y);
        vp.zoom_at_center(2.0);
        assert!((vp.zoom - 200.0).abs() < EPSILON);
        assert!((vp.center_x - orig_cx).abs() < EPSILON);
        assert!((vp.center_y - orig_cy).abs() < EPSILON);
    }

    #[test]
    fn zoom_at_center_clamps() {
        let mut vp = Viewport {
            zoom: 1e-3,
            ..Default::default()
        };
        vp.zoom_at_center(0.1);
        assert!(vp.zoom >= 1e-3);

        let mut vp = Viewport {
            zoom: 1e15,
            ..Default::default()
        };
        vp.zoom_at_center(10.0);
        assert!(vp.zoom <= 1e15);
    }

    #[test]
    fn zoom_at_center_in_then_out_returns_to_original() {
        let mut vp = Viewport {
            zoom: 100.0,
            ..Default::default()
        };
        vp.zoom_at_center(1.2);
        vp.zoom_at_center(1.0 / 1.2);
        assert!((vp.zoom - 100.0).abs() < 1e-10);
    }

    #[test]
    fn world_to_screen_extreme_zoom_min() {
        let vp = Viewport {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1e-3,
        };
        let rect = test_rect();
        let screen = vp.world_to_screen(1e6, 1e6, rect);
        assert!(screen.x.is_finite());
        assert!(screen.y.is_finite());
    }

    #[test]
    fn world_to_screen_extreme_zoom_max() {
        let vp = Viewport {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1e15,
        };
        let rect = test_rect();
        let screen = vp.world_to_screen(1e-15, 1e-15, rect);
        assert!(screen.x.is_finite());
        assert!(screen.y.is_finite());
    }

    #[test]
    fn world_to_screen_large_world_coordinates() {
        let vp = Viewport {
            center_x: 2.0,
            center_y: 2.0,
            zoom: 1.0,
        };
        let rect = test_rect();
        let screen = vp.world_to_screen(2.147e9 * 1e-9, 2.147e9 * 1e-9, rect);
        assert!(screen.x.is_finite());
        assert!(screen.y.is_finite());
    }

    #[test]
    fn zoom_to_fit_zero_area_bounds() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(5.0, 5.0, 5.0, 5.0);
        vp.zoom_to_fit(&bounds, rect);
        assert!((vp.center_x - 5.0).abs() < EPSILON);
        assert!((vp.center_y - 5.0).abs() < EPSILON);
        assert!(
            (vp.zoom - 1.0).abs() < EPSILON,
            "zoom should stay at default for zero-area bounds"
        );
    }

    #[test]
    fn zoom_to_fit_zero_width() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(5.0, 0.0, 5.0, 10.0);
        vp.zoom_to_fit(&bounds, rect);
        assert!((vp.center_x - 5.0).abs() < EPSILON);
        assert!(
            (vp.zoom - 1.0).abs() < EPSILON,
            "zoom should stay at default for zero-width bounds"
        );
    }

    #[test]
    fn zoom_to_fit_huge_bounds() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(-1e6, -1e6, 1e6, 1e6);
        vp.zoom_to_fit(&bounds, rect);
        assert!(vp.zoom > 0.0);
        assert!(vp.zoom.is_finite());
    }

    #[test]
    fn zoom_to_fit_tiny_bounds() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        let bounds = WorldBBox::new(0.0, 0.0, 1e-12, 1e-12);
        vp.zoom_to_fit(&bounds, rect);
        assert!(vp.zoom > 0.0);
        assert!(vp.zoom.is_finite());
    }

    #[test]
    fn visible_world_rect_extreme_zoom_min() {
        let vp = Viewport {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1e-3,
        };
        let rect = test_rect();
        let vis = vp.visible_world_rect(rect);
        assert!(vis.min_x.is_finite());
        assert!(vis.max_x.is_finite());
        assert!(vis.min_y.is_finite());
        assert!(vis.max_y.is_finite());
        assert!(vis.max_x > vis.min_x);
        assert!(vis.max_y > vis.min_y);
    }

    #[test]
    fn visible_world_rect_extreme_zoom_max() {
        let vp = Viewport {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1e15,
        };
        let rect = test_rect();
        let vis = vp.visible_world_rect(rect);
        assert!(vis.min_x.is_finite());
        assert!(vis.max_x.is_finite());
        assert!(vis.max_x > vis.min_x);
    }

    #[test]
    fn pan_extreme_values() {
        let mut vp = Viewport::default();
        vp.pan(1e15, -1e15);
        assert!(vp.center_x.is_finite());
        assert!(vp.center_y.is_finite());
    }

    #[test]
    fn repeated_zoom_stays_finite() {
        let mut vp = Viewport {
            zoom: 100.0,
            ..Default::default()
        };
        for _ in 0..1000 {
            vp.zoom_at_center(1.2);
        }
        assert!(vp.zoom.is_finite());
        assert!(vp.zoom <= 1e15);

        for _ in 0..2000 {
            vp.zoom_at_center(1.0 / 1.2);
        }
        assert!(vp.zoom.is_finite());
        assert!(vp.zoom >= 1e-3);
    }

    #[test]
    fn zoom_in_then_out_returns_to_original() {
        let rect = test_rect();
        let cursor = Pos2::new(600.0, 400.0);
        let mut vp = Viewport {
            center_x: 10.0,
            center_y: 20.0,
            zoom: 500.0,
        };
        let (orig_cx, orig_cy, orig_z) = (vp.center_x, vp.center_y, vp.zoom);

        apply_zoom(&mut vp, rect, cursor, 2.0);
        apply_zoom(&mut vp, rect, cursor, 0.5);

        assert!((vp.zoom - orig_z).abs() < EPSILON);
        assert!((vp.center_x - orig_cx).abs() < EPSILON);
        assert!((vp.center_y - orig_cy).abs() < EPSILON);
    }
}
