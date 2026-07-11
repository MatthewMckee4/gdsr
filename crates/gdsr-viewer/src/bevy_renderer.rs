use std::collections::{BTreeMap, BTreeSet};

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Indices, Mesh, PrimitiveTopology};
use gdsr::{DataType, Element, Layer, Point};

use crate::drawable::WorldBBox;
use crate::spatial::SpatialGrid;

pub struct BevyLayerBatch {
    pub layer: (Layer, DataType),
    pub mesh: Mesh,
    pub element_count: usize,
}

pub struct BevyRenderScene {
    pub batches: Vec<BevyLayerBatch>,
    pub batched_element_count: usize,
    pub skipped_element_count: usize,
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

pub fn build_visible_scene(
    elements: &[Element],
    visible_indices: &[u32],
    hidden_layers: &BTreeSet<(Layer, DataType)>,
) -> Result<BevyRenderScene, BevyRenderError> {
    let mut builders = BTreeMap::new();
    let mut skipped_element_count = 0;

    for &idx in visible_indices {
        let Some(element) = elements.get(idx as usize) else {
            skipped_element_count += 1;
            continue;
        };
        let Some((layer, points)) = element_polygon_points(element) else {
            skipped_element_count += 1;
            continue;
        };
        if hidden_layers.contains(&layer) {
            continue;
        }

        let builder = builders.entry(layer).or_insert_with(MeshBuilder::default);
        if builder.push_polygon(layer, &points)? {
            builder.element_count += 1;
        } else {
            skipped_element_count += 1;
        }
    }

    let mut batched_element_count = 0;
    let batches = builders
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

    Ok(BevyRenderScene {
        batches,
        batched_element_count,
        skipped_element_count,
    })
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
            polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0),
            polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0),
        ];
        let bounds = compute_bounds(&elements).expect("test elements should have bounds");
        let grid = SpatialGrid::build(&elements, &bounds);
        let visible = crate::drawable::WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
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
            polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0),
            polygon(vec![(200, 0), (300, 0), (300, 100), (200, 100)], 2, 0),
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
}
