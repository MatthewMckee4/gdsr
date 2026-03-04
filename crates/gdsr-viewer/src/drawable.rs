use std::collections::HashSet;

use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke};
use gdsr::{Dimensions, Element};

use crate::colors::LayerColorMap;
use crate::viewport::Viewport;

/// World-space axis-aligned bounding box with named fields.
#[derive(Clone, Copy, Debug)]
pub struct WorldBBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl WorldBBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

/// Trait for viewer-drawable elements. Provides layer info, bounding box, and drawing.
pub trait Drawable {
    /// Returns all `(layer, data_type)` pairs this element contributes to.
    fn layer_keys(&self) -> Vec<(u16, u16)>;

    /// Returns the world-space bounding box, or `None` for elements without geometry.
    fn world_bbox(&self) -> Option<WorldBBox>;

    /// Draws this element onto the painter, resolving its own color and visibility.
    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    );
}

/// Screen-pixel threshold below which polygons render as a filled bounding box.
const BBOX_FALLBACK_PX: f32 = 8.0;

impl Drawable for gdsr::Polygon {
    fn layer_keys(&self) -> Vec<(u16, u16)> {
        vec![(self.layer(), self.data_type())]
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        let (min_pt, max_pt) = self.bounding_box();
        Some(WorldBBox::new(
            min_pt.x().absolute_value(),
            min_pt.y().absolute_value(),
            max_pt.x().absolute_value(),
            max_pt.y().absolute_value(),
        ))
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    ) {
        let key = (self.layer(), self.data_type());
        if hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();
        if points.len() < 3 {
            return;
        }

        let Some(bbox) = self.world_bbox() else {
            return;
        };
        if !bbox.overlaps(visible) {
            return;
        }

        let s_min = viewport.world_to_screen(bbox.min_x, bbox.min_y, rect);
        let s_max = viewport.world_to_screen(bbox.max_x, bbox.max_y, rect);
        let sw = (s_max.x - s_min.x).abs();
        let sh = (s_min.y - s_max.y).abs();

        if sw < 2.0 && sh < 2.0 {
            return;
        }

        let color = layer_colors.get(key.0, key.1);
        let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);

        if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
            let bbox_rect = Rect::from_two_pos(s_min, s_max);
            painter.rect_filled(bbox_rect, 0.0, fill);
            painter.rect_stroke(
                bbox_rect,
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
}

impl Drawable for gdsr::Path {
    fn layer_keys(&self) -> Vec<(u16, u16)> {
        vec![(self.layer(), self.data_type())]
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        let (min_pt, max_pt) = self.bounding_box();
        Some(WorldBBox::new(
            min_pt.x().absolute_value(),
            min_pt.y().absolute_value(),
            max_pt.x().absolute_value(),
            max_pt.y().absolute_value(),
        ))
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    ) {
        let key = (self.layer(), self.data_type());
        if hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();
        if points.len() < 2 {
            return;
        }

        let Some(bbox) = self.world_bbox() else {
            return;
        };
        if !bbox.overlaps(visible) {
            return;
        }

        let s_min = viewport.world_to_screen(bbox.min_x, bbox.min_y, rect);
        let s_max = viewport.world_to_screen(bbox.max_x, bbox.max_y, rect);
        let sw = (s_max.x - s_min.x).abs();
        let sh = (s_min.y - s_max.y).abs();

        if sw < 1.0 && sh < 1.0 {
            return;
        }

        let color = layer_colors.get(key.0, key.1);

        if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
            let stroke = Stroke::new(1.0, color);
            painter.line_segment([s_min, s_max], stroke);
            return;
        }

        let screen_pts: Vec<Pos2> = points
            .iter()
            .map(|p| viewport.world_to_screen(p.x().absolute_value(), p.y().absolute_value(), rect))
            .collect();

        let width_px = self
            .width()
            .map(|w| (w.absolute_value() * viewport.zoom) as f32)
            .unwrap_or(1.0)
            .clamp(1.0, 20.0);

        let stroke = Stroke::new(width_px, color);
        for pair in screen_pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], stroke);
        }
    }
}

impl Drawable for gdsr::Text {
    fn layer_keys(&self) -> Vec<(u16, u16)> {
        vec![(self.layer(), 0)]
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        let origin = self.origin();
        let x = origin.x().absolute_value();
        let y = origin.y().absolute_value();
        Some(WorldBBox::new(x, y, x, y))
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        _visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    ) {
        let key = (self.layer(), 0);
        if hidden_layers.contains(&key) {
            return;
        }

        let origin = self.origin();
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

        let color = layer_colors.get(key.0, key.1);
        painter.text(
            screen_pos,
            egui::Align2::LEFT_BOTTOM,
            self.text(),
            FontId::monospace(font_size),
            color,
        );
    }
}

impl Drawable for gdsr::Reference {
    fn layer_keys(&self) -> Vec<(u16, u16)> {
        match self.instance().as_element() {
            Some(element) => element.layer_keys(),
            None => vec![],
        }
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        let element = self.instance().as_element()?;
        let mut result: Option<WorldBBox> = None;
        for el in self.get_elements_in_grid(element) {
            if let Some(bbox) = el.world_bbox() {
                result = Some(match result {
                    Some(acc) => acc.merge(&bbox),
                    None => bbox,
                });
            }
        }
        result
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    ) {
        if let Some(element) = self.instance().as_element() {
            for el in self.get_elements_in_grid(element) {
                el.draw(
                    painter,
                    viewport,
                    rect,
                    visible,
                    hidden_layers,
                    layer_colors,
                );
            }
        }
    }
}

impl Drawable for Element {
    fn layer_keys(&self) -> Vec<(u16, u16)> {
        match self {
            Self::Polygon(p) => p.layer_keys(),
            Self::Path(p) => p.layer_keys(),
            Self::Text(t) => t.layer_keys(),
            Self::Reference(r) => r.layer_keys(),
        }
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        match self {
            Self::Polygon(p) => p.world_bbox(),
            Self::Path(p) => p.world_bbox(),
            Self::Text(t) => t.world_bbox(),
            Self::Reference(r) => r.world_bbox(),
        }
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        hidden_layers: &HashSet<(u16, u16)>,
        layer_colors: &mut LayerColorMap,
    ) {
        match self {
            Self::Polygon(p) => p.draw(
                painter,
                viewport,
                rect,
                visible,
                hidden_layers,
                layer_colors,
            ),
            Self::Path(p) => p.draw(
                painter,
                viewport,
                rect,
                visible,
                hidden_layers,
                layer_colors,
            ),
            Self::Text(t) => t.draw(
                painter,
                viewport,
                rect,
                visible,
                hidden_layers,
                layer_colors,
            ),
            Self::Reference(r) => {
                r.draw(
                    painter,
                    viewport,
                    rect,
                    visible,
                    hidden_layers,
                    layer_colors,
                );
            }
        }
    }
}
