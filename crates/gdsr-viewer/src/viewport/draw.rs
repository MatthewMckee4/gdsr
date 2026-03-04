use std::collections::HashSet;

use egui::{Color32, Rect};
use gdsr::Element;

use super::Viewport;
use crate::colors::LayerColorMap;
use crate::drawable::{Drawable, WorldBBox};
use crate::spatial::SpatialGrid;

/// Fallback drawing path: iterates over every element without spatial indexing.
pub(crate) fn draw_elements_flat(
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

/// Dispatches a single element to its `Drawable::draw` impl after layer filtering.
pub(crate) fn draw_element(
    painter: &egui::Painter,
    viewport: &Viewport,
    rect: Rect,
    visible: &WorldBBox,
    element: &Element,
    hidden_layers: &HashSet<(u16, u16)>,
    layer_colors: &mut LayerColorMap,
) {
    let keys = element.layer_keys();
    if keys.iter().all(|key| hidden_layers.contains(key)) {
        return;
    }
    if let Some(&key) = keys.first() {
        let color = layer_colors.get(key.0, key.1);
        element.draw(painter, viewport, rect, visible, color);
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
