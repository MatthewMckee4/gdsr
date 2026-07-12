use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Indices, Mesh, PrimitiveTopology};
use gdsr::{DataType, Element, Layer, Library, Point};

use crate::drawable::{
    Drawable, WorldBBox, reference_draw_units_for_depth, reference_grid_count,
    reference_instance_bbox, should_collapse_reference,
};
use crate::spatial::SpatialGrid;

pub struct BevyLayerBatch {
    pub layer: (Layer, DataType),
    pub mesh: Mesh,
    pub element_count: usize,
}

pub struct BevyReferenceLoad {
    pub bbox: WorldBBox,
    pub cell_name: Option<String>,
}

pub struct BevyRenderScene {
    pub batches: Vec<BevyLayerBatch>,
    pub reference_lods: Vec<BevyReferenceLoad>,
    pub batched_element_count: usize,
    pub skipped_element_count: usize,
}

pub struct BevySceneOptions<'a> {
    pub visible: &'a WorldBBox,
    pub zoom: f64,
    pub library: Option<&'a Library>,
    pub render_depth: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BevyRenderError {
    TooManyVertices { layer: (Layer, DataType) },
}

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    element_count: usize,
}

const MIN_VISIBLE_PX: f32 = 1.0;
const BBOX_FALLBACK_PX: f32 = 8.0;

pub fn build_visible_scene(
    elements: &[Element],
    visible_indices: &[u32],
    hidden_layers: &BTreeSet<(Layer, DataType)>,
) -> Result<BevyRenderScene, BevyRenderError> {
    let visible = WorldBBox::new(f64::MIN, f64::MIN, f64::MAX, f64::MAX);
    build_visible_scene_with_options(
        elements,
        visible_indices,
        hidden_layers,
        &BevySceneOptions {
            visible: &visible,
            zoom: 1.0,
            library: None,
            render_depth: 1,
        },
    )
}

pub fn build_visible_scene_with_options(
    elements: &[Element],
    visible_indices: &[u32],
    hidden_layers: &BTreeSet<(Layer, DataType)>,
    options: &BevySceneOptions<'_>,
) -> Result<BevyRenderScene, BevyRenderError> {
    let mut builder = SceneBuilder::new(hidden_layers, options);
    for &idx in visible_indices {
        let Some(element) = elements.get(idx as usize) else {
            builder.skipped_element_count += 1;
            continue;
        };
        builder.push_element(element, options.render_depth)?;
    }
    Ok(builder.into_scene())
}

pub fn build_visible_scene_from_grid(
    elements: &[Element],
    grid: &SpatialGrid,
    visible: &WorldBBox,
    hidden_layers: &BTreeSet<(Layer, DataType)>,
    query_buf: &mut Vec<u32>,
) -> Result<BevyRenderScene, BevyRenderError> {
    let visible_indices = grid.query_visible_indices(visible, query_buf);
    build_visible_scene(elements, visible_indices, hidden_layers)
}

pub fn build_visible_scene_from_grid_with_options(
    elements: &[Element],
    grid: &SpatialGrid,
    visible: &WorldBBox,
    hidden_layers: &BTreeSet<(Layer, DataType)>,
    query_buf: &mut Vec<u32>,
    options: &BevySceneOptions<'_>,
) -> Result<BevyRenderScene, BevyRenderError> {
    let visible_indices = grid.query_visible_indices(visible, query_buf);
    let mut builder = SceneBuilder::new(hidden_layers, options);
    for &idx in visible_indices {
        let Some(element) = elements.get(idx as usize) else {
            builder.skipped_element_count += 1;
            continue;
        };
        builder.push_element(element, options.render_depth)?;
    }

    for element in elements {
        if matches!(element, Element::Reference(_)) {
            builder.push_element(element, options.render_depth)?;
        }
    }

    Ok(builder.into_scene())
}

struct SceneBuilder<'a> {
    hidden_layers: &'a BTreeSet<(Layer, DataType)>,
    options: &'a BevySceneOptions<'a>,
    builders: BTreeMap<(Layer, DataType), MeshBuilder>,
    reference_lods: Vec<BevyReferenceLoad>,
    skipped_element_count: usize,
    cell_bbox_cache: HashMap<String, Option<WorldBBox>>,
    cell_complexity_cache: HashMap<(String, u32), u64>,
    reference_stack: HashSet<String>,
}

impl<'a> SceneBuilder<'a> {
    fn new(
        hidden_layers: &'a BTreeSet<(Layer, DataType)>,
        options: &'a BevySceneOptions<'a>,
    ) -> Self {
        Self {
            hidden_layers,
            options,
            builders: BTreeMap::new(),
            reference_lods: Vec::new(),
            skipped_element_count: 0,
            cell_bbox_cache: HashMap::new(),
            cell_complexity_cache: HashMap::new(),
            reference_stack: HashSet::new(),
        }
    }

    fn into_scene(self) -> BevyRenderScene {
        let mut batched_element_count = 0;
        let batches = self
            .builders
            .into_iter()
            .filter_map(|(layer, builder)| {
                let element_count = builder.element_count;
                let mesh = builder.into_mesh()?;
                batched_element_count += element_count;
                Some(BevyLayerBatch {
                    layer,
                    mesh,
                    element_count,
                })
            })
            .collect();

        BevyRenderScene {
            batches,
            reference_lods: self.reference_lods,
            batched_element_count,
            skipped_element_count: self.skipped_element_count,
        }
    }

    fn push_element(&mut self, element: &Element, depth: u32) -> Result<(), BevyRenderError> {
        if let Element::Reference(reference) = element {
            self.push_reference(reference, depth)?;
            return Ok(());
        }

        let Some(bbox) = element.world_bbox() else {
            self.skipped_element_count += 1;
            return Ok(());
        };
        if !bbox.overlaps(self.options.visible) {
            return Ok(());
        }

        let Some((layer, points)) = element_polygon_points(element) else {
            self.skipped_element_count += 1;
            return Ok(());
        };
        if self.hidden_layers.contains(&layer) {
            return Ok(());
        }

        let Some(points) = element_level_of_detail_points(&bbox, points, self.options.zoom) else {
            return Ok(());
        };

        let builder = self.builders.entry(layer).or_default();
        if builder.push_polygon(layer, &points)? {
            builder.element_count += 1;
        } else {
            self.skipped_element_count += 1;
        }
        Ok(())
    }

    fn push_reference(
        &mut self,
        reference: &gdsr::Reference,
        depth: u32,
    ) -> Result<(), BevyRenderError> {
        let Some(bbox) =
            reference_instance_bbox(reference, self.options.library, &mut self.cell_bbox_cache)
        else {
            self.skipped_element_count += 1;
            return Ok(());
        };
        if !bbox.overlaps(self.options.visible) {
            return Ok(());
        }

        let width_px = ((bbox.max_x - bbox.min_x) * self.options.zoom) as f32;
        let height_px = ((bbox.max_y - bbox.min_y) * self.options.zoom) as f32;
        let draw_unit_count = reference_draw_units_for_depth(
            reference,
            self.options.library,
            depth,
            &mut self.cell_complexity_cache,
            &mut self.reference_stack,
        );
        if should_collapse_reference(
            width_px.abs(),
            height_px.abs(),
            reference_grid_count(reference),
            draw_unit_count,
            depth,
            false,
        ) {
            self.reference_lods.push(BevyReferenceLoad {
                bbox,
                cell_name: reference.instance().as_cell().cloned(),
            });
            return Ok(());
        }

        let next_depth = depth.saturating_sub(1);
        if let Some(element) = reference.instance().as_element() {
            for element in reference.get_elements_in_grid(element.as_ref().as_ref()) {
                self.push_element(&element, next_depth)?;
            }
        } else if let Some(cell_name) = reference.instance().as_cell() {
            if !self.reference_stack.insert(cell_name.clone()) {
                self.reference_lods.push(BevyReferenceLoad {
                    bbox,
                    cell_name: Some(cell_name.clone()),
                });
                return Ok(());
            }
            if let Some(cell) = self
                .options
                .library
                .and_then(|library| library.get_cell(cell_name))
            {
                for element in cell.iter_elements() {
                    for transformed in reference.get_elements_in_grid(element) {
                        self.push_element(&transformed, next_depth)?;
                    }
                }
            }
            self.reference_stack.remove(cell_name);
        }

        Ok(())
    }
}

fn element_level_of_detail_points(
    bbox: &WorldBBox,
    points: Vec<Point>,
    zoom: f64,
) -> Option<Vec<Point>> {
    let Some((width_px, height_px)) = screen_size(bbox, zoom) else {
        return Some(points);
    };

    if width_px < MIN_VISIBLE_PX && height_px < MIN_VISIBLE_PX {
        return None;
    }

    if width_px < BBOX_FALLBACK_PX || height_px < BBOX_FALLBACK_PX {
        Some(bbox_points(bbox))
    } else {
        Some(points)
    }
}

fn screen_size(bbox: &WorldBBox, zoom: f64) -> Option<(f32, f32)> {
    if !zoom.is_finite() || zoom <= 1.0 {
        return None;
    }

    let width = ((bbox.max_x - bbox.min_x) * zoom).abs() as f32;
    let height = ((bbox.max_y - bbox.min_y) * zoom).abs() as f32;
    if width.is_finite() && height.is_finite() {
        Some((width, height))
    } else {
        None
    }
}

fn bbox_points(bbox: &WorldBBox) -> Vec<Point> {
    vec![
        Point::float(bbox.min_x, bbox.min_y, 1.0),
        Point::float(bbox.max_x, bbox.min_y, 1.0),
        Point::float(bbox.max_x, bbox.max_y, 1.0),
        Point::float(bbox.min_x, bbox.max_y, 1.0),
        Point::float(bbox.min_x, bbox.min_y, 1.0),
    ]
}

impl MeshBuilder {
    fn push_polygon(
        &mut self,
        layer: (Layer, DataType),
        points: &[Point],
    ) -> Result<bool, BevyRenderError> {
        let mut coords: Vec<(f64, f64)> = points
            .iter()
            .map(|p| (p.x().absolute_value(), p.y().absolute_value()))
            .collect();
        if coords.len() >= 2 && coords.first() == coords.last() {
            coords.pop();
        }
        if coords.len() < 3 {
            return Ok(false);
        }

        let mut flat = Vec::with_capacity(coords.len() * 2);
        for &(x, y) in &coords {
            flat.push(x);
            flat.push(y);
        }
        let Ok(indices) = earcutr::earcut(&flat, &[], 2) else {
            return Ok(false);
        };
        if indices.is_empty() {
            return Ok(false);
        }

        let base = u32::try_from(self.positions.len())
            .map_err(|_| BevyRenderError::TooManyVertices { layer })?;
        for &(x, y) in &coords {
            self.positions.push([x as f32, y as f32, 0.0]);
        }
        for idx in indices {
            let idx = u32::try_from(idx).map_err(|_| BevyRenderError::TooManyVertices { layer })?;
            self.indices.push(base + idx);
        }

        Ok(true)
    }

    fn into_mesh(self) -> Option<Mesh> {
        if self.positions.is_empty() || self.indices.is_empty() {
            return None;
        }

        Some(
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_indices(Indices::U32(self.indices)),
        )
    }
}

fn element_polygon_points(element: &Element) -> Option<((Layer, DataType), Vec<Point>)> {
    match element {
        Element::Polygon(polygon) => Some((
            (polygon.layer(), polygon.data_type()),
            polygon.points().to_vec(),
        )),
        Element::Box(gds_box) => Some((
            (gds_box.layer(), gds_box.box_type()),
            gds_box.points().to_vec(),
        )),
        Element::Path(path) => path
            .to_polygon_points(16)
            .map(|points| ((path.layer(), path.data_type()), points)),
        Element::Node(_) | Element::Text(_) | Element::Reference(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_mesh::VertexAttributeValues;

    use crate::testutil::helpers::{path, polygon, text};
    use crate::viewport::bounds::compute_bounds;

    fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => panic!("mesh should have f32x3 positions"),
        }
    }

    fn mesh_index_count(mesh: &Mesh) -> usize {
        match mesh.indices() {
            Some(Indices::U32(indices)) => indices.len(),
            _ => panic!("mesh should have u32 indices"),
        }
    }

    fn visible() -> WorldBBox {
        WorldBBox::new(-1.0, -1.0, 1.0, 1.0)
    }

    #[test]
    fn build_visible_scene_batches_polygon_meshes_by_layer() {
        let elements = vec![
            polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0),
            polygon(vec![(200, 0), (300, 0), (300, 100), (200, 100)], 1, 0),
        ];
        let hidden_layers = BTreeSet::new();

        let scene = build_visible_scene(&elements, &[0, 1], &hidden_layers)
            .expect("test scene should build");

        assert_eq!(scene.batches.len(), 1);
        assert_eq!(scene.reference_lods.len(), 0);
        assert_eq!(scene.batched_element_count, 2);
        assert_eq!(scene.skipped_element_count, 0);
        assert_eq!(scene.batches[0].element_count, 2);
        assert_eq!(mesh_positions(&scene.batches[0].mesh).len(), 8);
        assert_eq!(mesh_index_count(&scene.batches[0].mesh), 12);
    }

    #[test]
    fn build_visible_scene_uses_spatial_indices() {
        let scale = 1e-9;
        let elements = vec![
            polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0),
            polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0),
        ];
        let bounds = compute_bounds(&elements).expect("test elements should have bounds");
        let grid = SpatialGrid::build(&elements, &bounds);
        let visible = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let mut query_buf = Vec::new();
        let scene = build_visible_scene_from_grid(
            &elements,
            &grid,
            &visible,
            &BTreeSet::new(),
            &mut query_buf,
        )
        .expect("test scene should build");

        assert_eq!(scene.batched_element_count, 1);
        assert_eq!(scene.batches[0].layer, (Layer::new(1), DataType::new(0)));
        assert_eq!(query_buf, &[0]);
    }

    #[test]
    fn build_visible_scene_skips_hidden_layers_and_unsupported_elements() {
        let elements = vec![
            polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0),
            polygon(vec![(200, 0), (300, 0), (300, 100)], 2, 0),
            text("label", 0, 0, 3),
        ];
        let hidden_layers = BTreeSet::from([(Layer::new(2), DataType::new(0))]);

        let scene = build_visible_scene(&elements, &[0, 1, 2], &hidden_layers)
            .expect("test scene should build");

        assert_eq!(scene.batches.len(), 1);
        assert_eq!(scene.batches[0].layer, (Layer::new(1), DataType::new(0)));
        assert_eq!(scene.batched_element_count, 1);
        assert_eq!(scene.skipped_element_count, 1);
    }

    #[test]
    fn build_visible_scene_expands_paths_to_meshes() {
        let elements = vec![path(vec![(0, 0), (100, 0), (100, 100)], 7, 2, Some(10))];

        let scene =
            build_visible_scene(&elements, &[0], &BTreeSet::new()).expect("path should build");

        assert_eq!(scene.batched_element_count, 1);
        assert_eq!(scene.batches[0].layer, (Layer::new(7), DataType::new(2)));
        assert!(!mesh_positions(&scene.batches[0].mesh).is_empty());
        assert!(scene.batches[0].mesh.indices().is_some());
    }

    #[test]
    fn bevy_scene_collapses_dense_referenced_cell_to_load() {
        let mut leaf = gdsr::Cell::new("leaf");
        for i in 0..200 {
            let x = i * 20;
            leaf.add(polygon(
                vec![(x, 0), (x + 10, 0), (x + 10, 10), (x, 10)],
                1,
                0,
            ));
        }
        let mut library = Library::new("lib");
        library.add_cell(leaf);
        let elements = vec![Element::Reference(gdsr::Reference::new("leaf"))];
        let visible = visible();

        let scene = build_visible_scene_with_options(
            &elements,
            &[0],
            &BTreeSet::new(),
            &BevySceneOptions {
                visible: &visible,
                zoom: 1.0e6,
                library: Some(&library),
                render_depth: 2,
            },
        )
        .expect("scene should build");

        assert_eq!(scene.batched_element_count, 0);
        assert_eq!(scene.reference_lods.len(), 1);
        assert_eq!(scene.reference_lods[0].cell_name.as_deref(), Some("leaf"));
    }

    #[test]
    fn bevy_scene_expands_sparse_referenced_cell() {
        let mut leaf = gdsr::Cell::new("leaf");
        leaf.add(polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0));
        leaf.add(polygon(
            vec![(200, 0), (300, 0), (300, 100), (200, 100)],
            1,
            0,
        ));
        let mut library = Library::new("lib");
        library.add_cell(leaf);
        let elements = vec![Element::Reference(gdsr::Reference::new("leaf"))];
        let visible = visible();

        let scene = build_visible_scene_with_options(
            &elements,
            &[0],
            &BTreeSet::new(),
            &BevySceneOptions {
                visible: &visible,
                zoom: 1.0e12,
                library: Some(&library),
                render_depth: 2,
            },
        )
        .expect("scene should build");

        assert_eq!(scene.reference_lods.len(), 0);
        assert_eq!(scene.batched_element_count, 2);
    }

    #[test]
    fn bevy_scene_skips_subpixel_polygon() {
        let elements = vec![polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0)];
        let visible = visible();

        let scene = build_visible_scene_with_options(
            &elements,
            &[0],
            &BTreeSet::new(),
            &BevySceneOptions {
                visible: &visible,
                zoom: 1.0e6,
                library: None,
                render_depth: 1,
            },
        )
        .expect("scene should build");

        assert!(scene.batches.is_empty());
        assert_eq!(scene.batched_element_count, 0);
    }

    #[test]
    fn bevy_scene_uses_bbox_proxy_for_skinny_polygon() {
        let elements = vec![polygon(
            vec![(0, 0), (4000, 0), (4000, 4), (3000, 4), (2000, 2), (0, 4)],
            1,
            0,
        )];
        let visible = visible();

        let scene = build_visible_scene_with_options(
            &elements,
            &[0],
            &BTreeSet::new(),
            &BevySceneOptions {
                visible: &visible,
                zoom: 1.0e9,
                library: None,
                render_depth: 1,
            },
        )
        .expect("scene should build");

        assert_eq!(scene.batched_element_count, 1);
        assert_eq!(scene.batches.len(), 1);
        assert_eq!(mesh_positions(&scene.batches[0].mesh).len(), 4);
        assert_eq!(mesh_index_count(&scene.batches[0].mesh), 6);
    }
}
