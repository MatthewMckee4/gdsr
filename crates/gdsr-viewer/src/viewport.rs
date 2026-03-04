use egui::{Color32, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke};
use gdsr::Element;

use crate::colors::LayerColorMap;

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

    /// Adjusts center and zoom to fit the given bounding box in the viewport rect.
    pub fn zoom_to_fit(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, rect: Rect) {
        self.center_x = f64::midpoint(min_x, max_x);
        self.center_y = f64::midpoint(min_y, max_y);

        let world_w = max_x - min_x;
        let world_h = max_y - min_y;

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
    hidden_layers: &std::collections::HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
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
            viewport.zoom *= factor;
            viewport.zoom = viewport.zoom.clamp(1e-3, 1e15);
            // Adjust center so the world point under the cursor stays fixed
            let new_screen = viewport.world_to_screen(wx, wy, rect);
            viewport.center_x += f64::from(hover_pos.x - new_screen.x) / viewport.zoom;
            viewport.center_y -= f64::from(hover_pos.y - new_screen.y) / viewport.zoom;
        }
    }

    // Draw elements
    for element in elements {
        match element {
            Element::Polygon(polygon) => {
                let layer = polygon.layer();
                let dt = polygon.data_type();
                if hidden_layers.contains(&(layer, dt)) {
                    continue;
                }
                draw_polygon(
                    &painter,
                    viewport,
                    rect,
                    polygon,
                    layer_colors.get(layer, dt),
                );
            }
            Element::Path(path) => {
                let layer = path.layer();
                let dt = path.data_type();
                if hidden_layers.contains(&(layer, dt)) {
                    continue;
                }
                draw_path(&painter, viewport, rect, path, layer_colors.get(layer, dt));
            }
            Element::Text(text) => {
                let layer = text.layer();
                if hidden_layers.contains(&(layer, 0)) {
                    continue;
                }
                draw_text(&painter, viewport, rect, text, layer_colors.get(layer, 0));
            }
            Element::Reference(_) => {}
        }
    }

    // Return mouse world position
    response
        .hover_pos()
        .map(|pos| viewport.screen_to_world(pos.x, pos.y, rect))
}

fn draw_polygon(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    polygon: &gdsr::Polygon,
    color: Color32,
) {
    let points = polygon.points();
    if points.len() < 3 {
        return;
    }

    // Convert to screen coordinates
    let screen_pts: Vec<Pos2> = points
        .iter()
        .map(|p| viewport.world_to_screen(p.x().absolute_value(), p.y().absolute_value(), rect))
        .collect();

    // Quick visibility check: skip if bounding box is completely outside the viewport
    let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
    let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);
    for pt in &screen_pts {
        min_x = min_x.min(pt.x);
        min_y = min_y.min(pt.y);
        max_x = max_x.max(pt.x);
        max_y = max_y.max(pt.y);
    }
    if max_x < rect.min.x || min_x > rect.max.x || max_y < rect.min.y || min_y > rect.max.y {
        return;
    }

    // Triangulate with earcutr for correct rendering of concave polygons
    // Remove the closing point if present (earcutr expects open polygons)
    let open_pts = if screen_pts.len() >= 2 && screen_pts.first() == screen_pts.last() {
        &screen_pts[..screen_pts.len() - 1]
    } else {
        &screen_pts
    };

    if open_pts.len() < 3 {
        return;
    }

    let coords: Vec<f64> = open_pts
        .iter()
        .flat_map(|p| [f64::from(p.x), f64::from(p.y)])
        .collect();

    if let Ok(indices) = earcutr::earcut(&coords, &[], 2) {
        let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);
        let mut mesh = Mesh::default();
        for pt in open_pts {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *pt,
                uv: egui::epaint::WHITE_UV,
                color: fill,
            });
        }
        for idx in indices {
            mesh.indices.push(idx as u32);
        }
        painter.add(Shape::mesh(mesh));
    }

    // Outline stroke
    let stroke = Stroke::new(1.0, color);
    for i in 0..open_pts.len() {
        let next = (i + 1) % open_pts.len();
        painter.line_segment([open_pts[i], open_pts[next]], stroke);
    }
}

fn draw_path(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    path: &gdsr::Path,
    color: Color32,
) {
    let points = path.points();
    if points.len() < 2 {
        return;
    }

    let screen_pts: Vec<Pos2> = points
        .iter()
        .map(|p| viewport.world_to_screen(p.x().absolute_value(), p.y().absolute_value(), rect))
        .collect();

    let width_px = path
        .width()
        .map(|w| (w.absolute_value() * viewport.zoom) as f32)
        .unwrap_or(1.0)
        .clamp(1.0, 20.0);

    let stroke = Stroke::new(width_px, color);
    for pair in screen_pts.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
}

fn draw_text(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    text: &gdsr::Text,
    color: Color32,
) {
    let origin = text.origin();
    let screen_pos = viewport.world_to_screen(
        origin.x().absolute_value(),
        origin.y().absolute_value(),
        rect,
    );

    // Skip if outside viewport
    if !rect.contains(screen_pos) {
        return;
    }

    let font_size = (12.0 * viewport.zoom.log10().max(1.0)) as f32;
    if font_size < 4.0 {
        return;
    }
    let font_size = font_size.min(48.0);

    painter.text(
        screen_pos,
        egui::Align2::LEFT_BOTTOM,
        text.text(),
        FontId::monospace(font_size),
        color,
    );
}

#[cfg(test)]
fn test_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
}

/// Computes the bounding box of the given elements in world coordinates.
/// Returns `None` if there are no geometric elements.
pub fn compute_bounds(elements: &[Element]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    for element in elements {
        let points: &[gdsr::Point] = match element {
            Element::Polygon(p) => p.points(),
            Element::Path(p) => p.points(),
            Element::Text(t) => {
                let o = t.origin();
                let x = o.x().absolute_value();
                let y = o.y().absolute_value();
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
                continue;
            }
            Element::Reference(_) => continue,
        };

        for p in points {
            let x = p.x().absolute_value();
            let y = p.y().absolute_value();
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            found = true;
        }
    }

    if found {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
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
        vp.zoom_to_fit(10.0, 20.0, 30.0, 40.0, rect);
        assert!((vp.center_x - 20.0).abs() < EPSILON);
        assert!((vp.center_y - 30.0).abs() < EPSILON);
    }

    #[test]
    fn zoom_to_fit_bounds_are_within_viewport() {
        let mut vp = Viewport::default();
        let rect = test_rect();
        vp.zoom_to_fit(-1.0, -2.0, 3.0, 4.0, rect);

        let min_screen = vp.world_to_screen(-1.0, -2.0, rect);
        let max_screen = vp.world_to_screen(3.0, 4.0, rect);

        assert!(min_screen.x >= rect.min.x);
        assert!(max_screen.x <= rect.max.x);
        // Y is flipped so max_world_y maps to smaller screen_y
        assert!(max_screen.y >= rect.min.y);
        assert!(min_screen.y <= rect.max.y);
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
        let bounds = compute_bounds(&[Element::Polygon(polygon)]);
        let (min_x, min_y, max_x, max_y) = bounds.expect("should have bounds");
        assert!((min_x - 0.0).abs() < EPSILON);
        assert!((min_y - 0.0).abs() < EPSILON);
        assert!((max_x - 1000.0 * 1e-9).abs() < EPSILON);
        assert!((max_y - 2000.0 * 1e-9).abs() < EPSILON);
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
        let bounds = compute_bounds(&[Element::Path(path)]);
        let (min_x, min_y, max_x, max_y) = bounds.expect("should have bounds");
        assert!((min_x - 100.0 * 1e-9).abs() < EPSILON);
        assert!((min_y - 200.0 * 1e-9).abs() < EPSILON);
        assert!((max_x - 300.0 * 1e-9).abs() < EPSILON);
        assert!((max_y - 400.0 * 1e-9).abs() < EPSILON);
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
        let bounds = compute_bounds(&[Element::Text(text)]);
        let (min_x, min_y, max_x, max_y) = bounds.expect("should have bounds");
        assert!((min_x - 500.0 * 1e-9).abs() < EPSILON);
        assert!((min_y - 600.0 * 1e-9).abs() < EPSILON);
        assert_eq!(min_x, max_x);
        assert_eq!(min_y, max_y);
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
        let bounds = compute_bounds(&[Element::Polygon(polygon), Element::Text(text)]);
        let (min_x, min_y, _, _) = bounds.expect("should have bounds");
        assert!((min_x - 0.0).abs() < EPSILON);
        assert!((min_y - 0.0).abs() < EPSILON);
    }
}
