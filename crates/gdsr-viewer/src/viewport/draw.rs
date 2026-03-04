use std::collections::HashSet;

use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke};
use gdsr::Element;

use super::Viewport;
use crate::colors::LayerColorMap;
use crate::spatial::SpatialGrid;
use crate::viewport::bounds::{bbox_overlaps, points_bbox};

/// Fallback drawing path: iterates over every element without spatial indexing.
pub(crate) fn draw_elements_flat(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &[f64; 4],
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

/// Dispatches a single element to the appropriate draw function after layer filtering.
pub(crate) fn draw_element(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &[f64; 4],
    element: &Element,
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
) {
    match element {
        Element::Polygon(polygon) => {
            let layer = polygon.layer();
            let dt = polygon.data_type();
            if hidden_layers.contains(&(layer, dt)) {
                return;
            }
            draw_polygon(
                painter,
                viewport,
                rect,
                visible,
                polygon,
                layer_colors.get(layer, dt),
            );
        }
        Element::Path(path) => {
            let layer = path.layer();
            let dt = path.data_type();
            if hidden_layers.contains(&(layer, dt)) {
                return;
            }
            draw_path(
                painter,
                viewport,
                rect,
                visible,
                path,
                layer_colors.get(layer, dt),
            );
        }
        Element::Text(text) => {
            let layer = text.layer();
            if hidden_layers.contains(&(layer, 0)) {
                return;
            }
            draw_text(painter, viewport, rect, text, layer_colors.get(layer, 0));
        }
        Element::Reference(_) => {}
    }
}

/// Screen-pixel threshold below which a grid cell draws as a single LOAD rectangle
/// instead of rendering individual elements.
const CELL_LOAD_THRESHOLD_PX: f32 = 24.0;

/// Draws elements using the spatial grid for cell-level LOAD. Cells that are too small
/// on screen are drawn as a single colored rectangle using the dominant layer color.
pub(crate) fn draw_with_grid(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &[f64; 4],
    elements: &[Element],
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
    grid: &SpatialGrid,
) {
    for cell in grid.query_visible(visible) {
        let s_min = viewport.world_to_screen(cell.bbox[0], cell.bbox[1], rect);
        let s_max = viewport.world_to_screen(cell.bbox[2], cell.bbox[3], rect);
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

/// Screen-pixel threshold below which polygons render as a filled bounding box instead of
/// full triangulation. Avoids expensive earcut calls for elements that are just a few pixels.
const BBOX_FALLBACK_PX: f32 = 8.0;

/// Draws a polygon with earcut triangulation. Falls back to a filled bounding box when
/// the element is small on screen, and skips sub-pixel elements entirely.
fn draw_polygon(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &[f64; 4],
    polygon: &gdsr::Polygon,
    color: Color32,
) {
    let points = polygon.points();
    if points.len() < 3 {
        return;
    }

    let Some(bbox) = points_bbox(points) else {
        return;
    };
    if !bbox_overlaps(&bbox, visible) {
        return;
    }

    let s_min = viewport.world_to_screen(bbox[0], bbox[1], rect);
    let s_max = viewport.world_to_screen(bbox[2], bbox[3], rect);
    let sw = (s_max.x - s_min.x).abs();
    let sh = (s_min.y - s_max.y).abs(); // Y flipped

    if sw < 2.0 && sh < 2.0 {
        return;
    }

    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);

    if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
        let bbox = Rect::from_two_pos(s_min, s_max);
        painter.rect_filled(bbox, 0.0, fill);
        painter.rect_stroke(
            bbox,
            0.0,
            Stroke::new(1.0, color),
            egui::StrokeKind::Outside,
        );
        return;
    }

    let screen_pts: Vec<Pos2> = points
        .iter()
        .map(|p| viewport.world_to_screen(p.x().absolute_value(), p.y().absolute_value(), rect))
        .collect();

    // earcutr expects open polygons — remove the closing point if present
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

    let stroke = Stroke::new(1.0, color);
    for i in 0..open_pts.len() {
        let next = (i + 1) % open_pts.len();
        painter.line_segment([open_pts[i], open_pts[next]], stroke);
    }
}

/// Draws a path as connected line segments. Width is scaled by zoom and clamped to
/// `[1.0, 20.0]` pixels. Falls back to a single bbox diagonal for small-on-screen paths.
fn draw_path(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &[f64; 4],
    path: &gdsr::Path,
    color: Color32,
) {
    let points = path.points();
    if points.len() < 2 {
        return;
    }

    let Some(bbox) = points_bbox(points) else {
        return;
    };
    if !bbox_overlaps(&bbox, visible) {
        return;
    }

    let s_min = viewport.world_to_screen(bbox[0], bbox[1], rect);
    let s_max = viewport.world_to_screen(bbox[2], bbox[3], rect);
    let sw = (s_max.x - s_min.x).abs();
    let sh = (s_min.y - s_max.y).abs();
    if sw < 1.0 && sh < 1.0 {
        return;
    }

    if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
        let stroke = Stroke::new(1.0, color);
        painter.line_segment([s_min, s_max], stroke);
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

/// Draws a text label at its origin. Font size scales logarithmically with zoom
/// and is clamped to `[4.0, 48.0]` pixels; labels below minimum size are skipped.
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
