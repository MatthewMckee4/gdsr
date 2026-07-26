use std::collections::{HashMap, HashSet};

use egui::epaint;
use egui::{Color32, FontId, Mesh, Pos2, Rect, Shape, Stroke, StrokeKind};
use gdsr::{DataType, Dimensions, Element, Layer, Library};

use crate::state::LayerState;
use crate::viewport::Viewport;

/// World-space axis-aligned bounding box with named fields.
pub type WorldBBox = gdsr::WorldBounds;

pub fn cell_world_bbox(cell_name: &str, library: &Library) -> Option<WorldBBox> {
    library.hierarchy_bounds(cell_name).ok()?.root_bounds()
}

pub fn cell_world_bboxes(library: &Library, root_cell: &str) -> HashMap<String, Option<WorldBBox>> {
    let Ok(report) = library.hierarchy_bounds(root_cell) else {
        return HashMap::new();
    };
    report
        .cell_bounds()
        .iter()
        .map(|(cell_name, bounds)| (cell_name.clone(), *bounds))
        .collect()
}

fn reference_bbox_from_source_bbox(
    reference: &gdsr::Reference,
    source_bbox: WorldBBox,
) -> Option<WorldBBox> {
    source_bbox.transformed_by_grid(reference.grid())
}

pub fn reference_grid_count(reference: &gdsr::Reference) -> u64 {
    let grid = reference.grid();
    u64::from(grid.columns()) * u64::from(grid.rows())
}

fn reference_screen_size(bbox: WorldBBox, ctx: &DrawContext) -> (f32, f32, Rect) {
    let s_min = ctx
        .viewport
        .world_to_screen(bbox.min_x, bbox.min_y, ctx.rect);
    let s_max = ctx
        .viewport
        .world_to_screen(bbox.max_x, bbox.max_y, ctx.rect);
    let sw = (s_max.x - s_min.x).abs();
    let sh = (s_min.y - s_max.y).abs();
    (sw, sh, Rect::from_two_pos(s_min, s_max))
}

pub fn reference_instance_bbox(
    reference: &gdsr::Reference,
    library: Option<&Library>,
    cell_bbox_cache: &mut HashMap<String, Option<WorldBBox>>,
) -> Option<WorldBBox> {
    if let Some(cell_name) = reference.instance().as_cell() {
        let source_bbox = named_cell_bbox(cell_name, library?, cell_bbox_cache)?;
        return reference_bbox_from_source_bbox(reference, source_bbox);
    }
    let element = reference.instance().as_element()?;
    if let Element::Reference(nested) = element.as_ref().as_ref() {
        return inline_reference_instance_bbox(reference.grid(), nested, library, cell_bbox_cache);
    }
    reference_bbox_from_source_bbox(reference, element.as_ref().as_ref().world_bbox()?)
}

fn named_cell_bbox(
    cell_name: &str,
    library: &Library,
    cell_bbox_cache: &mut HashMap<String, Option<WorldBBox>>,
) -> Option<WorldBBox> {
    if let Some(cached) = cell_bbox_cache.get(cell_name) {
        return *cached;
    }
    let report = library.hierarchy_bounds(cell_name).ok()?;
    let root_bounds = report.root_bounds();
    cell_bbox_cache.extend(
        report
            .cell_bounds()
            .iter()
            .map(|(name, bounds)| (name.clone(), *bounds)),
    );
    root_bounds
}

fn inline_reference_instance_bbox(
    outer_grid: &gdsr::Grid,
    reference: &gdsr::Reference,
    library: Option<&Library>,
    cell_bbox_cache: &mut HashMap<String, Option<WorldBBox>>,
) -> Option<WorldBBox> {
    let mut current = reference;
    let mut grids = vec![outer_grid];
    let source_bbox = loop {
        grids.push(current.grid());
        if let Some(cell_name) = current.instance().as_cell() {
            break named_cell_bbox(cell_name, library?, cell_bbox_cache)?;
        }
        let element = current.instance().as_element()?;
        if let Element::Reference(reference) = element.as_ref().as_ref() {
            current = reference;
        } else {
            break element.as_ref().as_ref().world_bbox()?;
        }
    };

    let mut bounds = Some(source_bbox);
    for grid in grids.into_iter().rev() {
        bounds = bounds.and_then(|bounds| bounds.transformed_by_grid(grid));
    }
    bounds
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
    /// When true, the element is drawn with a brighter fill and bolder outline.
    pub highlight: bool,
    /// When true, un-flattened cell references draw as outlined bounding boxes
    /// with the cell name centered, instead of expanding their contents.
    pub show_ref_bbox: bool,
    pub reference_depth: u32,
    pub reference_stack: Vec<String>,
    pub cell_bbox_cache: &'a mut HashMap<String, Option<WorldBBox>>,
    pub cell_complexity_cache: &'a mut HashMap<(String, u32), u64>,
}

/// Fill alpha for normal and highlighted elements.
const FILL_ALPHA: u8 = 80;
const HIGHLIGHT_FILL_ALPHA: u8 = 160;
/// Stroke width for normal and highlighted outlines.
const STROKE_WIDTH: f32 = 1.0;
const HIGHLIGHT_STROKE_WIDTH: f32 = 3.0;
const REF_LOAD_THRESHOLD_PX: f32 = 24.0;
const REF_DENSE_GRID_MIN_INSTANCES: u64 = 128;
const REF_LOAD_MIN_AVG_INSTANCE_AREA_PX: f32 = 16.0;
const REF_MAX_GRID_EXPANSION: u64 = 8_192;

impl DrawContext<'_> {
    /// Returns the fill and stroke colors for the given layer, brightened when highlighted.
    pub fn element_style(&mut self, layer: Layer, dt: DataType) -> (Color32, Color32, f32) {
        let color = self.layer_state.layer_colors.get(layer, dt);
        if self.highlight {
            let brighten = |c: u8| c.saturating_add((255 - c) / 3);
            let bright = Color32::from_rgb(
                brighten(color.r()),
                brighten(color.g()),
                brighten(color.b()),
            );
            let fill = Color32::from_rgba_unmultiplied(
                bright.r(),
                bright.g(),
                bright.b(),
                HIGHLIGHT_FILL_ALPHA,
            );
            (fill, bright, HIGHLIGHT_STROKE_WIDTH)
        } else {
            let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), FILL_ALPHA);
            (fill, color, STROKE_WIDTH)
        }
    }

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
pub fn stroke_polyline_to_mesh(points: &[Pos2], stroke: Stroke, closed: bool) -> Mesh {
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

/// Screen-pixel threshold below which detailed geometry renders as a cheaper proxy.
const BBOX_FALLBACK_PX: f32 = 8.0;

fn invalid_screen_bbox(width_px: f32, height_px: f32) -> bool {
    !width_px.is_finite() || !height_px.is_finite()
}

fn should_use_bbox_fallback(width_px: f32, height_px: f32) -> bool {
    width_px < BBOX_FALLBACK_PX || height_px < BBOX_FALLBACK_PX
}

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

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let tolerance = 3.0 / zoom;
        let tol_sq = tolerance * tolerance;
        if let Some(bbox) = self.world_bbox() {
            if wx < bbox.min_x - tolerance
                || wx > bbox.max_x + tolerance
                || wy < bbox.min_y - tolerance
                || wy > bbox.max_y + tolerance
            {
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
        if point_in_polygon(wx, wy, &verts) {
            return true;
        }
        // Check proximity to edges so points near the boundary still hit.
        let n = verts.len();
        for i in 0..n {
            let (ax, ay) = verts[i];
            let (bx, by) = verts[(i + 1) % n];
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

        if invalid_screen_bbox(sw, sh) || (sw < 2.0 && sh < 2.0) {
            return;
        }

        let (fill, stroke_color, stroke_width) = ctx.element_style(key.0, key.1);

        if should_use_bbox_fallback(sw, sh) {
            let bbox_rect = Rect::from_two_pos(s_min, s_max);
            ctx.rect_filled(bbox_rect, 0.0, fill);
            ctx.rect_stroke(
                bbox_rect,
                0.0,
                Stroke::new(stroke_width, stroke_color),
                StrokeKind::Outside,
            );
            return;
        }

        // Determine closure from world-space points so that open_len is stable
        // across zoom levels. Screen-space f32 equality can flip between frames,
        // which would invalidate cached tessellation indices.
        let world_closed = points.len() >= 2 && points.first() == points.last();

        ctx.screen_pts_buf.clear();
        ctx.screen_pts_buf.extend(points.iter().map(|p| {
            ctx.viewport
                .world_to_screen(p.x().absolute_value(), p.y().absolute_value(), ctx.rect)
        }));

        let open_len = if world_closed {
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
            Stroke::new(stroke_width, stroke_color),
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
        let mut points = self.points().iter();
        let first = points.next()?;
        let first_x = first.x().absolute_value();
        let first_y = first.y().absolute_value();
        let mut bounds = WorldBBox::new(first_x, first_y, first_x, first_y);
        for point in points {
            let x = point.x().absolute_value();
            let y = point.y().absolute_value();
            bounds = bounds.merge(&WorldBBox::new(x, y, x, y));
        }
        let half_width = self
            .width()
            .map(|w| w.absolute_value().abs() / 2.0)
            .unwrap_or(0.0);
        let begin_ext = self
            .begin_extension()
            .map_or(0.0, |u| u.absolute_value().max(0.0));
        let end_ext = self
            .end_extension()
            .map_or(0.0, |u| u.absolute_value().max(0.0));
        let pad = half_width + begin_ext.max(end_ext);
        Some(WorldBBox::new(
            bounds.min_x - pad,
            bounds.min_y - pad,
            bounds.max_x + pad,
            bounds.max_y + pad,
        ))
    }

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let pts = self.points();
        if pts.len() < 2 {
            return false;
        }
        let half_width = self
            .width()
            .map(|w| w.absolute_value().abs() / 2.0)
            .unwrap_or(0.0);
        let min_tolerance = 3.0 / zoom;
        let tolerance = half_width.max(min_tolerance);
        let tol_sq = tolerance * tolerance;

        let path_type = self.path_type().unwrap_or_default();
        let begin_ext = self
            .begin_extension()
            .map_or(0.0, |u| u.absolute_value().max(0.0));
        let end_ext = self
            .end_extension()
            .map_or(0.0, |u| u.absolute_value().max(0.0));

        let mut segments: Vec<(f64, f64, f64, f64)> = pts
            .windows(2)
            .map(|pair| {
                (
                    pair[0].x().absolute_value(),
                    pair[0].y().absolute_value(),
                    pair[1].x().absolute_value(),
                    pair[1].y().absolute_value(),
                )
            })
            .collect();

        if path_type == gdsr::PathType::Overlap && !segments.is_empty() {
            if let Some(first) = segments.first_mut() {
                let dx = first.2 - first.0;
                let dy = first.3 - first.1;
                let len = dx.hypot(dy);
                if len > f64::EPSILON {
                    first.0 -= dx / len * begin_ext;
                    first.1 -= dy / len * begin_ext;
                }
            }
            if let Some(last) = segments.last_mut() {
                let dx = last.2 - last.0;
                let dy = last.3 - last.1;
                let len = dx.hypot(dy);
                if len > f64::EPSILON {
                    last.2 += dx / len * end_ext;
                    last.3 += dy / len * end_ext;
                }
            }
        }

        segments
            .iter()
            .any(|&(ax, ay, bx, by)| point_to_segment_dist_sq(wx, wy, ax, ay, bx, by) <= tol_sq)
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

        if invalid_screen_bbox(sw, sh) || (sw < 1.0 && sh < 1.0) {
            return;
        }

        let (fill, stroke_color, stroke_width) = ctx.element_style(key.0, key.1);
        if should_use_bbox_fallback(sw, sh) {
            ctx.screen_pts_buf.clear();
            ctx.screen_pts_buf.extend(points.iter().map(|p| {
                ctx.viewport.world_to_screen(
                    p.x().absolute_value(),
                    p.y().absolute_value(),
                    ctx.rect,
                )
            }));
            let stroke = Stroke::new(stroke_width, stroke_color);
            let stroke_mesh = stroke_polyline_to_mesh(ctx.screen_pts_buf, stroke, false);
            ctx.merge_mesh(key, &stroke_mesh);
            return;
        }

        if let Some(poly_pts) = self.to_polygon_points(16) {
            let world_closed = poly_pts.len() >= 2 && poly_pts.first() == poly_pts.last();

            ctx.screen_pts_buf.clear();
            ctx.screen_pts_buf.extend(poly_pts.iter().map(|p| {
                ctx.viewport.world_to_screen(
                    p.x().absolute_value(),
                    p.y().absolute_value(),
                    ctx.rect,
                )
            }));

            let open_len = if world_closed {
                ctx.screen_pts_buf.len() - 1
            } else {
                ctx.screen_pts_buf.len()
            };

            if open_len >= 3 {
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
                    Stroke::new(stroke_width, stroke_color),
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

        let stroke_mesh = stroke_polyline_to_mesh(
            ctx.screen_pts_buf,
            Stroke::new(stroke_width, stroke_color),
            false,
        );
        ctx.merge_mesh(key, &stroke_mesh);
    }
}

impl Drawable for gdsr::Text {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        vec![(self.layer(), self.data_type())]
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
        let key = (self.layer(), self.data_type());
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

        let (_fill, stroke_color, _stroke_width) = ctx.element_style(key.0, key.1);
        ctx.text(
            screen_pos,
            egui::Align2::LEFT_BOTTOM,
            self.text(),
            FontId::monospace(font_size),
            stroke_color,
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

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let tolerance = 3.0 / zoom;
        if let Some(bbox) = self.world_bbox() {
            wx >= bbox.min_x - tolerance
                && wx <= bbox.max_x + tolerance
                && wy >= bbox.min_y - tolerance
                && wy <= bbox.max_y + tolerance
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

        if invalid_screen_bbox(sw, sh) || (sw < 2.0 && sh < 2.0) {
            return;
        }

        let (fill, stroke_color, stroke_width) = ctx.element_style(key.0, key.1);

        if should_use_bbox_fallback(sw, sh) {
            let bbox_rect = Rect::from_two_pos(s_min, s_max);
            ctx.rect_filled(bbox_rect, 0.0, fill);
            ctx.rect_stroke(
                bbox_rect,
                0.0,
                Stroke::new(stroke_width, stroke_color),
                StrokeKind::Outside,
            );
            return;
        }

        // GdsBox always has 5 points with first == last.
        let world_closed = points.len() >= 2 && points.first() == points.last();

        ctx.screen_pts_buf.clear();
        ctx.screen_pts_buf.extend(points.iter().map(|p| {
            ctx.viewport
                .world_to_screen(p.x().absolute_value(), p.y().absolute_value(), ctx.rect)
        }));

        let open_len = if world_closed {
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
            Stroke::new(stroke_width, stroke_color),
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

        let (_fill, stroke_color, stroke_width) = ctx.element_style(key.0, key.1);
        let marker_size = if ctx.highlight { 5.0_f32 } else { 3.0_f32 };

        for point in points {
            let screen = ctx.viewport.world_to_screen(
                point.x().absolute_value(),
                point.y().absolute_value(),
                ctx.rect,
            );
            if ctx.rect.contains(screen) {
                let rect = Rect::from_center_size(screen, egui::Vec2::splat(marker_size * 2.0));
                ctx.rect_filled(rect, 0.0, stroke_color);
                if ctx.highlight {
                    ctx.rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(stroke_width, stroke_color),
                        StrokeKind::Outside,
                    );
                }
            }
        }
    }
}

fn reference_draw_units(reference: &gdsr::Reference, ctx: &mut DrawContext) -> u64 {
    let mut visiting = HashSet::new();
    reference_draw_units_for_depth(
        reference,
        ctx.library,
        ctx.reference_depth,
        ctx.cell_complexity_cache,
        &mut visiting,
    )
}

struct CellDrawUnitsFrame {
    cell_name: String,
    depth: u32,
    next_element: usize,
    count: u64,
    pending_multiplier: Option<u64>,
}

enum ReferenceDrawUnits<'a> {
    Ready(u64),
    Cell {
        cell_name: &'a str,
        depth: u32,
        multiplier: u64,
    },
}

pub fn reference_draw_units_for_depth(
    reference: &gdsr::Reference,
    library: Option<&Library>,
    depth: u32,
    cache: &mut HashMap<(String, u32), u64>,
    visiting: &mut HashSet<String>,
) -> u64 {
    match reference_draw_units_parts(reference, depth) {
        ReferenceDrawUnits::Ready(count) => count,
        ReferenceDrawUnits::Cell {
            cell_name,
            depth,
            multiplier,
        } => multiplier.saturating_mul(
            library
                .and_then(|lib| cell_draw_units(cell_name, lib, depth, cache, visiting))
                .unwrap_or(1),
        ),
    }
}

fn reference_draw_units_parts(
    reference: &gdsr::Reference,
    mut depth: u32,
) -> ReferenceDrawUnits<'_> {
    let mut reference = reference;
    let mut count = 1_u64;
    loop {
        let grid_count = reference_grid_count(reference);
        if grid_count == 0 {
            return ReferenceDrawUnits::Ready(0);
        }
        count = count.saturating_mul(grid_count);
        if depth <= 1 {
            return ReferenceDrawUnits::Ready(count);
        }
        depth -= 1;

        if let Some(cell_name) = reference.instance().as_cell() {
            return ReferenceDrawUnits::Cell {
                cell_name,
                depth,
                multiplier: count,
            };
        }
        let Some(element) = reference.instance().as_element() else {
            return ReferenceDrawUnits::Ready(count);
        };
        let Element::Reference(inner) = element.as_ref().as_ref() else {
            return ReferenceDrawUnits::Ready(count);
        };
        reference = inner;
    }
}

fn cell_draw_units(
    cell_name: &str,
    library: &Library,
    depth: u32,
    cache: &mut HashMap<(String, u32), u64>,
    visiting: &mut HashSet<String>,
) -> Option<u64> {
    let key = (cell_name.to_owned(), depth);
    if let Some(count) = cache.get(&key) {
        return Some(*count);
    }
    library.get_cell(cell_name)?;
    if !visiting.insert(cell_name.to_owned()) {
        return Some(1);
    }

    let mut frames = vec![CellDrawUnitsFrame {
        cell_name: cell_name.to_string(),
        depth,
        next_element: 0,
        count: 0,
        pending_multiplier: None,
    }];
    let mut completed: Option<u64> = None;
    loop {
        if let Some(child_count) = completed.take()
            && let Some(frame) = frames.last_mut()
            && let Some(multiplier) = frame.pending_multiplier.take()
        {
            frame.count = frame
                .count
                .saturating_add(multiplier.saturating_mul(child_count));
        }

        let frame = frames.last_mut()?;
        let element = library
            .get_cell(&frame.cell_name)
            .and_then(|cell| cell.elements().get(frame.next_element));
        let Some(element) = element else {
            let frame = frames.pop()?;
            let count = frame.count.max(1);
            visiting.remove(&frame.cell_name);
            cache.insert((frame.cell_name, frame.depth), count);
            if frames.is_empty() {
                return Some(count);
            }
            completed = Some(count);
            continue;
        };
        frame.next_element += 1;

        let Element::Reference(reference) = element else {
            frame.count = frame.count.saturating_add(1);
            continue;
        };
        match reference_draw_units_parts(reference, frame.depth) {
            ReferenceDrawUnits::Ready(count) => {
                frame.count = frame.count.saturating_add(count);
            }
            ReferenceDrawUnits::Cell {
                cell_name,
                depth,
                multiplier,
            } => {
                let child_key = (cell_name.to_string(), depth);
                if let Some(&count) = cache.get(&child_key) {
                    frame.count = frame.count.saturating_add(multiplier.saturating_mul(count));
                } else if visiting.contains(cell_name) || library.get_cell(cell_name).is_none() {
                    frame.count = frame.count.saturating_add(multiplier);
                } else {
                    frame.pending_multiplier = Some(multiplier);
                    visiting.insert(cell_name.to_string());
                    frames.push(CellDrawUnitsFrame {
                        cell_name: cell_name.to_string(),
                        depth,
                        next_element: 0,
                        count: 0,
                        pending_multiplier: None,
                    });
                }
            }
        }
    }
}

fn should_draw_reference_bbox(
    reference: &gdsr::Reference,
    bbox: WorldBBox,
    ctx: &mut DrawContext,
) -> bool {
    let (sw, sh, _) = reference_screen_size(bbox, ctx);
    let grid_count = reference_grid_count(reference);
    let draw_unit_count = reference_draw_units(reference, ctx);
    should_collapse_reference(
        sw,
        sh,
        grid_count,
        draw_unit_count,
        ctx.reference_depth,
        ctx.show_ref_bbox,
    )
}

pub fn should_collapse_reference(
    width_px: f32,
    height_px: f32,
    grid_count: u64,
    draw_unit_count: u64,
    reference_depth: u32,
    force_bbox: bool,
) -> bool {
    if force_bbox || reference_depth <= 1 {
        return true;
    }

    if width_px < REF_LOAD_THRESHOLD_PX || height_px < REF_LOAD_THRESHOLD_PX {
        return true;
    }

    if grid_count > REF_MAX_GRID_EXPANSION {
        return true;
    }
    if draw_unit_count < REF_DENSE_GRID_MIN_INSTANCES {
        return false;
    }

    let area_px = width_px * height_px;
    area_px.is_finite() && area_px / draw_unit_count as f32 <= REF_LOAD_MIN_AVG_INSTANCE_AREA_PX
}

fn draw_ref_bbox(reference: &gdsr::Reference, bbox: WorldBBox, ctx: &mut DrawContext) {
    if !bbox.overlaps(ctx.visible) {
        return;
    }

    let (sw, sh, screen_rect) = reference_screen_size(bbox, ctx);
    if sw < 1.0 && sh < 1.0 {
        return;
    }

    let stroke_color = Color32::from_rgb(180, 180, 180);
    ctx.rect_stroke(
        screen_rect,
        0.0,
        Stroke::new(1.0_f32, stroke_color),
        StrokeKind::Outside,
    );

    if let Some(name) = reference.instance().as_cell() {
        if sw >= 40.0 && sh >= 20.0 {
            let char_count = name.len().max(1) as f32;
            let fit_w = sw * 0.9 / (char_count * 0.6);
            let fit_h = sh * 0.4;
            let font_size = fit_w.min(fit_h).min(48.0);
            if font_size >= 8.0 {
                let center = screen_rect.center();
                ctx.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    name,
                    FontId::monospace(font_size),
                    stroke_color,
                );
            }
        }
    }
}

enum ReferenceDrawAction {
    Element(Element, u32),
    LeaveCell(String),
}

fn draw_reference_iteratively(reference: &gdsr::Reference, ctx: &mut DrawContext) {
    let previous_depth = ctx.reference_depth;
    let previous_stack_len = ctx.reference_stack.len();
    let mut active_cells: HashSet<String> = ctx.reference_stack.iter().cloned().collect();
    let mut pending = vec![ReferenceDrawAction::Element(
        Element::Reference(reference.clone()),
        previous_depth,
    )];

    while let Some(action) = pending.pop() {
        let (element, depth) = match action {
            ReferenceDrawAction::Element(element, depth) => (element, depth),
            ReferenceDrawAction::LeaveCell(cell_name) => {
                active_cells.remove(&cell_name);
                ctx.reference_stack.pop();
                continue;
            }
        };
        ctx.reference_depth = depth;
        let Element::Reference(reference) = element else {
            element.draw(ctx);
            continue;
        };
        let Some(bbox) = reference_instance_bbox(&reference, ctx.library, ctx.cell_bbox_cache)
        else {
            continue;
        };
        if !bbox.overlaps(ctx.visible) {
            continue;
        }
        if should_draw_reference_bbox(&reference, bbox, ctx) {
            draw_ref_bbox(&reference, bbox, ctx);
            continue;
        }

        let child_depth = depth.saturating_sub(1);
        if let Some(element) = reference.instance().as_element() {
            pending.extend(
                reference
                    .get_elements_in_grid(element)
                    .into_iter()
                    .rev()
                    .map(|element| ReferenceDrawAction::Element(element, child_depth)),
            );
        } else if let Some(cell_name) = reference.instance().as_cell() {
            if !active_cells.insert(cell_name.clone()) {
                draw_ref_bbox(&reference, bbox, ctx);
                continue;
            }
            ctx.reference_stack.push(cell_name.clone());
            pending.push(ReferenceDrawAction::LeaveCell(cell_name.clone()));
            if let Some(cell) = ctx.library.and_then(|library| library.get_cell(cell_name)) {
                let mut elements = Vec::new();
                for element in cell.iter_elements() {
                    elements.extend(reference.get_elements_in_grid(element));
                }
                pending.extend(
                    elements
                        .into_iter()
                        .rev()
                        .map(|element| ReferenceDrawAction::Element(element, child_depth)),
                );
            }
        }
    }

    ctx.reference_stack.truncate(previous_stack_len);
    ctx.reference_depth = previous_depth;
}

impl Drawable for gdsr::Reference {
    fn layer_keys(&self) -> Vec<(Layer, DataType)> {
        let mut reference = self;
        loop {
            let Some(element) = reference.instance().as_element() else {
                return Vec::new();
            };
            match element.as_ref().as_ref() {
                Element::Reference(inner) => reference = inner,
                element => return element.layer_keys(),
            }
        }
    }

    fn world_bbox(&self) -> Option<WorldBBox> {
        reference_instance_bbox(self, None, &mut HashMap::new())
    }

    fn hit_test(&self, wx: f64, wy: f64, zoom: f64) -> bool {
        let Some(element) = self.instance().as_element() else {
            return false;
        };
        let mut pending = self.get_elements_in_grid(element);
        while let Some(element) = pending.pop() {
            if let Element::Reference(reference) = element {
                if let Some(inner) = reference.instance().as_element() {
                    pending.extend(reference.get_elements_in_grid(inner));
                }
            } else if element.hit_test(wx, wy, zoom) {
                return true;
            }
        }
        false
    }

    fn draw(&self, ctx: &mut DrawContext) {
        draw_reference_iteratively(self, ctx);
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

/// Re-renders a single element with highlight styling (brighter fill, bolder stroke)
/// directly onto the painter, bypassing the render cache.
pub fn draw_highlight(
    element: &Element,
    viewport: &crate::viewport::Viewport,
    painter: &egui::Painter,
    rect: Rect,
    layer_state: &mut LayerState,
    library: Option<&Library>,
    tessellation_cache: &mut HashMap<u32, Vec<usize>>,
) {
    let visible = viewport.visible_world_rect(rect);
    let mut layer_meshes = HashMap::new();
    let mut extra_shapes = Vec::new();
    let mut screen_pts_buf = Vec::new();
    let mut cell_bbox_cache = HashMap::new();
    let mut cell_complexity_cache = HashMap::new();
    let mut ctx = DrawContext {
        painter,
        layer_meshes: &mut layer_meshes,
        extra_shapes: &mut extra_shapes,
        viewport,
        rect,
        visible: &visible,
        layer_state,
        library,
        current_element_idx: None,
        tessellation_cache,
        screen_pts_buf: &mut screen_pts_buf,
        highlight: true,
        show_ref_bbox: false,
        reference_depth: u32::MAX,
        reference_stack: Vec::new(),
        cell_bbox_cache: &mut cell_bbox_cache,
        cell_complexity_cache: &mut cell_complexity_cache,
    };
    element.draw(&mut ctx);
    for (_, mesh) in layer_meshes {
        painter.add(Shape::mesh(mesh));
    }
    painter.extend(extra_shapes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::helpers::*;

    const SCALE: f64 = 1e-9;
    const ZOOM: f64 = 1e9;

    fn deeply_named_library(depth: usize) -> Library {
        let mut library = Library::new("deep");
        let leaf_name = format!("cell_{:05}", depth - 1);
        let mut leaf = gdsr::Cell::new(&leaf_name);
        leaf.add(polygon(vec![(0, 0), (100, 0), (100, 100)], 3, 1));
        library.add_cell(leaf);
        for index in (0..depth - 1).rev() {
            let cell_name = format!("cell_{index:05}");
            let mut cell = gdsr::Cell::new(&cell_name);
            cell.add(gdsr::Reference::new(format!("cell_{:05}", index + 1)));
            library.add_cell(cell);
        }
        library
    }

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
    fn gds_box_layer_keys() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(100, 100),
            Layer::new(3),
            DataType::new(5),
        );
        assert_eq!(b.layer_keys(), vec![(Layer::new(3), DataType::new(5))]);
    }

    #[test]
    fn gds_box_world_bbox() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(10, 20),
            gdsr::Point::default_integer(30, 40),
            Layer::new(1),
            DataType::new(0),
        );
        let bbox = b.world_bbox().expect("should have bbox");
        assert!((bbox.min_x - 10.0 * SCALE).abs() < 1e-15);
        assert!((bbox.min_y - 20.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_x - 30.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_y - 40.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn node_layer_keys() {
        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(0, 0)],
            Layer::new(7),
            DataType::new(2),
        );
        assert_eq!(n.layer_keys(), vec![(Layer::new(7), DataType::new(2))]);
    }

    #[test]
    fn node_world_bbox() {
        let n = gdsr::Node::new(
            vec![
                gdsr::Point::default_integer(10, 20),
                gdsr::Point::default_integer(30, 40),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let bbox = n.world_bbox().expect("should have bbox");
        assert!((bbox.min_x - 10.0 * SCALE).abs() < 1e-15);
        assert!((bbox.min_y - 20.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_x - 30.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_y - 40.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn gds_box_hit_near_edge() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(100, 100),
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Box(b);
        assert!(el.hit_test(0.0, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn node_hit_multiple_points() {
        let n = gdsr::Node::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 100),
                gdsr::Point::default_integer(200, 0),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let el = Element::Node(n);
        assert!(el.hit_test(0.0, 0.0, ZOOM));
        assert!(el.hit_test(100.0 * SCALE, 100.0 * SCALE, ZOOM));
        assert!(el.hit_test(200.0 * SCALE, 0.0, ZOOM));
        assert!(!el.hit_test(500.0 * SCALE, 500.0 * SCALE, ZOOM));
    }

    #[test]
    fn reference_with_element_layer_keys() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(10, 0),
                gdsr::Point::default_integer(10, 10),
            ],
            Layer::new(3),
            DataType::new(1),
        );
        let r = gdsr::Reference::new(poly);
        assert_eq!(r.layer_keys(), vec![(Layer::new(3), DataType::new(1))]);
    }

    #[test]
    fn reference_empty_layer_keys() {
        let r = gdsr::Reference::default();
        assert!(r.layer_keys().is_empty());
    }

    #[test]
    fn reference_empty_world_bbox() {
        let r = gdsr::Reference::default();
        assert!(r.world_bbox().is_none());
    }

    #[test]
    fn reference_load_collapses_at_depth_limit() {
        assert!(should_collapse_reference(400.0, 400.0, 1, 1, 1, false));
    }

    #[test]
    fn reference_load_collapses_tiny_references() {
        assert!(should_collapse_reference(12.0, 20.0, 1, 1, 2, false));
    }

    #[test]
    fn reference_load_collapses_skinny_references() {
        assert!(should_collapse_reference(4000.0, 4.0, 1, 1, 2, false));
    }

    #[test]
    fn reference_load_collapses_huge_grids() {
        assert!(should_collapse_reference(
            10_000.0,
            10_000.0,
            REF_MAX_GRID_EXPANSION + 1,
            REF_MAX_GRID_EXPANSION + 1,
            2,
            false,
        ));
    }

    #[test]
    fn reference_load_draws_sparse_large_references() {
        assert!(!should_collapse_reference(400.0, 400.0, 1, 100, 2, false));
    }

    #[test]
    fn reference_load_collapses_dense_screen_area() {
        assert!(should_collapse_reference(400.0, 200.0, 1, 10_000, 2, false,));
    }

    #[test]
    fn bbox_fallback_handles_skinny_geometry() {
        assert!(should_use_bbox_fallback(4000.0, 4.0));
        assert!(should_use_bbox_fallback(4.0, 4000.0));
        assert!(!should_use_bbox_fallback(40.0, 40.0));
    }

    #[test]
    fn reference_draw_units_include_cell_contents_and_grid() {
        let mut leaf = gdsr::Cell::new("leaf");
        leaf.add(polygon(vec![(0, 0), (10, 0), (10, 10)], 1, 0));
        leaf.add(polygon(vec![(20, 0), (30, 0), (30, 10)], 1, 0));
        leaf.add(polygon(vec![(40, 0), (50, 0), (50, 10)], 1, 0));

        let mut library = gdsr::Library::new("lib");
        library.add_cell(leaf);

        let reference = gdsr::Reference::new("leaf").with_grid(
            gdsr::Grid::default()
                .with_columns(2)
                .with_rows(3)
                .with_spacing_x(Some(gdsr::Point::default_integer(20, 0)))
                .with_spacing_y(Some(gdsr::Point::default_integer(0, 20))),
        );
        let mut cache = HashMap::new();
        let mut visiting = HashSet::new();

        assert_eq!(
            reference_draw_units_for_depth(
                &reference,
                Some(&library),
                2,
                &mut cache,
                &mut visiting,
            ),
            18,
        );
    }

    #[test]
    fn reference_with_element_world_bbox() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 0),
                gdsr::Point::default_integer(100, 100),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let r = gdsr::Reference::new(poly);
        let bbox = r.world_bbox().expect("should have bbox");
        assert!((bbox.min_x - 0.0).abs() < 1e-15);
        assert!((bbox.max_x - 100.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn cell_world_bbox_includes_referenced_cells() {
        let mut leaf = gdsr::Cell::new("leaf");
        leaf.add(polygon(vec![(10, 20), (110, 20), (110, 220)], 1, 0));

        let mut top = gdsr::Cell::new("top");
        top.add(gdsr::Reference::new("leaf"));

        let mut library = gdsr::Library::new("lib");
        library.add_cell(leaf);
        library.add_cell(top);

        let bbox = cell_world_bbox("top", &library).expect("should have bbox");
        assert!((bbox.min_x - 10.0 * SCALE).abs() < 1e-15);
        assert!((bbox.min_y - 20.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_x - 110.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_y - 220.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn cell_world_bbox_applies_reference_origin() {
        let mut leaf = gdsr::Cell::new("leaf");
        leaf.add(polygon(vec![(10, 20), (110, 20), (110, 220)], 1, 0));

        let mut top = gdsr::Cell::new("top");
        top.add(
            gdsr::Reference::new("leaf").with_grid(
                gdsr::Grid::default().with_origin(gdsr::Point::default_integer(100, 200)),
            ),
        );

        let mut library = gdsr::Library::new("lib");
        library.add_cell(leaf);
        library.add_cell(top);

        let bbox = cell_world_bbox("top", &library).expect("should have bbox");
        assert!((bbox.min_x - 110.0 * SCALE).abs() < 1e-15);
        assert!((bbox.min_y - 220.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_x - 210.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_y - 420.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn cell_world_bbox_handles_reference_cycles() {
        let mut a = gdsr::Cell::new("a");
        a.add(gdsr::Reference::new("b"));

        let mut b = gdsr::Cell::new("b");
        b.add(gdsr::Reference::new("a"));

        let mut library = gdsr::Library::new("lib");
        library.add_cell(a);
        library.add_cell(b);

        assert!(cell_world_bbox("a", &library).is_none());
    }

    #[test]
    fn cyclic_cell_world_bbox_matches_conservative_core_report() {
        let mut a = gdsr::Cell::new("a");
        a.add(polygon(vec![(0, 0), (1, 0), (1, 1), (0, 1)], 1, 0));
        a.add(gdsr::Reference::new("b"));
        let mut b = gdsr::Cell::new("b");
        b.add(polygon(vec![(100, 0), (101, 0), (101, 1), (100, 1)], 2, 0));
        b.add(gdsr::Reference::new("a"));
        let mut top = gdsr::Cell::new("top");
        top.add(gdsr::Reference::new("a"));
        let mut library = gdsr::Library::new("lib");
        library.add_cell(top);
        library.add_cell(b);
        library.add_cell(a);

        let viewer_bounds =
            cell_world_bbox("top", &library).expect("viewer should retain geometry");
        let core_bounds = library
            .hierarchy_bounds("top")
            .expect("core should report cyclic bounds")
            .root_bounds();

        assert_eq!(Some(viewer_bounds), core_bounds);
        assert_eq!(
            viewer_bounds,
            WorldBBox::new(0.0, 0.0, 101.0 * SCALE, SCALE)
        );
    }

    #[test]
    fn signed_path_world_bbox_matches_core_conservative_bounds() {
        let path = gdsr::Path::new(
            [
                gdsr::Point::float(0.0, 0.0, 1.0),
                gdsr::Point::float(10.0, 0.0, 1.0),
            ],
            Layer::new(1),
            DataType::new(0),
            None,
            Some(gdsr::Unit::float(-4.0, 1.0)),
            Some(gdsr::Unit::float(-100.0, 1.0)),
            Some(gdsr::Unit::float(-100.0, 1.0)),
        );
        let viewer_bounds = path.world_bbox().expect("path should have bounds");
        let mut top = gdsr::Cell::new("top");
        top.add(path);
        let mut library = gdsr::Library::new("lib");
        library.add_cell(top);
        let core_bounds = library
            .hierarchy_bounds("top")
            .expect("core should report path bounds")
            .root_bounds();

        assert_eq!(Some(viewer_bounds), core_bounds);
        assert_eq!(viewer_bounds, WorldBBox::new(-2.0, -2.0, 12.0, 2.0));
    }

    #[test]
    fn cell_world_bbox_matches_partial_core_report() {
        let mut top = gdsr::Cell::new("top");
        top.add(polygon(vec![(10, 20), (110, 20), (110, 220)], 1, 0));
        top.add(gdsr::Reference::new("missing"));
        let mut library = gdsr::Library::new("lib");
        library.add_cell(top);

        let viewer_bounds = cell_world_bbox("top", &library).expect("valid geometry should remain");
        let core_report = library
            .hierarchy_bounds("top")
            .expect("known root should produce a report");
        assert_eq!(Some(viewer_bounds), core_report.root_bounds());
        assert_eq!(core_report.diagnostics().len(), 1);
    }

    #[test]
    fn reference_instance_bbox_resolves_inline_named_reference_chain() {
        let mut leaf = gdsr::Cell::new("leaf");
        leaf.add(polygon(vec![(10, 20), (110, 20), (110, 220)], 1, 0));
        let nested = gdsr::Reference::new("leaf")
            .with_grid(gdsr::Grid::default().with_origin(gdsr::Point::default_integer(100, 200)));
        let reference = gdsr::Reference::new(nested)
            .with_grid(gdsr::Grid::default().with_origin(gdsr::Point::default_integer(300, 400)));
        let mut library = gdsr::Library::new("lib");
        library.add_cell(leaf);

        let bbox = reference_instance_bbox(&reference, Some(&library), &mut HashMap::new())
            .expect("inline named hierarchy should resolve");
        assert!((bbox.min_x - 410.0 * SCALE).abs() < 1e-15);
        assert!((bbox.min_y - 620.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_x - 510.0 * SCALE).abs() < 1e-15);
        assert!((bbox.max_y - 820.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn reference_world_bbox_handles_ten_thousand_inline_references() {
        let polygon = gdsr::Polygon::new(
            [
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 0),
                gdsr::Point::default_integer(100, 100),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let mut reference = gdsr::Reference::new(polygon);
        for _ in 1..10_000 {
            reference = gdsr::Reference::new(reference);
        }

        let bbox = reference
            .world_bbox()
            .expect("inline bounds should resolve");
        assert!((bbox.min_x - 0.0).abs() < 1e-15);
        assert!((bbox.max_x - 100.0 * SCALE).abs() < 1e-15);
    }

    #[test]
    fn deep_inline_layer_keys_and_hit_test_are_iterative() {
        let polygon = gdsr::Polygon::new(
            [
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 0),
                gdsr::Point::default_integer(100, 100),
            ],
            Layer::new(3),
            DataType::new(1),
        );
        let mut reference = gdsr::Reference::new(polygon);
        for _ in 1..10_000 {
            reference = gdsr::Reference::new(reference);
        }

        assert_eq!(
            reference.layer_keys(),
            vec![(Layer::new(3), DataType::new(1))]
        );
        assert!(reference.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn deep_named_world_bbox_complexity_and_draw_are_iterative() {
        const DEPTH: usize = 10_000;
        let library = deeply_named_library(DEPTH);
        let bbox =
            cell_world_bbox("cell_00000", &library).expect("named hierarchy should have bounds");
        assert!((bbox.max_x - 100.0 * SCALE).abs() < 1e-15);

        let reference = gdsr::Reference::new("cell_00000");
        let mut cache = HashMap::new();
        let mut visiting = HashSet::new();
        assert_eq!(
            reference_draw_units_for_depth(
                &reference,
                Some(&library),
                u32::try_from(DEPTH + 1).expect("depth should fit"),
                &mut cache,
                &mut visiting,
            ),
            1,
        );

        let egui_context = egui::Context::default();
        let painter = egui_context.layer_painter(egui::LayerId::background());
        let viewport = Viewport {
            zoom: 1.0e12,
            ..Viewport::default()
        };
        let rect = Rect::from_min_size(Pos2::ZERO, egui::Vec2::splat(1_000.0));
        draw_highlight(
            &Element::Reference(reference),
            &viewport,
            &painter,
            rect,
            &mut LayerState::default(),
            Some(&library),
            &mut HashMap::new(),
        );
    }

    #[test]
    fn reference_with_element_hit_test() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 0),
                gdsr::Point::default_integer(100, 100),
                gdsr::Point::default_integer(0, 100),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let r = gdsr::Reference::new(poly);
        assert!(r.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
        assert!(!r.hit_test(500.0 * SCALE, 500.0 * SCALE, ZOOM));
    }

    #[test]
    fn element_dispatch_layer_keys() {
        let el = polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 3);
        assert_eq!(el.layer_keys(), vec![(Layer::new(5), DataType::new(3))]);

        let el = path(vec![(0, 0), (10, 0)], 7, 2, None);
        assert_eq!(el.layer_keys(), vec![(Layer::new(7), DataType::new(2))]);

        let el = text("t", 0, 0, 4);
        assert_eq!(el.layer_keys(), vec![(Layer::new(4), DataType::new(0))]);

        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(10, 10),
            Layer::new(8),
            DataType::new(1),
        );
        assert_eq!(
            Element::Box(b).layer_keys(),
            vec![(Layer::new(8), DataType::new(1))]
        );

        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(0, 0)],
            Layer::new(9),
            DataType::new(4),
        );
        assert_eq!(
            Element::Node(n).layer_keys(),
            vec![(Layer::new(9), DataType::new(4))]
        );
    }

    #[test]
    fn element_dispatch_world_bbox() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        assert!(el.world_bbox().is_some());

        let el = path(vec![(0, 0), (100, 100)], 1, 0, None);
        assert!(el.world_bbox().is_some());

        let el = text("hi", 50, 50, 1);
        assert!(el.world_bbox().is_some());

        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(10, 10),
            Layer::new(1),
            DataType::new(0),
        );
        assert!(Element::Box(b).world_bbox().is_some());

        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(0, 0)],
            Layer::new(1),
            DataType::new(0),
        );
        assert!(Element::Node(n).world_bbox().is_some());

        let el = reference();
        assert!(el.world_bbox().is_none());
    }

    #[test]
    fn element_dispatch_hit_test() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(0, 0),
            gdsr::Point::default_integer(100, 100),
            Layer::new(1),
            DataType::new(0),
        );
        assert!(Element::Box(b).hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));

        let n = gdsr::Node::new(
            vec![gdsr::Point::default_integer(50, 50)],
            Layer::new(1),
            DataType::new(0),
        );
        assert!(Element::Node(n).hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));

        assert!(!reference().hit_test(0.0, 0.0, ZOOM));
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
    fn polygon_hit_degenerate_single_point() {
        let el = polygon(vec![(50, 50), (50, 50), (50, 50)], 1, 0);
        // Exactly at the vertex — edge proximity returns true
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
        // Far away should miss
        assert!(!el.hit_test(500.0 * SCALE, 500.0 * SCALE, ZOOM));
    }

    #[test]
    fn polygon_hit_collinear_points() {
        let el = polygon(vec![(0, 0), (50, 0), (100, 0)], 1, 0);
        assert!(el.hit_test(50.0 * SCALE, 0.0, ZOOM));
    }

    #[test]
    fn polygon_hit_extreme_zoom_far() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, 1e-3));
    }

    #[test]
    fn polygon_hit_extreme_zoom_close() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(el.hit_test(50.0 * SCALE, 50.0 * SCALE, 1e15));
    }

    #[test]
    fn polygon_hit_at_edge() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(el.hit_test(0.0, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn polygon_hit_at_vertex() {
        let el = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        assert!(el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn polygon_with_two_points_no_panic() {
        let el = polygon(vec![(0, 0), (100, 0)], 1, 0);
        // Near the line segment — edge proximity hits
        assert!(el.hit_test(50.0 * SCALE, 0.0, ZOOM));
        // Far away should miss
        assert!(!el.hit_test(50.0 * SCALE, 500.0 * SCALE, ZOOM));
    }

    #[test]
    fn path_hit_zero_width() {
        let el = path(vec![(0, 0), (100, 0)], 1, 0, None);
        assert!(el.hit_test(50.0 * SCALE, 0.0, ZOOM));
    }

    #[test]
    fn path_hit_single_point_no_panic() {
        let el = path(vec![(50, 50)], 1, 0, Some(10));
        assert!(!el.hit_test(50.0 * SCALE, 50.0 * SCALE, ZOOM));
    }

    #[test]
    fn path_hit_empty_no_panic() {
        let el = path(vec![], 1, 0, Some(10));
        assert!(!el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn node_hit_empty_points_no_panic() {
        let n = gdsr::Node::new(vec![], Layer::new(1), DataType::new(0));
        let el = Element::Node(n);
        assert!(!el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn text_hit_extreme_zoom() {
        let el = text("hello", 500, 600, 1);
        assert!(el.hit_test(500.0 * SCALE, 600.0 * SCALE, 1e-3));
    }

    #[test]
    fn reference_empty_hit_test() {
        let el = reference();
        assert!(!el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn bbox_overlaps_identical() {
        let a = WorldBBox::new(1.0, 2.0, 3.0, 4.0);
        assert!(a.overlaps(&a));
    }

    #[test]
    fn bbox_merge_with_itself() {
        let a = WorldBBox::new(1.0, 2.0, 3.0, 4.0);
        let merged = a.merge(&a);
        assert!((merged.min_x - a.min_x).abs() < 1e-15);
        assert!((merged.max_x - a.max_x).abs() < 1e-15);
    }

    #[test]
    fn bbox_merge_disjoint() {
        let a = WorldBBox::new(0.0, 0.0, 1.0, 1.0);
        let b = WorldBBox::new(10.0, 10.0, 11.0, 11.0);
        let merged = a.merge(&b);
        assert!((merged.min_x - 0.0).abs() < 1e-15);
        assert!((merged.min_y - 0.0).abs() < 1e-15);
        assert!((merged.max_x - 11.0).abs() < 1e-15);
        assert!((merged.max_y - 11.0).abs() < 1e-15);
    }

    #[test]
    fn point_in_polygon_degenerate_less_than_3() {
        assert!(!point_in_polygon(0.0, 0.0, &[]));
        assert!(!point_in_polygon(0.0, 0.0, &[(0.0, 0.0)]));
        assert!(!point_in_polygon(0.0, 0.0, &[(0.0, 0.0), (1.0, 0.0)]));
    }

    #[test]
    fn point_in_polygon_concave() {
        let concave = [
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (5.0, 5.0),
            (0.0, 10.0),
        ];
        assert!(point_in_polygon(2.0, 2.0, &concave));
        assert!(!point_in_polygon(5.0, 8.0, &concave));
    }

    #[test]
    fn point_to_segment_dist_zero_length_segment() {
        let d = point_to_segment_dist_sq(3.0, 4.0, 0.0, 0.0, 0.0, 0.0);
        assert!((d - 25.0).abs() < 1e-10);
    }

    #[test]
    fn point_to_segment_dist_at_endpoint() {
        let d = point_to_segment_dist_sq(0.0, 0.0, 0.0, 0.0, 10.0, 0.0);
        assert!(d < 1e-20);
    }

    #[test]
    fn point_to_segment_dist_beyond_endpoint() {
        let d = point_to_segment_dist_sq(15.0, 0.0, 0.0, 0.0, 10.0, 0.0);
        assert!((d - 25.0).abs() < 1e-10);
    }

    #[test]
    fn stroke_polyline_empty() {
        let stroke = Stroke::new(2.0_f32, Color32::WHITE);
        let mesh = stroke_polyline_to_mesh(&[], stroke, false);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn stroke_polyline_single_point() {
        let stroke = Stroke::new(2.0_f32, Color32::WHITE);
        let mesh = stroke_polyline_to_mesh(&[Pos2::new(5.0, 5.0)], stroke, false);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.indices.is_empty());
    }

    #[test]
    fn stroke_polyline_coincident_points() {
        let stroke = Stroke::new(2.0_f32, Color32::WHITE);
        let pts = vec![
            Pos2::new(5.0, 5.0),
            Pos2::new(5.0, 5.0),
            Pos2::new(5.0, 5.0),
        ];
        let mesh = stroke_polyline_to_mesh(&pts, stroke, false);
        assert!(
            mesh.vertices.is_empty(),
            "coincident points should produce no edges"
        );
    }

    #[test]
    fn stroke_polyline_closed_vs_open() {
        let stroke = Stroke::new(2.0_f32, Color32::WHITE);
        let pts = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
        ];
        let open = stroke_polyline_to_mesh(&pts, stroke, false);
        let closed = stroke_polyline_to_mesh(&pts, stroke, true);
        assert!(
            closed.vertices.len() > open.vertices.len(),
            "closed should have more vertices from closing edge"
        );
    }

    #[test]
    fn polygon_bbox_with_extreme_coords() {
        let max = i32::MAX / 2;
        let el = polygon(vec![(0, 0), (max, 0), (max, max), (0, max)], 1, 0);
        let bbox = el.world_bbox().expect("should have bbox");
        assert!(bbox.min_x.is_finite());
        assert!(bbox.max_x.is_finite());
        assert!(bbox.min_y.is_finite());
        assert!(bbox.max_y.is_finite());
        assert!(bbox.max_x > bbox.min_x);
        assert!(bbox.max_y > bbox.min_y);
    }

    #[test]
    fn path_bbox_with_extreme_coords() {
        let max = i32::MAX / 2;
        let el = path(vec![(0, 0), (max, max)], 1, 0, Some(100));
        let bbox = el.world_bbox().expect("should have bbox");
        assert!(bbox.min_x.is_finite());
        assert!(bbox.max_x.is_finite());
    }

    #[test]
    fn polygon_many_vertices_no_panic() {
        let pts: Vec<(i32, i32)> = (0..1000)
            .map(|i| {
                let angle = f64::from(i) * 2.0 * std::f64::consts::PI / 1000.0;
                let x = (angle.cos() * 1000.0) as i32;
                let y = (angle.sin() * 1000.0) as i32;
                (x, y)
            })
            .collect();
        let el = polygon(pts, 1, 0);
        let bbox = el.world_bbox().expect("should have bbox");
        assert!(bbox.max_x > bbox.min_x);
        assert!(el.hit_test(0.0, 0.0, ZOOM));
    }

    #[test]
    fn point_to_segment_dist_perpendicular() {
        let d = point_to_segment_dist_sq(5.0, 3.0, 0.0, 0.0, 10.0, 0.0);
        assert!((d - 9.0).abs() < 1e-10);
    }
}
