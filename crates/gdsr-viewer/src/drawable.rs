use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke};
use gdsr::{Dimensions, Element, Path, Polygon, Reference, Text};

use crate::viewport::Viewport;

/// Screen-pixel threshold below which polygons render as a filled bounding box instead of
/// full triangulation. Avoids expensive earcut calls for elements that are just a few pixels.
const BBOX_FALLBACK_PX: f32 = 8.0;

/// Axis-aligned bounding box in absolute world coordinates (f64).
///
/// GDS elements store coordinates as `Unit` values (integer or float with a scale factor).
/// This type holds the result of converting those to absolute f64 values, giving the
/// viewer a uniform representation for culling, spatial indexing, and bounds computation.
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

    /// Returns the smallest bbox containing both `self` and `other`.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }
}

pub trait Drawable {
    fn layer(&self) -> u16;
    fn data_type(&self) -> u16;

    fn layer_key(&self) -> (u16, u16) {
        (self.layer(), self.data_type())
    }

    /// World-space axis-aligned bounding box.
    fn world_bbox(&self) -> Option<WorldBBox>;

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        color: Color32,
    );
}

/// Computes the world-space AABB from a [`Dimensions`] implementor,
/// returning `None` for degenerate (point-like) bounding boxes.
fn dimensions_bbox(item: &impl Dimensions) -> Option<WorldBBox> {
    let (min, max) = item.bounding_box();
    let (min_x, min_y) = (min.x().absolute_value(), min.y().absolute_value());
    let (max_x, max_y) = (max.x().absolute_value(), max.y().absolute_value());
    if min_x == max_x && min_y == max_y {
        return None;
    }
    Some(WorldBBox::new(min_x, min_y, max_x, max_y))
}

impl Drawable for Polygon {
    fn layer(&self) -> u16 {
        self.layer()
    }

    fn data_type(&self) -> u16 {
        self.data_type()
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        dimensions_bbox(self)
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        color: Color32,
    ) {
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

impl Drawable for Path {
    fn layer(&self) -> u16 {
        self.layer()
    }

    fn data_type(&self) -> u16 {
        self.data_type()
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        dimensions_bbox(self)
    }

    fn draw(
        &self,
        painter: &egui::Painter,
        viewport: &Viewport,
        rect: Rect,
        visible: &WorldBBox,
        color: Color32,
    ) {
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

impl Drawable for Text {
    fn layer(&self) -> u16 {
        self.layer()
    }

    fn data_type(&self) -> u16 {
        0
    }

    /// Returns the origin as a point bbox. Text has no spatial extent but still
    /// needs to participate in bounds computation and spatial grid building.
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
        color: Color32,
    ) {
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

        painter.text(
            screen_pos,
            egui::Align2::LEFT_BOTTOM,
            self.text(),
            FontId::monospace(font_size),
            color,
        );
    }
}

impl Drawable for Reference {
    fn layer(&self) -> u16 {
        0
    }

    fn data_type(&self) -> u16 {
        0
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        None
    }

    fn draw(
        &self,
        _painter: &egui::Painter,
        _viewport: &Viewport,
        _rect: Rect,
        _visible: &WorldBBox,
        _color: Color32,
    ) {
    }
}

impl Drawable for Element {
    fn layer(&self) -> u16 {
        match self {
            Self::Polygon(p) => p.layer(),
            Self::Path(p) => p.layer(),
            Self::Text(t) => t.layer(),
            Self::Reference(r) => r.layer(),
        }
    }

    fn data_type(&self) -> u16 {
        match self {
            Self::Polygon(p) => p.data_type(),
            Self::Path(p) => p.data_type(),
            Self::Text(t) => t.data_type(),
            Self::Reference(r) => r.data_type(),
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
        color: Color32,
    ) {
        match self {
            Self::Polygon(p) => p.draw(painter, viewport, rect, visible, color),
            Self::Path(p) => p.draw(painter, viewport, rect, visible, color),
            Self::Text(t) => t.draw(painter, viewport, rect, visible, color),
            Self::Reference(r) => r.draw(painter, viewport, rect, visible, color),
        }
    }
}
