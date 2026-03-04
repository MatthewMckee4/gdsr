use std::collections::HashSet;

use egui::{Color32, Pos2, Rect, Sense};
use gdsr::Element;

use crate::colors::LayerColorMap;
use crate::drawable::{Drawable, WorldBBox};
use crate::spatial::SpatialGrid;

/// Camera state for the 2D viewport: center position in world coordinates and zoom level.
pub struct Viewport {
    pub center_x: f64,
    pub center_y: f64,
    /// Pixels per world-unit.
    pub zoom: f64,
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
    pub fn world_to_screen(&self, wx: f64, wy: f64, rect: Rect) -> Pos2 {
        let cx = f64::from(rect.center().x);
        let cy = f64::from(rect.center().y);
        let sx = cx + (wx - self.center_x) * self.zoom;
        // Negate Y: GDS is Y-up, screen is Y-down
        let sy = cy - (wy - self.center_y) * self.zoom;
        Pos2::new(sx as f32, sy as f32)
    }

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

/// Draws the viewport and handles pan/zoom interaction.
///
/// Returns the mouse position in world coordinates if the pointer is inside the viewport.
pub fn draw_viewport(
    ui: &mut egui::Ui,
    viewport: &mut Viewport,
    elements: &[Element],
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
    spatial_grid: Option<&SpatialGrid>,
) -> Option<(f64, f64)> {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let rect = response.rect;

    // Dark background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 30));

    // Pan via drag
    if response.dragged() {
        let delta = response.drag_delta();
        viewport.center_x -= f64::from(delta.x) / viewport.zoom;
        viewport.center_y += f64::from(delta.y) / viewport.zoom;
    }

    // Zoom toward cursor on scroll
    if let Some(hover_pos) = response.hover_pos() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let (wx, wy) = viewport.screen_to_world(hover_pos.x, hover_pos.y, rect);
            let factor = 1.0 + f64::from(scroll) * 0.002;
            let new_zoom = (viewport.zoom * factor).clamp(1e-3, 1e15);
            // Anchor: the world point under the cursor must stay at hover_pos
            // screen_x = cx + (wx - center_x) * zoom  →  center_x = wx - (screen_x - cx) / zoom
            let cx = f64::from(rect.center().x);
            let cy = f64::from(rect.center().y);
            let sx = f64::from(hover_pos.x);
            let sy = f64::from(hover_pos.y);
            viewport.center_x = wx - (sx - cx) / new_zoom;
            viewport.center_y = wy + (sy - cy) / new_zoom;
            viewport.zoom = new_zoom;
        }
    }

    let visible = viewport.visible_world_rect(rect);

    if let Some(grid) = spatial_grid {
        draw_with_grid(
            &painter,
            viewport,
            rect,
            &visible,
            elements,
            hidden_layers,
            layer_colors,
            grid,
        );
    } else {
        draw_elements_flat(
            &painter,
            viewport,
            rect,
            &visible,
            elements,
            hidden_layers,
            layer_colors,
        );
    }

    // Return mouse world position
    response
        .hover_pos()
        .map(|pos| viewport.screen_to_world(pos.x, pos.y, rect))
}

fn draw_elements_flat(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &WorldBBox,
    elements: &[Element],
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
) {
    for element in elements {
        draw_element(
            painter,
            viewport,
            rect,
            visible,
            element,
            hidden_layers,
            layer_colors,
        );
    }
}

fn draw_element(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &WorldBBox,
    element: &Element,
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
) {
    let key = element.layer_key();
    if hidden_layers.contains(&key) {
        return;
    }
    let color = layer_colors.get(key.0, key.1);
    element.draw(painter, viewport, rect, visible, color);
}

/// Screen-pixel threshold below which a grid cell draws as a single LOAD rectangle.
const CELL_LOAD_THRESHOLD_PX: f32 = 24.0;

fn draw_with_grid(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &WorldBBox,
    elements: &[Element],
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
    grid: &SpatialGrid,
) {
    for cell in grid.query_visible(visible) {
        let s_min = viewport.world_to_screen(cell.bbox.min_x, cell.bbox.min_y, rect);
        let s_max = viewport.world_to_screen(cell.bbox.max_x, cell.bbox.max_y, rect);
        let sw = (s_max.x - s_min.x).abs();
        let sh = (s_min.y - s_max.y).abs(); // Y flipped

        if sw < 1.0 && sh < 1.0 {
            continue;
        }

        if sw < CELL_LOAD_THRESHOLD_PX && sh < CELL_LOAD_THRESHOLD_PX {
            if !hidden_layers.contains(&cell.dominant_layer) {
                let color = layer_colors.get(cell.dominant_layer.0, cell.dominant_layer.1);
                let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);
                let cell_rect = Rect::from_two_pos(s_min, s_max);
                painter.rect_filled(cell_rect, 0.0, fill);
            }
            continue;
        }

        for &idx in &cell.indices {
            if let Some(element) = elements.get(idx as usize) {
                draw_element(
                    painter,
                    viewport,
                    rect,
                    visible,
                    element,
                    hidden_layers,
                    layer_colors,
                );
            }
        }
    }
}

#[cfg(test)]
fn test_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
}

/// Computes the bounding box of the given elements in world coordinates.
/// Returns `None` if there are no geometric elements.
pub fn compute_bounds(elements: &[Element]) -> Option<WorldBBox> {
    let mut result: Option<WorldBBox> = None;

    for element in elements {
        if let Some(bbox) = element.world_bbox() {
            result = Some(match result {
                Some(acc) => acc.merge(&bbox),
                None => bbox,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{HorizontalPresentation, Path, Point, Polygon, Text, Unit, VerticalPresentation};

    const EPSILON: f64 = 1e-6;

    /// `world_to_screen` followed by `screen_to_world` should return the original point.
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

    /// The viewport center should map to the screen center.
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
        vp.zoom_to_fit(&WorldBBox::new(10.0, 20.0, 30.0, 40.0), rect);
        assert!((vp.center_x - 20.0).abs() < EPSILON);
        assert!((vp.center_y - 30.0).abs() < EPSILON);
    }

    #[test]
    fn zoom_to_fit_bounds_are_within_viewport() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        vp.zoom_to_fit(&WorldBBox::new(-1.0, -2.0, 3.0, 4.0), rect);

        let min_screen = vp.world_to_screen(-1.0, -2.0, rect);
        let max_screen = vp.world_to_screen(3.0, 4.0, rect);

        assert!(min_screen.x >= rect.min.x);
        assert!(max_screen.x <= rect.max.x);
        // Y is flipped so max_world_y maps to smaller screen_y
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

        // Top-left screen corner → max world Y (Y flipped)
        let (wx_tl, wy_tl) = vp.screen_to_world(rect.min.x, rect.min.y, rect);
        assert!((vis.min_x - wx_tl).abs() < EPSILON);
        assert!((vis.max_y - wy_tl).abs() < EPSILON);

        // Bottom-right screen corner → min world Y
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

    #[test]
    fn compute_bounds_empty_returns_none() {
        assert!(compute_bounds(&[]).is_none());
    }

    #[test]
    fn compute_bounds_ignores_references() {
        let reference = Element::Reference(gdsr::Reference::default());
        assert!(compute_bounds(&[reference]).is_none());
    }

    #[test]
    fn compute_bounds_polygon() {
        let polygon = Polygon::new(
            vec![
                Point::default_integer(0, 0),
                Point::default_integer(1000, 0),
                Point::default_integer(1000, 2000),
            ],
            1,
            0,
        );
        let bounds = compute_bounds(&[Element::Polygon(polygon)]).expect("should have bounds");
        assert!((bounds.min_x - 0.0).abs() < EPSILON);
        assert!((bounds.min_y - 0.0).abs() < EPSILON);
        assert!((bounds.max_x - 1000.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_y - 2000.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_path() {
        let path = Path::new(
            vec![
                Point::default_integer(100, 200),
                Point::default_integer(300, 400),
            ],
            1,
            0,
            None,
            Some(Unit::default_integer(10)),
        );
        let bounds = compute_bounds(&[Element::Path(path)]).expect("should have bounds");
        assert!((bounds.min_x - 100.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.min_y - 200.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_x - 300.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_y - 400.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_text() {
        let text = Text::new(
            "hello",
            Point::default_integer(500, 600),
            1,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        );
        let bounds = compute_bounds(&[Element::Text(text)]).expect("should have bounds");
        assert!((bounds.min_x - 500.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.min_y - 600.0 * 1e-9).abs() < EPSILON);
        assert_eq!(bounds.min_x, bounds.max_x);
        assert_eq!(bounds.min_y, bounds.max_y);
    }

    #[test]
    fn compute_bounds_mixed_elements() {
        let polygon = Polygon::new(
            vec![
                Point::default_integer(0, 0),
                Point::default_integer(100, 0),
                Point::default_integer(100, 100),
            ],
            1,
            0,
        );
        let text = Text::new(
            "far",
            Point::default_integer(500, 500),
            2,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        );
        let bounds = compute_bounds(&[Element::Polygon(polygon), Element::Text(text)])
            .expect("should have bounds");
        assert!((bounds.min_x - 0.0).abs() < EPSILON);
        assert!((bounds.min_y - 0.0).abs() < EPSILON);
    }
}
