use std::collections::HashMap;

use egui::epaint;
use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};
use gdsr::{DataType, Dimensions, Element, Layer, Library};

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
    pub layer_meshes: &'a mut HashMap<(Layer, DataType), Mesh>,
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
    pub fn merge_mesh(&mut self, key: (Layer, DataType), src: &Mesh) {
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

/// Tests whether the point `(px, py)` lies inside the polygon defined by `verts`
/// using the ray-casting algorithm.
fn point_in_polygon(px: f64, py: f64, verts: &[(f64, f64)]) -> bool {
    let n = verts.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = verts[i];
        let (xj, yj) = verts[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Returns the squared distance from point `(px, py)` to the line segment `(ax, ay)-(bx, by)`.
fn point_to_segment_dist_sq(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-30 {
        let ex = px - ax;
        let ey = py - ay;
        return ex * ex + ey * ey;
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let cx = ax + t * dx - px;
    let cy = ay + t * dy - py;
    cx * cx + cy * cy
}

/// Trait for viewer-drawable elements. Provides layer info, bounding box, and drawing.
pub trait Drawable {
    /// Returns all `(layer, data_type)` pairs this element contributes to.
    fn layer_keys(&self) -> Vec<(Layer, DataType)>;

    /// Returns the world-space bounding box, or `None` for elements without geometry.
    fn world_bbox(&self) -> Option<WorldBBox>;

    /// Draws this element onto the painter, resolving its own color and visibility.
    fn draw(&self, ctx: &mut DrawContext);

    /// Returns `true` if the world-space point `(wx, wy)` hits this element.
    /// `zoom` is pixels-per-world-unit, used for screen-pixel tolerances.
    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool;
}

/// Screen-pixel threshold below which polygons render as a filled bounding box.
const BBOX_FALLBACK_PX: f32 = 8.0;

impl Drawable for gdsr::Polygon {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
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

    fn hit_test(&self, wx: f64, wy: f64, _zoom: f64) -> bool {
        if let Some(bbox) = self.world_bbox() {
            if wx < bbox.min_x || wx > bbox.max_x || wy < bbox.min_y || wy > bbox.max_y {
                return false;
            }
        }
        let pts = self.points();
        let mut verts: Vec<(f64, f64)> = pts
            .iter()
            .map(|p| (p.x().absolute_value(), p.y().absolute_value()))
            .collect();
        if verts.len() >= 2 && verts.first() == verts.last() {
            verts.pop();
        }
        point_in_polygon(wx, wy, &verts)
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
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
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

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let pts = self.points();
        if pts.len() < 2 {
            return false;
        }
        let half_width = self
            .width()
            .map(|w| w.absolute_value() / 2.0)
            .unwrap_or(0.0);
        let min_tolerance = 3.0 / zoom;
        let tolerance = half_width.max(min_tolerance);
        let tol_sq = tolerance * tolerance;
        for pair in pts.windows(2) {
            let (ax, ay) = (pair[0].x().absolute_value(), pair[0].y().absolute_value());
            let (bx, by) = (pair[1].x().absolute_value(), pair[1].y().absolute_value());
            if point_to_segment_dist_sq(wx, wy, ax, ay, bx, by) <= tol_sq {
                return true;
            }
        }
        false
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

        if let Some(poly_pts) = self.to_polygon_points(16) {
            ctx.screen_pts_buf.clear();
            ctx.screen_pts_buf.extend(poly_pts.iter().map(|p| {
                ctx.viewport.world_to_screen(
                    p.x().absolute_value(),
                    p.y().absolute_value(),
                    ctx.rect,
                )
            }));

            let open_len = if ctx.screen_pts_buf.len() >= 2
                && ctx.screen_pts_buf.first() == ctx.screen_pts_buf.last()
            {
                ctx.screen_pts_buf.len() - 1
            } else {
                ctx.screen_pts_buf.len()
            };

            if open_len >= 3 {
                let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 80);

                let coords: Vec<f64> = ctx.screen_pts_buf[..open_len]
                    .iter()
                    .flat_map(|p| [f64::from(p.x), f64::from(p.y)])
                    .collect();

                let indices = earcutr::earcut(&coords, &[], 2).unwrap_or_default();

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
                return;
            }
        }

        // Fallback: render as 1px centerline stroke
        ctx.screen_pts_buf.clear();
        ctx.screen_pts_buf.extend(points.iter().map(|p| {
            ctx.viewport
                .world_to_screen(p.x().absolute_value(), p.y().absolute_value(), ctx.rect)
        }));

        let stroke_mesh =
            stroke_polyline_to_mesh(ctx.screen_pts_buf, Stroke::new(1.0, color), false);
        ctx.merge_mesh(key, &stroke_mesh);
    }
}

impl Drawable for gdsr::Text {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        vec![(self.layer(), DataType::new(0))]
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        let origin = self.origin();
        let x = origin.x().absolute_value();
        let y = origin.y().absolute_value();
        Some(WorldBBox::new(x, y, x, y))
    }

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let origin = self.origin();
        let ox = origin.x().absolute_value();
        let oy = origin.y().absolute_value();
        let radius = 5.0 / zoom;
        let dx = wx - ox;
        let dy = wy - oy;
        dx * dx + dy * dy <= radius * radius
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), DataType::new(0));
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

impl Drawable for gdsr::GdsBox {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        vec![(self.layer(), self.box_type())]
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

    fn hit_test(&self, wx: f64, wy: f64, _zoom: f64) -> bool {
        if let Some(bbox) = self.world_bbox() {
            wx >= bbox.min_x && wx <= bbox.max_x && wy >= bbox.min_y && wy <= bbox.max_y
        } else {
            false
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), self.box_type());
        if ctx.layer_state.hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();

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

impl Drawable for gdsr::Node {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        vec![(self.layer(), self.node_type())]
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

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let radius = 3.0 / zoom;
        let r_sq = radius * radius;
        for p in self.points() {
            let dx = wx - p.x().absolute_value();
            let dy = wy - p.y().absolute_value();
            if dx * dx + dy * dy <= r_sq {
                return true;
            }
        }
        false
    }

    fn draw(&self, ctx: &mut DrawContext) {
        let key = (self.layer(), self.node_type());
        if ctx.layer_state.hidden_layers.contains(&key) {
            return;
        }

        let points = self.points();
        if points.is_empty() {
            return;
        }

        let Some(bbox) = self.world_bbox() else {
            return;
        };
        if !bbox.overlaps(ctx.visible) {
            return;
        }

        let color = ctx.layer_state.layer_colors.get(key.0, key.1);
        let marker_size = 3.0_f32;

        for point in points {
            let screen = ctx.viewport.world_to_screen(
                point.x().absolute_value(),
                point.y().absolute_value(),
                ctx.rect,
            );
            if ctx.rect.contains(screen) {
                let rect = Rect::from_center_size(screen, egui::Vec2::splat(marker_size * 2.0));
                ctx.rect_filled(rect, 0.0, color);
            }
        }
    }
}

impl Drawable for gdsr::Reference {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
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

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        if let Some(element) = self.instance().as_element() {
            for el in self.get_elements_in_grid(element) {
                if el.hit_test(wx, wy, zoom) {
                    return true;
                }
            }
        }
        false
    }

    fn draw(&self, ctx: &mut DrawContext) {
        if let Some(element) = self.instance().as_element() {
            for el in self.get_elements_in_grid(element) {
                el.draw(ctx);
            }
        } else if let Some(cell_name) = self.instance().as_cell() {
            if let Some(lib) = ctx.library {
                if let Some(cell) = lib.get_cell(cell_name) {
                    for element in cell.iter_elements() {
                        for el in self.get_elements_in_grid(element) {
                            el.draw(ctx);
                        }
                    }
                }
            }
        }
    }
}

impl Drawable for Element {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        match self {
            Self::Polygon(p) => p.layer_keys(),
            Self::Box(b) => b.layer_keys(),
            Self::Node(n) => n.layer_keys(),
            Self::Path(p) => p.layer_keys(),
            Self::Text(t) => t.layer_keys(),
            Self::Reference(r) => r.layer_keys(),
        }
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        match self {
            Self::Polygon(p) => p.world_bbox(),
            Self::Box(b) => b.world_bbox(),
            Self::Node(n) => n.world_bbox(),
            Self::Path(p) => p.world_bbox(),
            Self::Text(t) => t.world_bbox(),
            Self::Reference(r) => r.world_bbox(),
        }
    }

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        match self {
            Self::Polygon(p) => p.hit_test(wx, wy, zoom),
            Self::Box(b) => b.hit_test(wx, wy, zoom),
            Self::Node(n) => n.hit_test(wx, wy, zoom),
            Self::Path(p) => p.hit_test(wx, wy, zoom),
            Self::Text(t) => t.hit_test(wx, wy, zoom),
            Self::Reference(r) => r.hit_test(wx, wy, zoom),
        }
    }

    fn draw(&self, ctx: &mut DrawContext) {
        match self {
            Self::Polygon(p) => p.draw(ctx),
            Self::Box(b) => b.draw(ctx),
            Self::Node(n) => n.draw(ctx),
            Self::Path(p) => p.draw(ctx),
            Self::Text(t) => t.draw(ctx),
            Self::Reference(r) => r.draw(ctx),
        }
    }
}

const HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(255, 255, 100);
const HIGHLIGHT_WIDTH: f32 = 2.0;

/// Draws a highlight overlay for the given element using the painter directly (not cached).
pub fn draw_highlight(
    element: &Element,
    viewport: &crate::viewport::Viewport,
    painter: &egui::Painter,
    rect: Rect,
) {
    let stroke = Stroke::new(HIGHLIGHT_WIDTH, HIGHLIGHT_COLOR);
    match element {
        Element::Polygon(p) => {
            let pts = p.points();
            if pts.len() < 3 {
                return;
            }
            let screen: Vec<Pos2> = pts
                .iter()
                .map(|pt| {
                    viewport.world_to_screen(pt.x().absolute_value(), pt.y().absolute_value(), rect)
                })
                .collect();
            let mut open = screen;
            if open.len() >= 2 && open.first() == open.last() {
                open.pop();
            }
            let mesh = stroke_polyline_to_mesh(&open, stroke, true);
            painter.add(Shape::mesh(mesh));
        }
        Element::Box(b) => {
            let pts = b.points();
            let screen: Vec<Pos2> = pts
                .iter()
                .map(|pt| {
                    viewport.world_to_screen(pt.x().absolute_value(), pt.y().absolute_value(), rect)
                })
                .collect();
            let mut open = screen;
            if open.len() >= 2 && open.first() == open.last() {
                open.pop();
            }
            let mesh = stroke_polyline_to_mesh(&open, stroke, true);
            painter.add(Shape::mesh(mesh));
        }
        Element::Path(p) => {
            let pts = p.points();
            if pts.len() < 2 {
                return;
            }
            let screen: Vec<Pos2> = pts
                .iter()
                .map(|pt| {
                    viewport.world_to_screen(pt.x().absolute_value(), pt.y().absolute_value(), rect)
                })
                .collect();
            let width_px = p
                .width()
                .map(|w| (w.absolute_value() * viewport.zoom) as f32)
                .unwrap_or(1.0)
                .clamp(1.0, 20.0);
            let mesh = stroke_polyline_to_mesh(
                &screen,
                Stroke::new(width_px + 2.0, HIGHLIGHT_COLOR),
                false,
            );
            painter.add(Shape::mesh(mesh));
        }
        Element::Text(t) => {
            let origin = t.origin();
            let screen = viewport.world_to_screen(
                origin.x().absolute_value(),
                origin.y().absolute_value(),
                rect,
            );
            let marker_rect = Rect::from_center_size(screen, Vec2::splat(10.0));
            painter.add(Shape::from(epaint::RectShape::stroke(
                marker_rect,
                2.0,
                stroke,
                StrokeKind::Outside,
            )));
        }
        Element::Node(n) => {
            for pt in n.points() {
                let screen = viewport.world_to_screen(
                    pt.x().absolute_value(),
                    pt.y().absolute_value(),
                    rect,
                );
                let marker_rect = Rect::from_center_size(screen, Vec2::splat(10.0));
                painter.add(Shape::from(epaint::RectShape::stroke(
                    marker_rect,
                    2.0,
                    stroke,
                    StrokeKind::Outside,
                )));
            }
        }
        Element::Reference(r) => {
            if let Some(bbox) = r.world_bbox() {
                let s_min = viewport.world_to_screen(bbox.min_x, bbox.min_y, rect);
                let s_max = viewport.world_to_screen(bbox.max_x, bbox.max_y, rect);
                let highlight_rect = Rect::from_two_pos(s_min, s_max);
                painter.add(Shape::from(epaint::RectShape::stroke(
                    highlight_rect,
                    0.0,
                    stroke,
                    StrokeKind::Outside,
                )));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::helpers::*;

    const SCALE: f64 = 1e-9;
    const ZOOM: f64 = 1e9;

    #[test]
    fn polygon_hit_inside() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn polygon_hit_outside() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(!el.hit_test(200.0 * SCALE, 200.0 * SCALE, ZOOM));
    }

    #[test]
    fn path_hit_on_segment() {
        let el = path(vec![(0, 0), (100, 0)], 1, 0, Some(10));
        assert!(el.hit_test(50.0 * SCALE, 0.0, ZOOM));
    }

    #[test]
    fn path_hit_far_from_segment() {
        let el = path(vec![(0, 0), (100, 0)], 1, 0, Some(10));
        assert!(!el.hit_test(50.0 * SCALE, 100.0 * SCALE, ZOOM));
    }

    #[test]
    fn text_hit_near_origin() {
        let el = text("hello", 500, 600, 1);
        assert!(el.hit_test(500.0 * SCALE, 600.0 * SCALE, ZOOM));
    }

    #[test]
    fn text_hit_far_from_origin() {
        let el = text("hello", 500, 600, 1);
        assert!(!el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn gds_box_hit_inside() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(100, 100),
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Box(b);
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn gds_box_hit_outside() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(100, 100),
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Box(b);
        assert!(!el.hit_test(200.0 * SCALE, 200.0 * SCALE, ZOOM));
    }

    #[test]
    fn node_hit_near_point() {
        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(50, 50)],
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Node(n);
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn node_hit_far_from_point() {
        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(50, 50)],
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Node(n);
        assert!(!el.hit_test(500.0 * SCALE, 500.0 * SCALE, ZOOM));
    }

    #[test]
    fn point_in_polygon_triangle() {
        let tri = [(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)];
        assert!(point_in_polygon(5.0, 3.0, &tri));
        assert!(!point_in_polygon(20.0, 20.0, &tri));
    }

    #[test]
    fn point_to_segment_dist_on_segment() {
        let d = point_to_segment_dist_sq(5.0, 0.0, 0.0, 0.0, 10.0, 0.0);
        assert!(d < 1e-20);
    }

    #[test]
    fn point_to_segment_dist_perpendicular() {
        let d = point_to_segment_dist_sq(5.0, 3.0, 0.0, 0.0, 10.0, 0.0);
        assert!((d - 9.0).abs() < 1e-10);
    }
}
