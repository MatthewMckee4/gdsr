use std::collections::HashMap;

use egui::epaint;
use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind};
use gdsr::{Dimensions, Element, Library};

use crate::state::LayerState;
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

/// Bundles the rendering context passed to every `Drawable::draw` call.
///
/// Geometry is batched by layer into large meshes (`layer_meshes`) to minimize
/// per-frame clones on cache hits. Non-mesh shapes (text, rect fallbacks) go
/// into `extra_shapes`. The `screen_pts_buf` is a reusable scratch buffer to
/// avoid per-element allocations.
pub struct DrawContext<'a> {
    pub painter: &'a egui::Painter,
    pub layer_meshes: &'a mut HashMap<(u16, u16), Mesh>,
    pub extra_shapes: &'a mut Vec<Shape>,
    pub viewport: &'a Viewport,
    pub rect: Rect,
    pub visible: &'a WorldBBox,
    pub layer_state: &'a mut LayerState,
    pub library: Option<&'a Library>,
    pub current_element_idx: Option<u32>,
    pub tessellation_cache: &'a mut HashMap<u32, Vec<usize>>,
    pub screen_pts_buf: &'a mut Vec<Pos2>,
}

impl DrawContext<'_> {
    /// Merges a mesh's geometry into the batched mesh for the given layer.
    pub fn merge_mesh(&mut self, key: (u16, u16), src: &Mesh) {
        let dst = self.layer_meshes.entry(key).or_default();
        let base = dst.vertices.len() as u32;
        dst.vertices.extend_from_slice(&src.vertices);
        dst.indices.extend(src.indices.iter().map(|&i| i + base));
    }

    pub fn rect_filled(&mut self, rect: Rect, corner_radius: f32, fill: Color32) {
        self.extra_shapes
            .push(Shape::from(epaint::RectShape::filled(
                rect,
                corner_radius,
                fill,
            )));
    }

    pub fn rect_stroke(
        &mut self,
        rect: Rect,
        corner_radius: f32,
        stroke: Stroke,
        kind: StrokeKind,
    ) {
        self.extra_shapes
            .push(Shape::from(epaint::RectShape::stroke(
                rect,
                corner_radius,
                stroke,
                kind,
            )));
    }

    pub fn line_segment(&mut self, points: [Pos2; 2], stroke: Stroke) {
        self.extra_shapes
            .push(Shape::LineSegment { points, stroke });
    }

    pub fn text(
        &mut self,
        pos: Pos2,
        anchor: egui::Align2,
        text: &str,
        font_id: FontId,
        color: Color32,
    ) {
        let galley = self.painter.layout_no_wrap(text.to_owned(), font_id, color);
        let rect = anchor.anchor_size(pos, galley.size());
        self.extra_shapes
            .push(Shape::galley(rect.min, galley, color));
    }
}

/// Pre-tessellates a polyline stroke into a [`Mesh`] of quads (2 triangles per edge).
/// For `closed` polylines, the last point connects back to the first.
pub(crate) fn stroke_polyline_to_mesh(points: &[Pos2], stroke: Stroke, closed: bool) -> Mesh {
    let mut mesh = Mesh::default();
    if points.len() < 2 {
        return mesh;
    }
    let half_w = stroke.width / 2.0;
    let edge_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };

    for i in 0..edge_count {
        let p0 = points[i];
        let p1 = points[(i + 1) % points.len()];
        let dir = p1 - p0;
        let len = dir.length();
        if len < 1e-6 {
            continue;
        }
        let normal = egui::Vec2::new(-dir.y, dir.x) / len * half_w;
        let base = mesh.vertices.len() as u32;
        for pos in [p0 + normal, p0 - normal, p1 + normal, p1 - normal] {
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv: egui::epaint::WHITE_UV,
                color: stroke.color,
            });
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }
    mesh
}

/// Trait for viewer-drawable elements. Provides layer info, bounding box, and drawing.
pub trait Drawable {
    /// Returns all `(layer, data_type)` pairs this element contributes to.
    fn layer_keys(&self) -> Vec<(u16, u16)>;

    /// Returns the world-space bounding box, or `None` for elements without geometry.
    fn world_bbox(&self) -> Option<WorldBBox>;

    /// Draws this element onto the painter, resolving its own color and visibility.
    fn draw(&self, ctx: &mut DrawContext);
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

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), self.data_type());
        if ctx.layer_state.hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();
        if points.len() < 3 {
            return;
        }

        let Some(bbox) = self.world_bbox() else {
            return;
        };
        if !bbox.overlaps(ctx.visible) {
            return;
        }

        let s_min = ctx
            .viewport
            .world_to_screen(bbox.min_x, bbox.min_y, ctx.rect);
        let s_max = ctx
            .viewport
            .world_to_screen(bbox.max_x, bbox.max_y, ctx.rect);
        let sw = (s_max.x - s_min.x).abs();
        let sh = (s_min.y - s_max.y).abs();

        if sw < 2.0 && sh < 2.0 {
            return;
        }

        let color = ctx.layer_state.layer_colors.get(key.0, key.1);
        let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);

        if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
            let bbox_rect = Rect::from_two_pos(s_min, s_max);
            ctx.rect_filled(bbox_rect, 0.0, fill);
            ctx.rect_stroke(bbox_rect, 0.0, Stroke::new(1.0, color), StrokeKind::Outside);
            return;
        }

        ctx.screen_pts_buf.clear();
        ctx.screen_pts_buf.extend(points.iter().map(|p| {
            ctx.viewport
                .world_to_screen(p.x().absolute_value(), p.y().absolute_value(), ctx.rect)
        }));

        let open_len = if ctx.screen_pts_buf.len() >= 2
            && ctx.screen_pts_buf.first() == ctx.screen_pts_buf.last()
        {
            ctx.screen_pts_buf.len() - 1
        } else {
            ctx.screen_pts_buf.len()
        };

        if open_len < 3 {
            return;
        }

        let coords: Vec<f64> = ctx.screen_pts_buf[..open_len]
            .iter()
            .flat_map(|p| [f64::from(p.x), f64::from(p.y)])
            .collect();

        let indices = if let Some(idx) = ctx.current_element_idx {
            ctx.tessellation_cache
                .entry(idx)
                .or_insert_with(|| earcutr::earcut(&coords, &[], 2).unwrap_or_default())
                .clone()
        } else {
            earcutr::earcut(&coords, &[], 2).unwrap_or_default()
        };

        let mut mesh = Mesh::default();
        for pt in &ctx.screen_pts_buf[..open_len] {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *pt,
                uv: egui::epaint::WHITE_UV,
                color: fill,
            });
        }
        for idx in indices {
            mesh.indices.push(idx as u32);
        }
        ctx.merge_mesh(key, &mesh);

        let outline = stroke_polyline_to_mesh(
            &ctx.screen_pts_buf[..open_len],
            Stroke::new(1.0, color),
            true,
        );
        ctx.merge_mesh(key, &outline);
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

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), self.data_type());
        if ctx.layer_state.hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();
        if points.len() < 2 {
            return;
        }

        let Some(bbox) = self.world_bbox() else {
            return;
        };
        if !bbox.overlaps(ctx.visible) {
            return;
        }

        let s_min = ctx
            .viewport
            .world_to_screen(bbox.min_x, bbox.min_y, ctx.rect);
        let s_max = ctx
            .viewport
            .world_to_screen(bbox.max_x, bbox.max_y, ctx.rect);
        let sw = (s_max.x - s_min.x).abs();
        let sh = (s_min.y - s_max.y).abs();

        if sw < 1.0 && sh < 1.0 {
            return;
        }

        let color = ctx.layer_state.layer_colors.get(key.0, key.1);

        if sw < BBOX_FALLBACK_PX && sh < BBOX_FALLBACK_PX {
            let stroke = Stroke::new(1.0, color);
            ctx.line_segment([s_min, s_max], stroke);
            return;
        }

        ctx.screen_pts_buf.clear();
        ctx.screen_pts_buf.extend(points.iter().map(|p| {
            ctx.viewport
                .world_to_screen(p.x().absolute_value(), p.y().absolute_value(), ctx.rect)
        }));

        let width_px = self
            .width()
            .map(|w| (w.absolute_value() * ctx.viewport.zoom) as f32)
            .unwrap_or(1.0)
            .clamp(1.0, 20.0);

        let stroke_mesh =
            stroke_polyline_to_mesh(ctx.screen_pts_buf, Stroke::new(width_px, color), false);
        ctx.merge_mesh(key, &stroke_mesh);
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

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), 0);
        if ctx.layer_state.hidden_layers.contains(&key) {
            return;
        }

        let origin = self.origin();
        let screen_pos = ctx.viewport.world_to_screen(
            origin.x().absolute_value(),
            origin.y().absolute_value(),
            ctx.rect,
        );

        if !ctx.rect.contains(screen_pos) {
            return;
        }

        let font_size = (12.0 * ctx.viewport.zoom.log10().max(1.0)) as f32;
        if font_size < 4.0 {
            return;
        }
        let font_size = font_size.min(48.0);

        let color = ctx.layer_state.layer_colors.get(key.0, key.1);
        ctx.text(
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

    fn draw(&self, ctx: &mut DrawContext) {
        if let Some(element) = self.instance().as_element() {
            for el in self.get_elements_in_grid(element) {
                el.draw(ctx);
            }
        } else if let Some(cell_name) = self.instance().as_cell() {
            if let Some(lib) = ctx.library {
                if let Some(cell) = lib.get_cell(cell_name) {
                    for polygon in cell.polygons() {
                        for el in self.get_elements_in_grid(&Element::Polygon(polygon.clone())) {
                            el.draw(ctx);
                        }
                    }
                    for path in cell.paths() {
                        for el in self.get_elements_in_grid(&Element::Path(path.clone())) {
                            el.draw(ctx);
                        }
                    }
                    for text in cell.texts() {
                        for el in self.get_elements_in_grid(&Element::Text(text.clone())) {
                            el.draw(ctx);
                        }
                    }
                    for reference in cell.references() {
                        for el in self.get_elements_in_grid(&Element::Reference(reference.clone()))
                        {
                            el.draw(ctx);
                        }
                    }
                }
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

    fn draw(&self, ctx: &mut DrawContext) {
        match self {
            Self::Polygon(p) => p.draw(ctx),
            Self::Path(p) => p.draw(ctx),
            Self::Text(t) => t.draw(ctx),
            Self::Reference(r) => r.draw(ctx),
        }
    }
}
