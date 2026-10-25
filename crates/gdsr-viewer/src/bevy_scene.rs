use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use bevy::math::{DAffine2, DMat2, DVec2};
use gdsr::{DataType, Element, Grid, Instance, Layer, Library, Point};

use crate::drawable::{Drawable, WorldBBox, cell_world_bboxes, should_collapse_reference};

const MAX_DETAIL_CELLS: usize = 20_000;
const MAX_SCENE_OBJECTS: usize = 20_000;
const MAX_TEXT_INSTANCES: usize = 10_000;
const MAX_LOAD_PROXIES: usize = 100_000;
const MAX_BATCH_ELEMENTS: u32 = 256;
const DENSE_BATCH_MIN_ELEMENTS: u32 = 128;
const MIN_AVERAGE_ELEMENT_AREA_PX: f32 = 16.0;
const MIN_FIXED_CONTENT_PX: f32 = 24.0;

#[derive(Debug, PartialEq, Eq)]
pub enum SceneBuildError {
    TooManyVertices { layer: (Layer, DataType) },
}

#[derive(Default)]
struct GeometryBuilder {
    positions: Vec<DVec2>,
    triangle_indices: Vec<u32>,
    line_indices: Vec<u32>,
    element_count: u32,
}

pub struct PreparedGeometry {
    pub layer: (Layer, DataType),
    pub positions: Vec<DVec2>,
    pub triangle_indices: Vec<u32>,
    pub line_indices: Vec<u32>,
    pub origin: DVec2,
    bbox: WorldBBox,
    element_count: u32,
}

#[derive(Clone)]
struct PreparedText {
    value: Arc<str>,
    origin: DVec2,
    layer: (Layer, DataType),
}

#[derive(Clone)]
struct PreparedNode {
    points: Vec<DVec2>,
    layer: (Layer, DataType),
}

enum PreparedSource {
    Cell(String),
    Geometry(usize),
    Text(PreparedText),
    Node(PreparedNode),
}

struct PreparedReference {
    grids: Vec<Grid>,
    source: PreparedSource,
    source_bbox: Option<WorldBBox>,
    suffix_bounds: Vec<Option<WorldBBox>>,
    suffix_grid_counts: Vec<u64>,
    label: Option<Arc<str>>,
}

struct GridTraversalFrame {
    grid_index: usize,
    transform: DAffine2,
    column: u32,
    row: u32,
    entered: bool,
}

struct ReferenceTraversal<'a> {
    reference: &'a PreparedReference,
    parent_transform: DAffine2,
    depth: u32,
    stack: Vec<GridTraversalFrame>,
}

enum PlanAction<'a> {
    Cell {
        cell_name: String,
        transform: DAffine2,
        depth: u32,
    },
    CellReferences {
        cell_name: String,
        transform: DAffine2,
        depth: u32,
        next_reference: usize,
        world_bbox: Option<WorldBBox>,
    },
    Reference(ReferenceTraversal<'a>),
}

struct CellComplexityFrame {
    cell_name: String,
    depth: u32,
    next_reference: usize,
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

struct PreparedCell {
    geometry_ids: Vec<usize>,
    direct_draw_units: u64,
    texts: Vec<PreparedText>,
    nodes: Vec<PreparedNode>,
    references: Vec<PreparedReference>,
    bbox: Option<WorldBBox>,
}

pub struct PreparedLibrary {
    cells: HashMap<String, PreparedCell>,
    pub geometries: Vec<PreparedGeometry>,
}

pub struct SceneOptions<'a> {
    pub visible: &'a WorldBBox,
    pub zoom: f64,
    pub depth: u32,
    pub hidden_layers: &'a HashSet<(Layer, DataType)>,
}

pub struct GeometryInstance {
    pub geometry_id: usize,
    pub transform: DAffine2,
}

pub struct LoadProxy {
    pub bbox: WorldBBox,
    pub label: Option<Arc<str>>,
}

pub struct TextInstance {
    pub value: Arc<str>,
    pub position: DVec2,
    pub layer: (Layer, DataType),
}

#[derive(Default)]
pub struct SceneStats {
    pub detail_cells: usize,
    pub detail_objects: usize,
    pub culled_references: usize,
    pub skipped_subpixel_references: usize,
    pub culled_geometry_batches: usize,
    pub skipped_subpixel_geometry_batches: usize,
}

#[derive(Default)]
pub struct ScenePlan {
    pub geometry_instances: Vec<GeometryInstance>,
    pub load_proxies: Vec<LoadProxy>,
    pub texts: Vec<TextInstance>,
    pub node_points: BTreeMap<(Layer, DataType), Vec<DVec2>>,
    pub stats: SceneStats,
}

impl GeometryBuilder {
    fn push_polygon(
        &mut self,
        layer: (Layer, DataType),
        points: &[Point],
    ) -> Result<(), SceneBuildError> {
        let mut coordinates: Vec<DVec2> = points.iter().map(point_position).collect();
        if coordinates.len() >= 2 && coordinates.first() == coordinates.last() {
            coordinates.pop();
        }
        if coordinates.len() < 3 {
            return Ok(());
        }

        let mut flat = Vec::with_capacity(coordinates.len() * 2);
        for point in &coordinates {
            flat.extend([point.x, point.y]);
        }
        let Ok(triangles) = earcutr::earcut(&flat, &[], 2) else {
            return Ok(());
        };
        if triangles.is_empty() {
            return Ok(());
        }

        let base = u32::try_from(self.positions.len())
            .map_err(|_| SceneBuildError::TooManyVertices { layer })?;
        let vertex_count = u32::try_from(coordinates.len())
            .map_err(|_| SceneBuildError::TooManyVertices { layer })?;
        self.positions.extend(coordinates);
        for triangle in triangles {
            let index =
                u32::try_from(triangle).map_err(|_| SceneBuildError::TooManyVertices { layer })?;
            self.triangle_indices.push(base + index);
        }
        for index in 0..vertex_count {
            self.line_indices.push(base + index);
            self.line_indices.push(base + ((index + 1) % vertex_count));
        }
        self.element_count = self.element_count.saturating_add(1);
        Ok(())
    }

    fn finish(self, layer: (Layer, DataType)) -> Option<PreparedGeometry> {
        if self.positions.is_empty() || self.triangle_indices.is_empty() {
            return None;
        }
        let mut min = DVec2::splat(f64::INFINITY);
        let mut max = DVec2::splat(f64::NEG_INFINITY);
        for position in &self.positions {
            min = min.min(*position);
            max = max.max(*position);
        }

        Some(PreparedGeometry {
            layer,
            positions: self.positions,
            triangle_indices: self.triangle_indices,
            line_indices: self.line_indices,
            origin: (min + max) * 0.5,
            bbox: WorldBBox::new(min.x, min.y, max.x, max.y),
            element_count: self.element_count,
        })
    }
}

fn push_polygon_batch(
    builders: &mut BTreeMap<(Layer, DataType), Vec<GeometryBuilder>>,
    layer: (Layer, DataType),
    points: &[Point],
) -> Result<(), SceneBuildError> {
    let layer_builders = builders.entry(layer).or_default();
    if layer_builders
        .last()
        .is_none_or(|builder| builder.element_count >= MAX_BATCH_ELEMENTS)
    {
        layer_builders.push(GeometryBuilder::default());
    }
    if let Some(builder) = layer_builders.last_mut() {
        builder.push_polygon(layer, points)?;
    }
    Ok(())
}

impl PreparedLibrary {
    pub fn build(library: &Library, root_cell: &str) -> Result<Self, SceneBuildError> {
        let cell_bboxes = cell_world_bboxes(library, root_cell);
        let cell_names = reachable_cell_names(library, root_cell);
        let mut prepared = Self {
            cells: HashMap::with_capacity(cell_names.len()),
            geometries: Vec::new(),
        };

        for cell_name in cell_names {
            let Some(cell) = library.get_cell(&cell_name) else {
                continue;
            };
            let mut builders = BTreeMap::<(Layer, DataType), Vec<GeometryBuilder>>::new();
            let mut direct_draw_units = 0_u64;
            let mut texts = Vec::new();
            let mut nodes = Vec::new();
            let mut references = Vec::new();

            for element in cell.iter_elements() {
                if !matches!(element, Element::Reference(_)) {
                    direct_draw_units = direct_draw_units.saturating_add(1);
                }
                match element {
                    Element::Polygon(polygon) => push_polygon_batch(
                        &mut builders,
                        (polygon.layer(), polygon.data_type()),
                        polygon.points(),
                    )?,
                    Element::Box(gds_box) => push_polygon_batch(
                        &mut builders,
                        (gds_box.layer(), gds_box.box_type()),
                        &gds_box.points(),
                    )?,
                    Element::Path(path) => {
                        if let Some(points) = path.to_polygon_points(16) {
                            push_polygon_batch(
                                &mut builders,
                                (path.layer(), path.data_type()),
                                &points,
                            )?;
                        }
                    }
                    Element::Text(text) => texts.push(prepare_text(text)),
                    Element::Node(node) => nodes.push(prepare_node(node)),
                    Element::Reference(reference) => {
                        references.push(prepared.prepare_reference(reference, &cell_bboxes)?);
                    }
                }
            }

            let mut geometry_ids = Vec::new();
            for (layer, layer_builders) in builders {
                for builder in layer_builders {
                    if let Some(geometry) = builder.finish(layer) {
                        geometry_ids.push(prepared.geometries.len());
                        prepared.geometries.push(geometry);
                    }
                }
            }

            prepared.cells.insert(
                cell_name.clone(),
                PreparedCell {
                    geometry_ids,
                    direct_draw_units,
                    texts,
                    nodes,
                    references,
                    bbox: cell_bboxes.get(&cell_name).copied().flatten(),
                },
            );
        }

        Ok(prepared)
    }

    fn prepare_reference(
        &mut self,
        reference: &gdsr::Reference,
        cell_bboxes: &HashMap<String, Option<WorldBBox>>,
    ) -> Result<PreparedReference, SceneBuildError> {
        let mut current = reference;
        let mut grids = Vec::new();
        let (source, source_bbox, cell_label) = loop {
            grids.push(current.grid().clone());
            match current.instance() {
                Instance::Cell(cell_name) => {
                    break (
                        PreparedSource::Cell(cell_name.clone()),
                        cell_bboxes.get(cell_name).copied().flatten(),
                        Some(Arc::<str>::from(cell_name.as_str())),
                    );
                }
                Instance::Element(element) => {
                    let element = element.as_ref().as_ref();
                    match element {
                        Element::Reference(reference) => current = reference,
                        Element::Text(text) => {
                            break (
                                PreparedSource::Text(prepare_text(text)),
                                element.world_bbox(),
                                None,
                            );
                        }
                        Element::Node(node) => {
                            break (
                                PreparedSource::Node(prepare_node(node)),
                                element.world_bbox(),
                                None,
                            );
                        }
                        _ => {
                            let layer = element_layer(element);
                            let geometry_id = self.prepare_inline_geometry(element, layer)?;
                            break (
                                PreparedSource::Geometry(geometry_id),
                                element.world_bbox(),
                                None,
                            );
                        }
                    }
                }
            }
        };
        let label = (grids.len() == 1).then_some(cell_label).flatten();
        let mut suffix_bounds = vec![source_bbox; grids.len() + 1];
        let mut suffix_grid_counts = vec![1_u64; grids.len() + 1];
        for grid_index in (0..grids.len()).rev() {
            suffix_bounds[grid_index] = suffix_bounds[grid_index + 1]
                .and_then(|bounds| reference_bbox(&grids[grid_index], bounds));
            suffix_grid_counts[grid_index] =
                grid_count(&grids[grid_index]).saturating_mul(suffix_grid_counts[grid_index + 1]);
        }

        Ok(PreparedReference {
            grids,
            source,
            source_bbox,
            suffix_bounds,
            suffix_grid_counts,
            label,
        })
    }

    fn prepare_inline_geometry(
        &mut self,
        element: &Element,
        layer: (Layer, DataType),
    ) -> Result<usize, SceneBuildError> {
        let mut builder = GeometryBuilder::default();
        match element {
            Element::Polygon(polygon) => builder.push_polygon(layer, polygon.points())?,
            Element::Box(gds_box) => builder.push_polygon(layer, &gds_box.points())?,
            Element::Path(path) => {
                if let Some(points) = path.to_polygon_points(16) {
                    builder.push_polygon(layer, &points)?;
                }
            }
            Element::Node(_) | Element::Reference(_) | Element::Text(_) => {}
        }
        let geometry_id = self.geometries.len();
        if let Some(geometry) = builder.finish(layer) {
            self.geometries.push(geometry);
        } else {
            self.geometries.push(PreparedGeometry {
                layer,
                positions: Vec::new(),
                triangle_indices: Vec::new(),
                line_indices: Vec::new(),
                origin: DVec2::ZERO,
                bbox: WorldBBox::new(0.0, 0.0, 0.0, 0.0),
                element_count: 0,
            });
        }
        Ok(geometry_id)
    }

    pub fn plan(&self, cell_name: &str, options: &SceneOptions<'_>) -> ScenePlan {
        let mut planner = Planner {
            library: self,
            options,
            plan: ScenePlan::default(),
            complexity_cache: HashMap::new(),
            complexity_stack: HashSet::new(),
            cell_stack: HashSet::new(),
        };

        if options.depth == 0 {
            if let Some(cell) = self.cells.get(cell_name)
                && let Some(bbox) = cell.bbox
                && bbox.overlaps(options.visible)
            {
                planner.add_load(bbox, Some(Arc::<str>::from(cell_name)));
            }
        } else {
            planner.visit_iteratively(cell_name, options.depth);
        }
        planner.plan
    }
}

fn reachable_cell_names(library: &Library, root_cell: &str) -> BTreeSet<String> {
    let mut reachable = BTreeSet::new();
    let mut pending = vec![root_cell.to_string()];
    while let Some(cell_name) = pending.pop() {
        if !reachable.insert(cell_name.clone()) {
            continue;
        }
        if let Some(cell) = library.get_cell(&cell_name) {
            pending.extend(cell.referenced_cell_names().into_iter().map(str::to_string));
        }
    }
    reachable
}

struct Planner<'a> {
    library: &'a PreparedLibrary,
    options: &'a SceneOptions<'a>,
    plan: ScenePlan,
    complexity_cache: HashMap<(String, u32), u64>,
    complexity_stack: HashSet<String>,
    cell_stack: HashSet<String>,
}

impl<'a> Planner<'a> {
    fn visit_iteratively(&mut self, cell_name: &str, depth: u32) {
        let mut actions = vec![PlanAction::Cell {
            cell_name: cell_name.to_string(),
            transform: DAffine2::IDENTITY,
            depth,
        }];
        while let Some(action) = actions.pop() {
            match action {
                PlanAction::Cell {
                    cell_name,
                    transform,
                    depth,
                } => self.visit_cell(&cell_name, transform, depth, &mut actions),
                PlanAction::CellReferences {
                    cell_name,
                    transform,
                    depth,
                    next_reference,
                    world_bbox,
                } => self.visit_cell_references(
                    &cell_name,
                    transform,
                    depth,
                    next_reference,
                    world_bbox,
                    &mut actions,
                ),
                PlanAction::Reference(traversal) => {
                    self.visit_reference(traversal, &mut actions);
                }
            }
        }
    }

    fn visit_cell(
        &mut self,
        cell_name: &str,
        transform: DAffine2,
        depth: u32,
        actions: &mut Vec<PlanAction<'a>>,
    ) {
        let Some(cell) = self.library.cells.get(cell_name) else {
            return;
        };
        let world_bbox = cell.bbox.map(|bbox| transform_bbox(transform, bbox));
        if world_bbox.is_some_and(|bbox| !bbox.overlaps(self.options.visible)) {
            return;
        }
        if self.plan.stats.detail_cells >= MAX_DETAIL_CELLS {
            if let Some(bbox) = world_bbox {
                self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
            }
            return;
        }
        if self.detail_budget_exhausted() {
            if let Some(bbox) = world_bbox {
                self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
            }
            return;
        }
        if !self.cell_stack.insert(cell_name.to_string()) {
            if let Some(bbox) = world_bbox {
                self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
            }
            return;
        }

        self.plan.stats.detail_cells += 1;
        for &geometry_id in &cell.geometry_ids {
            if self.detail_budget_exhausted() {
                if let Some(bbox) = world_bbox {
                    self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
                }
                self.cell_stack.remove(cell_name);
                return;
            }
            self.push_geometry(geometry_id, transform);
        }
        for text in &cell.texts {
            if self.detail_budget_exhausted() {
                if let Some(bbox) = world_bbox {
                    self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
                }
                self.cell_stack.remove(cell_name);
                return;
            }
            self.push_text(text, transform);
        }
        for node in &cell.nodes {
            if self.detail_budget_exhausted() || !self.push_node(node, transform) {
                if let Some(bbox) = world_bbox {
                    self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
                }
                self.cell_stack.remove(cell_name);
                return;
            }
        }
        actions.push(PlanAction::CellReferences {
            cell_name: cell_name.to_string(),
            transform,
            depth,
            next_reference: 0,
            world_bbox,
        });
    }

    fn visit_cell_references(
        &mut self,
        cell_name: &str,
        transform: DAffine2,
        depth: u32,
        next_reference: usize,
        world_bbox: Option<WorldBBox>,
        actions: &mut Vec<PlanAction<'a>>,
    ) {
        let Some(cell) = self.library.cells.get(cell_name) else {
            self.cell_stack.remove(cell_name);
            return;
        };
        let Some(reference) = cell.references.get(next_reference) else {
            self.cell_stack.remove(cell_name);
            return;
        };
        if self.detail_budget_exhausted() {
            if let Some(bbox) = world_bbox {
                self.add_load(bbox, Some(Arc::<str>::from(cell_name)));
            }
            self.cell_stack.remove(cell_name);
            return;
        }

        actions.push(PlanAction::CellReferences {
            cell_name: cell_name.to_string(),
            transform,
            depth,
            next_reference: next_reference + 1,
            world_bbox,
        });
        actions.push(PlanAction::Reference(ReferenceTraversal {
            reference,
            parent_transform: transform,
            depth,
            stack: vec![GridTraversalFrame {
                grid_index: 0,
                transform,
                column: 0,
                row: 0,
                entered: false,
            }],
        }));
    }

    fn visit_reference(
        &mut self,
        mut traversal: ReferenceTraversal<'a>,
        actions: &mut Vec<PlanAction<'a>>,
    ) {
        let reference = traversal.reference;
        if reference.source_bbox.is_none() {
            return;
        }

        while let Some(frame) = traversal.stack.last_mut() {
            let Some(grid) = reference.grids.get(frame.grid_index) else {
                traversal.stack.pop();
                continue;
            };
            let level_depth = u32::try_from(frame.grid_index)
                .map_or(0, |grid_index| traversal.depth.saturating_sub(grid_index));
            let Some(source_bbox) = reference.suffix_bounds[frame.grid_index + 1] else {
                traversal.stack.pop();
                continue;
            };
            let Some(local_bbox) = reference.suffix_bounds[frame.grid_index] else {
                traversal.stack.pop();
                continue;
            };
            let world_bbox = transform_bbox(frame.transform, local_bbox);
            if self.detail_budget_exhausted() {
                if let Some(top_bbox) = reference.suffix_bounds[0] {
                    self.add_load(
                        transform_bbox(traversal.parent_transform, top_bbox),
                        reference.label.clone(),
                    );
                }
                return;
            }
            if !frame.entered {
                if !world_bbox.overlaps(self.options.visible) {
                    self.plan.stats.culled_references += 1;
                    traversal.stack.pop();
                    continue;
                }
                if self.plan.stats.detail_cells >= MAX_DETAIL_CELLS {
                    self.add_load(world_bbox, reference.label.clone());
                    traversal.stack.pop();
                    continue;
                }

                let grid_count = grid_count(grid);
                let draw_units =
                    self.reference_draw_units_from(reference, frame.grid_index, level_depth);
                let source_is_degenerate = source_bbox.min_x == source_bbox.max_x
                    || source_bbox.min_y == source_bbox.max_y;
                let (width_px, height_px) =
                    reference_screen_size(world_bbox, self.options.zoom, source_is_degenerate);
                if should_collapse_reference(
                    width_px,
                    height_px,
                    grid_count,
                    draw_units,
                    level_depth,
                    false,
                ) {
                    self.add_load(world_bbox, reference.label.clone());
                    traversal.stack.pop();
                    continue;
                }
                frame.entered = true;
            }
            if grid.columns() == 0 || grid.rows() == 0 || frame.column >= grid.columns() {
                traversal.stack.pop();
                continue;
            }

            let column = frame.column;
            let row = frame.row;
            if frame.row + 1 < grid.rows() {
                frame.row += 1;
            } else {
                frame.row = 0;
                frame.column += 1;
            }
            let next_grid_index = frame.grid_index + 1;
            let member_transform = frame.transform * grid_member_transform(grid, column, row);
            let member_bbox = transform_bbox(member_transform, source_bbox);
            if !member_bbox.overlaps(self.options.visible) {
                self.plan.stats.culled_references += 1;
                continue;
            }
            let source_is_degenerate =
                source_bbox.min_x == source_bbox.max_x || source_bbox.min_y == source_bbox.max_y;
            let (member_width, member_height) =
                reference_screen_size(member_bbox, self.options.zoom, source_is_degenerate);
            if !member_width.is_finite()
                || !member_height.is_finite()
                || (member_width < 1.0 && member_height < 1.0)
            {
                self.plan.stats.skipped_subpixel_references += 1;
                continue;
            }
            let grid_count = grid_count(grid);
            let draw_units =
                self.reference_draw_units_from(reference, frame.grid_index, level_depth);
            let source_units = draw_units.checked_div(grid_count).unwrap_or(1).max(1);
            if should_collapse_reference(
                member_width,
                member_height,
                1,
                source_units,
                level_depth,
                false,
            ) {
                self.add_load(member_bbox, reference.label.clone());
            } else if next_grid_index < reference.grids.len() {
                traversal.stack.push(GridTraversalFrame {
                    grid_index: next_grid_index,
                    transform: member_transform,
                    column: 0,
                    row: 0,
                    entered: false,
                });
            } else {
                match &reference.source {
                    PreparedSource::Cell(cell_name) => {
                        actions.push(PlanAction::Reference(traversal));
                        actions.push(PlanAction::Cell {
                            cell_name: cell_name.clone(),
                            transform: member_transform,
                            depth: level_depth.saturating_sub(1),
                        });
                        return;
                    }
                    PreparedSource::Geometry(geometry_id) => {
                        self.push_geometry(*geometry_id, member_transform);
                    }
                    PreparedSource::Text(text) => self.push_text(text, member_transform),
                    PreparedSource::Node(node) => {
                        if !self.push_node(node, member_transform) {
                            self.add_load(member_bbox, reference.label.clone());
                            return;
                        }
                    }
                }
            }
        }
    }

    fn detail_budget_exhausted(&self) -> bool {
        self.plan.stats.detail_objects >= MAX_SCENE_OBJECTS
            || self.plan.texts.len() >= MAX_TEXT_INSTANCES
            || self.plan.load_proxies.len() >= MAX_LOAD_PROXIES
    }

    fn push_geometry(&mut self, geometry_id: usize, transform: DAffine2) {
        let geometry = &self.library.geometries[geometry_id];
        if self.options.hidden_layers.contains(&geometry.layer) {
            return;
        }
        let bbox = transform_bbox(transform, geometry.bbox);
        if !bbox.overlaps(self.options.visible) {
            self.plan.stats.culled_geometry_batches += 1;
            return;
        }
        let (width_px, height_px) = screen_size(bbox, self.options.zoom);
        if !width_px.is_finite() || !height_px.is_finite() || (width_px < 1.0 && height_px < 1.0) {
            self.plan.stats.skipped_subpixel_geometry_batches += 1;
            return;
        }
        let area_px = width_px * height_px;
        if geometry.element_count >= DENSE_BATCH_MIN_ELEMENTS
            && area_px.is_finite()
            && area_px / geometry.element_count as f32 <= MIN_AVERAGE_ELEMENT_AREA_PX
        {
            self.add_load(bbox, None);
            return;
        }
        if self.detail_budget_exhausted() {
            self.add_load(bbox, None);
            return;
        }
        self.plan.geometry_instances.push(GeometryInstance {
            geometry_id,
            transform,
        });
        self.plan.stats.detail_objects += 1;
    }

    fn push_text(&mut self, text: &PreparedText, transform: DAffine2) {
        if self.options.hidden_layers.contains(&text.layer) {
            return;
        }
        let position = transform.transform_point2(text.origin);
        if point_in_bbox(position, self.options.visible) {
            self.plan.texts.push(TextInstance {
                value: Arc::clone(&text.value),
                position,
                layer: text.layer,
            });
            self.plan.stats.detail_objects += 1;
        }
    }

    fn push_node(&mut self, node: &PreparedNode, transform: DAffine2) -> bool {
        if self.options.hidden_layers.contains(&node.layer) {
            return true;
        }
        let visible = self.options.visible;
        for point in &node.points {
            let point = transform.transform_point2(*point);
            if !point_in_bbox(point, visible) {
                continue;
            }
            if self.detail_budget_exhausted() {
                return false;
            }
            self.plan
                .node_points
                .entry(node.layer)
                .or_default()
                .push(point);
            self.plan.stats.detail_objects += 1;
        }
        true
    }

    fn add_load(&mut self, bbox: WorldBBox, label: Option<Arc<str>>) {
        if self.plan.load_proxies.len() < MAX_LOAD_PROXIES {
            self.plan.load_proxies.push(LoadProxy { bbox, label });
        }
    }

    fn reference_draw_units_from(
        &mut self,
        reference: &PreparedReference,
        start_grid: usize,
        depth: u32,
    ) -> u64 {
        match Self::reference_draw_units_parts(reference, start_grid, depth) {
            ReferenceDrawUnits::Ready(count) => count,
            ReferenceDrawUnits::Cell {
                cell_name,
                depth,
                multiplier,
            } => multiplier.saturating_mul(self.cell_draw_units(cell_name, depth).max(1)),
        }
    }

    fn reference_draw_units_parts(
        reference: &PreparedReference,
        start_grid: usize,
        depth: u32,
    ) -> ReferenceDrawUnits<'_> {
        let remaining_grids = reference.grids.len().saturating_sub(start_grid);
        if let Ok(grid_depth) = u32::try_from(remaining_grids)
            && depth > grid_depth
        {
            let multiplier = reference.suffix_grid_counts[start_grid];
            return match &reference.source {
                PreparedSource::Cell(cell_name) => ReferenceDrawUnits::Cell {
                    cell_name,
                    depth: depth.saturating_sub(grid_depth),
                    multiplier,
                },
                PreparedSource::Geometry(_) | PreparedSource::Text(_) | PreparedSource::Node(_) => {
                    ReferenceDrawUnits::Ready(multiplier)
                }
            };
        }

        let mut count = 1_u64;
        let mut remaining_depth = depth;
        for grid in reference.grids.iter().skip(start_grid) {
            count = count.saturating_mul(grid_count(grid));
            if count == 0 || remaining_depth <= 1 {
                return ReferenceDrawUnits::Ready(count);
            }
            remaining_depth -= 1;
        }
        match &reference.source {
            PreparedSource::Cell(cell_name) => ReferenceDrawUnits::Cell {
                cell_name,
                depth: remaining_depth,
                multiplier: count,
            },
            PreparedSource::Geometry(_) | PreparedSource::Text(_) | PreparedSource::Node(_) => {
                ReferenceDrawUnits::Ready(count)
            }
        }
    }

    fn cell_draw_units(&mut self, cell_name: &str, depth: u32) -> u64 {
        let key = (cell_name.to_string(), depth);
        if let Some(&count) = self.complexity_cache.get(&key) {
            return count;
        }
        if depth == 0 || self.complexity_stack.contains(cell_name) {
            return 1;
        }
        self.complexity_stack.insert(cell_name.to_string());
        let mut frames = vec![CellComplexityFrame {
            cell_name: cell_name.to_string(),
            depth,
            next_reference: 0,
            count: self
                .library
                .cells
                .get(cell_name)
                .map_or(1, |cell| cell.direct_draw_units),
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
                    .saturating_add(multiplier.saturating_mul(child_count.max(1)));
            }

            let Some(frame) = frames.last_mut() else {
                return 1;
            };
            let reference = self
                .library
                .cells
                .get(&frame.cell_name)
                .and_then(|cell| cell.references.get(frame.next_reference));
            let Some(reference) = reference else {
                let Some(frame) = frames.pop() else {
                    return 1;
                };
                self.complexity_stack.remove(&frame.cell_name);
                self.complexity_cache
                    .insert((frame.cell_name, frame.depth), frame.count);
                if frames.is_empty() {
                    return frame.count;
                }
                completed = Some(frame.count);
                continue;
            };
            frame.next_reference += 1;

            match Self::reference_draw_units_parts(reference, 0, frame.depth) {
                ReferenceDrawUnits::Ready(count) => {
                    frame.count = frame.count.saturating_add(count);
                }
                ReferenceDrawUnits::Cell {
                    cell_name,
                    depth,
                    multiplier,
                } => {
                    let child_key = (cell_name.to_string(), depth);
                    if let Some(&count) = self.complexity_cache.get(&child_key) {
                        frame.count = frame
                            .count
                            .saturating_add(multiplier.saturating_mul(count.max(1)));
                    } else if depth == 0 || self.complexity_stack.contains(cell_name) {
                        frame.count = frame.count.saturating_add(multiplier);
                    } else {
                        frame.pending_multiplier = Some(multiplier);
                        self.complexity_stack.insert(cell_name.to_string());
                        frames.push(CellComplexityFrame {
                            cell_name: cell_name.to_string(),
                            depth,
                            next_reference: 0,
                            count: self
                                .library
                                .cells
                                .get(cell_name)
                                .map_or(1, |cell| cell.direct_draw_units),
                            pending_multiplier: None,
                        });
                    }
                }
            }
        }
    }
}

fn prepare_text(text: &gdsr::Text) -> PreparedText {
    PreparedText {
        value: Arc::<str>::from(text.text().as_str()),
        origin: point_position(text.origin()),
        layer: (text.layer(), text.data_type()),
    }
}

fn prepare_node(node: &gdsr::Node) -> PreparedNode {
    PreparedNode {
        points: node.points().iter().map(point_position).collect(),
        layer: (node.layer(), node.node_type()),
    }
}

fn element_layer(element: &Element) -> (Layer, DataType) {
    match element {
        Element::Polygon(polygon) => (polygon.layer(), polygon.data_type()),
        Element::Box(gds_box) => (gds_box.layer(), gds_box.box_type()),
        Element::Path(path) => (path.layer(), path.data_type()),
        Element::Node(node) => (node.layer(), node.node_type()),
        Element::Text(text) => (text.layer(), text.data_type()),
        Element::Reference(_) => (Layer::default(), DataType::default()),
    }
}

fn point_position(point: &Point) -> DVec2 {
    DVec2::new(point.x().absolute_value(), point.y().absolute_value())
}

fn grid_count(grid: &Grid) -> u64 {
    u64::from(grid.columns()) * u64::from(grid.rows())
}

fn grid_member_transform(grid: &Grid, column: u32, row: u32) -> DAffine2 {
    let angle = grid.angle().value();
    let (sin, cos) = angle.sin_cos();
    let magnification = grid.magnification();
    let spacing_x = grid
        .spacing_x()
        .map_or(DVec2::ZERO, |point| point_position(&point));
    let spacing_y = grid
        .spacing_y()
        .map_or(DVec2::ZERO, |point| point_position(&point));
    let offset = spacing_x * f64::from(column) + spacing_y * f64::from(row);
    let rotated_offset = DVec2::new(
        offset.x.mul_add(cos, -(offset.y * sin)),
        offset.x.mul_add(sin, offset.y * cos),
    );
    let origin = point_position(&grid.origin()) + rotated_offset;
    let x_axis = DVec2::new(cos, sin) * magnification;
    let y_axis = if grid.x_reflection() {
        DVec2::new(sin, -cos) * magnification
    } else {
        DVec2::new(-sin, cos) * magnification
    };
    DAffine2::from_mat2_translation(DMat2::from_cols(x_axis, y_axis), origin)
}

fn reference_bbox(grid: &Grid, source_bbox: WorldBBox) -> Option<WorldBBox> {
    if grid.columns() == 0 || grid.rows() == 0 {
        return None;
    }
    let mut bbox = None;
    for column in [0, grid.columns() - 1] {
        for row in [0, grid.rows() - 1] {
            let member_bbox = transform_bbox(grid_member_transform(grid, column, row), source_bbox);
            bbox = Some(bbox.map_or(member_bbox, |bbox: WorldBBox| bbox.merge(&member_bbox)));
        }
    }
    bbox
}

fn transform_bbox(transform: DAffine2, bbox: WorldBBox) -> WorldBBox {
    let corners = [
        DVec2::new(bbox.min_x, bbox.min_y),
        DVec2::new(bbox.max_x, bbox.min_y),
        DVec2::new(bbox.max_x, bbox.max_y),
        DVec2::new(bbox.min_x, bbox.max_y),
    ];
    let mut min = DVec2::splat(f64::INFINITY);
    let mut max = DVec2::splat(f64::NEG_INFINITY);
    for corner in corners {
        let corner = transform.transform_point2(corner);
        min = min.min(corner);
        max = max.max(corner);
    }
    WorldBBox::new(min.x, min.y, max.x, max.y)
}

fn screen_size(bbox: WorldBBox, zoom: f64) -> (f32, f32) {
    (
        ((bbox.max_x - bbox.min_x).abs() * zoom) as f32,
        ((bbox.max_y - bbox.min_y).abs() * zoom) as f32,
    )
}

fn reference_screen_size(bbox: WorldBBox, zoom: f64, source_is_degenerate: bool) -> (f32, f32) {
    let (width, height) = screen_size(bbox, zoom);
    if source_is_degenerate {
        (
            width.max(MIN_FIXED_CONTENT_PX),
            height.max(MIN_FIXED_CONTENT_PX),
        )
    } else {
        (width, height)
    }
}

fn point_in_bbox(point: DVec2, bbox: &WorldBBox) -> bool {
    point.x >= bbox.min_x && point.x <= bbox.max_x && point.y >= bbox.min_y && point.y <= bbox.max_y
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{
        Cell, DataType, HorizontalPresentation, Layer, Path, Polygon, Radians, Reference, Text,
        Unit, VerticalPresentation,
    };

    fn point(x: i32, y: i32) -> Point {
        Point::integer(x, y, 1e-9)
    }

    fn square(size: i32, layer: u16) -> Polygon {
        square_at(0, 0, size, layer)
    }

    fn square_at(x: i32, y: i32, size: i32, layer: u16) -> Polygon {
        Polygon::new(
            [
                point(x, y),
                point(x + size, y),
                point(x + size, y + size),
                point(x, y + size),
            ],
            Layer::new(layer),
            DataType::new(0),
        )
    }

    fn visible() -> WorldBBox {
        WorldBBox::new(-1.0, -1.0, 1.0, 1.0)
    }

    fn label() -> Text {
        Text::new(
            "label",
            point(0, 0),
            Layer::new(1),
            DataType::new(0),
            1.0,
            Radians::default(),
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        )
    }

    #[test]
    fn member_transform_matches_reference_geometry() {
        let reference = Reference::new("leaf").with_grid(
            Grid::default()
                .with_origin(point(100, 200))
                .with_columns(2)
                .with_spacing_x(Some(point(40, 0)))
                .with_magnification(2.0)
                .with_angle(Radians::FRAC_PI_2)
                .with_x_reflection(true),
        );
        let element = Element::Polygon(square(10, 1));
        let transformed = reference.get_elements_in_grid(&element);
        let Element::Polygon(second) = &transformed[1] else {
            panic!("reference should preserve polygon type");
        };
        let affine = grid_member_transform(reference.grid(), 1, 0);

        for (actual, source) in second.points().iter().zip(square(10, 1).points()) {
            let expected = affine.transform_point2(point_position(source));
            assert!((actual.x().absolute_value() - expected.x).abs() < 1e-12);
            assert!((actual.y().absolute_value() - expected.y).abs() < 1e-12);
        }
    }

    #[test]
    fn repeated_cells_share_one_prepared_geometry() {
        let mut leaf = Cell::new("leaf");
        leaf.add(square(100, 1));
        let mut top = Cell::new("top");
        top.add(
            Reference::new("leaf").with_grid(
                Grid::default()
                    .with_columns(10)
                    .with_rows(10)
                    .with_spacing_x(Some(point(200, 0)))
                    .with_spacing_y(Some(point(0, 200))),
            ),
        );
        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);

        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e9,
                depth: 2,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(prepared.geometries.len(), 1);
        assert_eq!(plan.geometry_instances.len(), 100);
        assert!(
            plan.geometry_instances
                .iter()
                .all(|instance| instance.geometry_id == 0)
        );
    }

    #[test]
    fn preparation_skips_cells_outside_selected_hierarchy() {
        let mut top = Cell::new("top");
        top.add(square(100, 1));
        let mut unused = Cell::new("unused");
        unused.add(square(100, 2));
        let mut library = Library::new("test");
        library.add_cell(top);
        library.add_cell(unused);

        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        assert_eq!(prepared.cells.len(), 1);
        assert!(prepared.cells.contains_key("top"));
        assert_eq!(prepared.geometries.len(), 1);
        assert_eq!(
            prepared.geometries[0].layer,
            (Layer::new(1), DataType::new(0))
        );
    }

    #[test]
    fn subpixel_grid_members_are_not_expanded() {
        let mut leaf = Cell::new("leaf");
        leaf.add(square(1, 1));
        let mut top = Cell::new("top");
        top.add(
            Reference::new("leaf").with_grid(
                Grid::default()
                    .with_columns(10)
                    .with_rows(10)
                    .with_spacing_x(Some(point(10_000, 0)))
                    .with_spacing_y(Some(point(0, 10_000))),
            ),
        );
        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);

        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e6,
                depth: 2,
                hidden_layers: &HashSet::new(),
            },
        );

        assert!(plan.geometry_instances.is_empty());
        assert_eq!(plan.stats.skipped_subpixel_references, 100);
    }

    #[test]
    fn depth_zero_uses_one_cell_proxy() {
        let mut cell = Cell::new("top");
        cell.add(square(100, 1));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e9,
                depth: 0,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.load_proxies.len(), 1);
        assert_eq!(plan.load_proxies[0].label.as_deref(), Some("top"));
        assert!(plan.geometry_instances.is_empty());
    }

    #[test]
    fn hidden_layers_do_not_emit_geometry() {
        let mut cell = Cell::new("top");
        cell.add(square(100, 7));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let hidden = HashSet::from([(Layer::new(7), DataType::new(0))]);

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e9,
                depth: 1,
                hidden_layers: &hidden,
            },
        );

        assert!(plan.geometry_instances.is_empty());
    }

    #[test]
    fn large_direct_cells_are_split_into_bounded_batches() {
        let mut cell = Cell::new("top");
        for x in 0..257 {
            cell.add(square_at(x * 20, 0, 10, 1));
        }
        let mut library = Library::new("test");
        library.add_cell(cell);

        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        assert_eq!(prepared.geometries.len(), 2);
        assert_eq!(prepared.geometries[0].element_count, MAX_BATCH_ELEMENTS);
        assert_eq!(prepared.geometries[1].element_count, 1);
    }

    #[test]
    fn offscreen_geometry_batches_are_culled() {
        let mut cell = Cell::new("top");
        for x in 0..256 {
            cell.add(square_at(x * 20, 0, 10, 1));
        }
        cell.add(square_at(1_000_000, 0, 10, 1));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let visible = WorldBBox::new(0.000_999, -0.001, 0.001_001, 0.001);

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible,
                zoom: 1.0e12,
                depth: 1,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.geometry_instances.len(), 1);
        assert_eq!(plan.stats.culled_geometry_batches, 1);
    }

    #[test]
    fn cyclic_parent_bounds_do_not_cull_reachable_geometry() {
        let mut a = Cell::new("a");
        a.add(square_at(0, 0, 1, 1));
        a.add(Reference::new("b"));
        let mut b = Cell::new("b");
        b.add(square_at(100, 0, 1, 2));
        b.add(Reference::new("a"));
        let mut top = Cell::new("top");
        top.add(Reference::new("a"));
        let mut library = Library::new("test");
        library.add_cell(top);
        library.add_cell(b);
        library.add_cell(a);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let visible = WorldBBox::new(99.0e-9, -1.0e-9, 102.0e-9, 2.0e-9);

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible,
                zoom: 1.0e12,
                depth: 4,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.geometry_instances.len(), 1);
        let geometry = &prepared.geometries[plan.geometry_instances[0].geometry_id];
        assert_eq!(geometry.layer, (Layer::new(2), DataType::new(0)));
    }

    #[test]
    fn signed_inline_path_bounds_do_not_cull_visible_geometry() {
        let path = Path::new(
            [Point::float(0.0, 0.0, 1.0), Point::float(10.0, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
            None,
            Some(Unit::float(-4.0, 1.0)),
            Some(Unit::float(-100.0, 1.0)),
            Some(Unit::float(-100.0, 1.0)),
        );
        let mut top = Cell::new("top");
        top.add(Reference::new(Element::Path(path)));
        let mut library = Library::new("test");
        library.add_cell(top);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let visible = WorldBBox::new(4.0, 1.0, 6.0, 2.0);

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible,
                zoom: 100.0,
                depth: 2,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.geometry_instances.len(), 1);
    }

    #[test]
    fn dense_subpixel_geometry_uses_one_proxy() {
        let mut cell = Cell::new("top");
        for _ in 0..DENSE_BATCH_MIN_ELEMENTS {
            cell.add(square(100, 1));
        }
        let mut library = Library::new("test");
        library.add_cell(cell);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e8,
                depth: 1,
                hidden_layers: &HashSet::new(),
            },
        );

        assert!(plan.geometry_instances.is_empty());
        assert_eq!(plan.load_proxies.len(), 1);
    }

    #[test]
    fn subpixel_direct_geometry_is_skipped() {
        let mut cell = Cell::new("top");
        cell.add(square(1, 1));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e6,
                depth: 1,
                hidden_layers: &HashSet::new(),
            },
        );

        assert!(plan.geometry_instances.is_empty());
        assert_eq!(plan.stats.skipped_subpixel_geometry_batches, 1);
    }

    #[test]
    fn nested_inline_arrays_stop_at_geometry_budget() {
        let inner = Reference::new(Element::Polygon(square(100, 1))).with_grid(
            Grid::default()
                .with_columns(100)
                .with_rows(50)
                .with_spacing_x(Some(point(200, 0)))
                .with_spacing_y(Some(point(0, 200))),
        );
        let outer = Reference::new(Element::Reference(inner)).with_grid(
            Grid::default()
                .with_columns(5)
                .with_spacing_x(Some(point(100_000, 0))),
        );
        let mut top = Cell::new("top");
        top.add(outer);
        let mut library = Library::new("test");
        library.add_cell(top);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e12,
                depth: 3,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.geometry_instances.len(), MAX_SCENE_OBJECTS);
        assert_eq!(plan.load_proxies.len(), 1);
    }

    #[test]
    fn ten_thousand_inline_references_prepare_and_plan_iteratively() {
        let mut reference = Reference::new(Element::Polygon(square(100, 1)));
        for _ in 1..10_000 {
            reference = Reference::new(Element::Reference(reference));
        }
        let mut top = Cell::new("top");
        top.add(reference);
        let mut library = Library::new("test");
        library.add_cell(top);

        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");
        let top = prepared.cells.get("top").expect("top should be prepared");
        assert_eq!(top.references[0].grids.len(), 10_000);
        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e12,
                depth: 10_001,
                hidden_layers: &HashSet::new(),
            },
        );
        assert_eq!(plan.geometry_instances.len(), 1);
    }

    #[test]
    fn ten_thousand_named_cells_prepare_and_plan_iteratively() {
        const DEPTH: usize = 10_000;
        let mut library = Library::new("test");
        let leaf_name = format!("cell_{:05}", DEPTH - 1);
        let mut leaf = Cell::new(&leaf_name);
        leaf.add(square(100, 1));
        library.add_cell(leaf);
        for index in (0..DEPTH - 1).rev() {
            let cell_name = format!("cell_{index:05}");
            let mut cell = Cell::new(&cell_name);
            cell.add(Reference::new(format!("cell_{:05}", index + 1)));
            library.add_cell(cell);
        }

        let prepared =
            PreparedLibrary::build(&library, "cell_00000").expect("scene should prepare");
        let plan = prepared.plan(
            "cell_00000",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e12,
                depth: u32::try_from(DEPTH + 1).expect("depth should fit"),
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.geometry_instances.len(), 1);
        assert_eq!(plan.stats.detail_cells, DEPTH);
    }

    #[test]
    fn referenced_text_is_not_discarded_as_subpixel_geometry() {
        let mut top = Cell::new("top");
        top.add(Reference::new(Element::Text(label())));
        let mut library = Library::new("test");
        library.add_cell(top);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e9,
                depth: 2,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.texts.len(), 1);
        assert!(plan.load_proxies.is_empty());
    }

    #[test]
    fn nested_inline_text_arrays_stop_at_text_budget() {
        let inner = Reference::new(Element::Text(label())).with_grid(
            Grid::default()
                .with_columns(100)
                .with_rows(50)
                .with_spacing_x(Some(point(200, 0)))
                .with_spacing_y(Some(point(0, 200))),
        );
        let outer = Reference::new(Element::Reference(inner)).with_grid(
            Grid::default()
                .with_columns(5)
                .with_spacing_x(Some(point(100_000, 0))),
        );
        let mut top = Cell::new("top");
        top.add(outer);
        let mut library = Library::new("test");
        library.add_cell(top);
        let prepared = PreparedLibrary::build(&library, "top").expect("scene should prepare");

        let plan = prepared.plan(
            "top",
            &SceneOptions {
                visible: &visible(),
                zoom: 1.0e12,
                depth: 3,
                hidden_layers: &HashSet::new(),
            },
        );

        assert_eq!(plan.texts.len(), MAX_TEXT_INSTANCES);
        assert_eq!(plan.load_proxies.len(), 1);
    }
}
