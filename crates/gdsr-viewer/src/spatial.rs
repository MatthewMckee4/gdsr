use std::collections::HashMap;

use gdsr::{DataType, Element, Layer};

use crate::drawable::{Drawable, WorldBBox};

const GRID_SIZE: usize = 256;

/// A single cell in the spatial grid, containing element indices that overlap it.
pub struct GridCell {
    /// Indices into the element slice passed to `SpatialGrid::build`.
    pub indices: Vec<u32>,
    /// Tight bounding box of all elements in this cell.
    pub bbox: WorldBBox,
    /// The most frequently occurring `(layer, data_type)` pair among elements in this cell.
    pub dominant_layer: (Layer, DataType),
}

/// A 256x256 uniform grid that maps world-space regions to element indices,
/// enabling fast visible-area queries and cell-level LOAD during rendering.
pub struct SpatialGrid {
    cells: Vec<Option<GridCell>>,
    world_min_x: f64,
    world_min_y: f64,
    cell_width: f64,
    cell_height: f64,
}

impl SpatialGrid {
    /// Builds the spatial grid from elements and their pre-computed bounding box.
    /// Each element is inserted into every grid cell its bbox overlaps.
    pub fn build(elements: &[Element], bounds: &WorldBBox) -> Self {
        let epsilon = 1e-12;
        let cell_width = (bounds.max_x - bounds.min_x + epsilon) / GRID_SIZE as f64;
        let cell_height = (bounds.max_y - bounds.min_y + epsilon) / GRID_SIZE as f64;

        let mut cells: Vec<Option<GridCell>> = (0..GRID_SIZE * GRID_SIZE).map(|_| None).collect();
        let mut layer_counts: HashMap<usize, HashMap<(Layer, DataType), u32>> = HashMap::new();

        for (i, element) in elements.iter().enumerate() {
            let Some(bbox) = element.world_bbox() else {
                continue;
            };

            let col_min = ((bbox.min_x - bounds.min_x) / cell_width) as usize;
            let col_max = ((bbox.max_x - bounds.min_x) / cell_width) as usize;
            let row_min = ((bbox.min_y - bounds.min_y) / cell_height) as usize;
            let row_max = ((bbox.max_y - bounds.min_y) / cell_height) as usize;
            let col_min = col_min.min(GRID_SIZE - 1);
            let col_max = col_max.min(GRID_SIZE - 1);
            let row_min = row_min.min(GRID_SIZE - 1);
            let row_max = row_max.min(GRID_SIZE - 1);

            let layer_keys = element.layer_keys();
            let layer = layer_keys
                .first()
                .copied()
                .unwrap_or((Layer::new(0), DataType::new(0)));
            for row in row_min..=row_max {
                for col in col_min..=col_max {
                    let cell_idx = row * GRID_SIZE + col;
                    let cell = cells[cell_idx].get_or_insert_with(|| GridCell {
                        indices: Vec::new(),
                        bbox: WorldBBox::new(f64::MAX, f64::MAX, f64::MIN, f64::MIN),
                        dominant_layer: (Layer::new(0), DataType::new(0)),
                    });

                    cell.indices.push(i as u32);
                    cell.bbox = cell.bbox.merge(&bbox);

                    *layer_counts
                        .entry(cell_idx)
                        .or_default()
                        .entry(layer)
                        .or_insert(0) += 1;
                }
            }
        }

        for (cell_idx, counts) in &layer_counts {
            if let Some(cell) = cells[*cell_idx].as_mut() {
                if let Some((&layer, _)) = counts.iter().max_by_key(|(_, count)| *count) {
                    cell.dominant_layer = layer;
                }
            }
        }

        Self {
            cells,
            world_min_x: bounds.min_x,
            world_min_y: bounds.min_y,
            cell_width,
            cell_height,
        }
    }

    /// Returns deduplicated element indices from the 3×3 neighbourhood of grid
    /// cells around the given world-space point, eliminating boundary misses.
    pub fn query_point<'a>(&self, wx: f64, wy: f64, buf: &'a mut Vec<u32>) -> &'a [u32] {
        buf.clear();
        let col = ((wx - self.world_min_x) / self.cell_width) as isize;
        let row = ((wy - self.world_min_y) / self.cell_height) as isize;
        let gs = GRID_SIZE as isize;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let r = row.saturating_add(dr);
                let c = col.saturating_add(dc);
                if r < 0 || c < 0 || r >= gs || c >= gs {
                    continue;
                }
                let idx = r as usize * GRID_SIZE + c as usize;
                if let Some(cell) = &self.cells[idx] {
                    buf.extend_from_slice(&cell.indices);
                }
            }
        }
        buf.sort_unstable();
        buf.dedup();
        buf
    }

    /// Returns an iterator over grid cells that overlap the given visible world-space rectangle.
    pub fn query_visible(&self, visible: &WorldBBox) -> impl Iterator<Item = &GridCell> {
        let col_min = ((visible.min_x - self.world_min_x) / self.cell_width).floor() as isize;
        let col_max = ((visible.max_x - self.world_min_x) / self.cell_width).ceil() as isize;
        let row_min = ((visible.min_y - self.world_min_y) / self.cell_height).floor() as isize;
        let row_max = ((visible.max_y - self.world_min_y) / self.cell_height).ceil() as isize;
        let col_min = col_min.saturating_sub(1);
        let col_max = col_max.saturating_add(1);
        let row_min = row_min.saturating_sub(1);
        let row_max = row_max.saturating_add(1);

        let col_min = col_min.clamp(0, GRID_SIZE as isize - 1) as usize;
        let col_max = col_max.clamp(0, GRID_SIZE as isize - 1) as usize;
        let row_min = row_min.clamp(0, GRID_SIZE as isize - 1) as usize;
        let row_max = row_max.clamp(0, GRID_SIZE as isize - 1) as usize;

        let grid_size = GRID_SIZE;
        let cells_ref = &self.cells;

        (row_min..=row_max).flat_map(move |row| {
            (col_min..=col_max).filter_map(move |col| cells_ref[row * grid_size + col].as_ref())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::helpers::*;

    /// Collects all unique element indices found across queried cells.
    fn query_element_indices(grid: &SpatialGrid, visible: &WorldBBox) -> Vec<u32> {
        let mut indices: Vec<u32> = grid
            .query_visible(visible)
            .flat_map(|c| c.indices.iter().copied())
            .collect();
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    #[test]
    fn build_empty() {
        let bounds = WorldBBox::new(0.0, 0.0, 1.0, 1.0);
        let grid = SpatialGrid::build(&[], &bounds);
        assert!(grid.cells.iter().all(Option::is_none));
    }

    #[test]
    fn build_single_element() {
        let poly = polygon(vec![(0, 0), (1000, 0), (1000, 1000)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9);
        let grid = SpatialGrid::build(&[poly], &bounds);

        let all = query_element_indices(
            &grid,
            &WorldBBox::new(0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9),
        );
        assert_eq!(all, vec![0]);

        for cell in grid.cells.iter().flatten() {
            assert!(cell.indices.contains(&0));
            assert_eq!(cell.dominant_layer, (Layer::new(1), DataType::new(0)));
        }
    }

    #[test]
    fn build_assigns_correct_cell() {
        let scale = 1e-9;
        let p1 = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

        let bounds = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], &bounds);

        let all = query_element_indices(
            &grid,
            &WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale),
        );
        assert_eq!(all, vec![0, 1]);

        for cell in grid.cells.iter().flatten() {
            assert!(
                !cell.indices.contains(&0) || !cell.indices.contains(&1),
                "small distant elements should not share cells"
            );
        }
    }

    #[test]
    fn query_returns_only_visible_elements() {
        let scale = 1e-9;
        let p1 = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

        let bounds = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], &bounds);

        let visible = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));
        assert!(!indices.contains(&1));
    }

    #[test]
    fn query_full_extent_returns_all() {
        let scale = 1e-9;
        let p1 = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

        let bounds = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], &bounds);

        let visible = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn query_finds_element_partially_overlapping_viewport() {
        let scale = 1e-9;
        let p = polygon(vec![(400, 400), (600, 400), (600, 600), (400, 600)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[p], &bounds);

        let visible = WorldBBox::new(0.0, 0.0, 500.0 * scale, 500.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert!(
            indices.contains(&0),
            "partially overlapping element must be found"
        );
    }

    #[test]
    fn dominant_layer_most_frequent() {
        let elems: Vec<Element> = vec![
            polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            polygon(vec![(0, 0), (10, 0), (10, 10)], 2, 0),
        ];

        let scale = 1e-9;
        let bounds = WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&elems, &bounds);

        for cell in grid.cells.iter().flatten() {
            assert_eq!(cell.dominant_layer, (Layer::new(5), DataType::new(0)));
        }
    }

    #[test]
    fn build_skips_references() {
        let reference = Element::Reference(gdsr::Reference::default());
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let scale = 1e-9;
        let bounds = WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&[reference, poly], &bounds);

        let all = query_element_indices(
            &grid,
            &WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale),
        );
        assert_eq!(all, vec![1]);
    }

    #[test]
    fn cell_bbox_is_tight() {
        let scale = 1e-9;
        let poly = polygon(vec![(100, 200), (300, 200), (300, 400), (100, 400)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);

        for cell in grid.cells.iter().flatten() {
            assert!((cell.bbox.min_x - 100.0 * scale).abs() < 1e-15);
            assert!((cell.bbox.min_y - 200.0 * scale).abs() < 1e-15);
            assert!((cell.bbox.max_x - 300.0 * scale).abs() < 1e-15);
            assert!((cell.bbox.max_y - 400.0 * scale).abs() < 1e-15);
        }
    }

    #[test]
    fn large_element_found_from_opposite_edge() {
        let scale = 1e-9;
        let p = polygon(vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[p], &bounds);

        let visible = WorldBBox::new(900.0 * scale, 900.0 * scale, 1000.0 * scale, 1000.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));

        let visible = WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));
    }

    #[test]
    fn query_point_returns_element_in_cell() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);

        let mut buf = Vec::new();
        let indices = grid.query_point(50.0 * scale, 50.0 * scale, &mut buf);
        assert!(indices.contains(&0));
    }

    #[test]
    fn query_point_misses_distant_element() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100), (0, 100)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);

        let mut buf = Vec::new();
        let indices = grid.query_point(900.0 * scale, 900.0 * scale, &mut buf);
        assert!(!indices.contains(&0));
    }

    #[test]
    fn query_point_outside_grid_returns_empty() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);

        let mut buf = Vec::new();
        assert!(grid.query_point(-1.0, -1.0, &mut buf).is_empty());
        assert!(grid.query_point(2.0, 2.0, &mut buf).is_empty());
    }

    #[test]
    fn layer_keys_polygon() {
        let elem = polygon(vec![(0, 0), (1, 0), (1, 1)], 5, 3);
        assert_eq!(elem.layer_keys(), vec![(Layer::new(5), DataType::new(3))]);
    }

    #[test]
    fn layer_keys_path() {
        let elem = path(vec![(0, 0), (1, 1)], 7, 2, None);
        assert_eq!(elem.layer_keys(), vec![(Layer::new(7), DataType::new(2))]);
    }

    #[test]
    fn layer_keys_text() {
        let elem = text("t", 0, 0, 4);
        assert_eq!(elem.layer_keys(), vec![(Layer::new(4), DataType::new(0))]);
    }

    #[test]
    fn layer_keys_reference_empty() {
        let elem = reference();
        assert!(elem.layer_keys().is_empty());
    }

    #[test]
    fn layer_keys_reference_with_element() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(10, 0),
                gdsr::Point::default_integer(10, 10),
            ],
            Layer::new(5),
            DataType::new(3),
        );
        let reference = Element::Reference(gdsr::Reference::new(poly));
        assert_eq!(
            reference.layer_keys(),
            vec![(Layer::new(5), DataType::new(3))]
        );
    }

    #[test]
    fn world_bbox_polygon() {
        let bbox = polygon(vec![(100, 200), (300, 400), (500, 100)], 1, 0)
            .world_bbox()
            .expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 100.0 * scale).abs() < 1e-15);
        assert!((bbox.min_y - 100.0 * scale).abs() < 1e-15);
        assert!((bbox.max_x - 500.0 * scale).abs() < 1e-15);
        assert!((bbox.max_y - 400.0 * scale).abs() < 1e-15);
    }

    #[test]
    fn world_bbox_path() {
        let bbox = path(vec![(10, 20), (30, 40)], 1, 0, Some(5))
            .world_bbox()
            .expect("should have bbox");
        let scale = 1e-9;
        let half_width = 2.5 * scale;
        assert!((bbox.min_x - (10.0 * scale - half_width)).abs() < 1e-15);
        assert!((bbox.min_y - (20.0 * scale - half_width)).abs() < 1e-15);
        assert!((bbox.max_x - (30.0 * scale + half_width)).abs() < 1e-15);
        assert!((bbox.max_y - (40.0 * scale + half_width)).abs() < 1e-15);
    }

    #[test]
    fn world_bbox_text() {
        let bbox = text("hello", 500, 600, 1)
            .world_bbox()
            .expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 500.0 * scale).abs() < 1e-15);
        assert!((bbox.min_y - 600.0 * scale).abs() < 1e-15);
        assert_eq!(bbox.min_x, bbox.max_x);
        assert_eq!(bbox.min_y, bbox.max_y);
    }

    #[test]
    fn world_bbox_reference_empty() {
        assert!(reference().world_bbox().is_none());
    }

    #[test]
    fn world_bbox_reference_with_element() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(100, 0),
                gdsr::Point::default_integer(100, 200),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let reference = Element::Reference(gdsr::Reference::new(poly));
        let bbox = reference.world_bbox().expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 0.0).abs() < 1e-15);
        assert!((bbox.min_y - 0.0).abs() < 1e-15);
        assert!((bbox.max_x - 100.0 * scale).abs() < 1e-15);
        assert!((bbox.max_y - 200.0 * scale).abs() < 1e-15);
    }

    #[test]
    fn build_degenerate_zero_area_bounds() {
        let poly = polygon(vec![(500, 500), (500, 500), (500, 500)], 1, 0);
        let bounds = WorldBBox::new(500e-9, 500e-9, 500e-9, 500e-9);
        let grid = SpatialGrid::build(&[poly], &bounds);
        let all = query_element_indices(&grid, &bounds);
        assert!(all.contains(&0));
    }

    #[test]
    fn build_many_elements_same_location() {
        let scale = 1e-9;
        let elems: Vec<Element> = (0..1000)
            .map(|_| polygon(vec![(0, 0), (10, 0), (10, 10)], 1, 0))
            .collect();
        let bounds = WorldBBox::new(0.0, 0.0, 10.0 * scale, 10.0 * scale);
        let grid = SpatialGrid::build(&elems, &bounds);
        let all = query_element_indices(&grid, &bounds);
        assert_eq!(all.len(), 1000);
    }

    #[test]
    fn build_element_spanning_entire_grid() {
        let scale = 1e-9;
        let large = polygon(vec![(0, 0), (10000, 0), (10000, 10000), (0, 10000)], 1, 0);
        let small = polygon(vec![(100, 100), (200, 100), (200, 200)], 2, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[large, small], &bounds);
        let visible = WorldBBox::new(50.0 * scale, 50.0 * scale, 250.0 * scale, 250.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0), "large spanning element must be found");
        assert!(
            indices.contains(&1),
            "small element in visible area must be found"
        );
    }

    #[test]
    fn query_point_at_grid_boundary() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);
        let mut buf = Vec::new();
        let indices = grid.query_point(0.0, 0.0, &mut buf);
        assert!(indices.contains(&0), "element at grid origin must be found");
    }

    #[test]
    fn query_visible_beyond_grid_extent() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);
        let huge_visible = WorldBBox::new(-1.0, -1.0, 1.0, 1.0);
        let indices = query_element_indices(&grid, &huge_visible);
        assert!(indices.contains(&0));
    }

    #[test]
    fn build_with_extreme_coordinates() {
        let max = i32::MAX / 2;
        let min = i32::MIN / 2;
        let scale = 1e-9;
        let p1 = polygon(
            vec![(min, min), (min + 100, min), (min + 100, min + 100)],
            1,
            0,
        );
        let p2 = polygon(
            vec![(max - 100, max - 100), (max, max - 100), (max, max)],
            2,
            0,
        );
        let bounds = WorldBBox::new(
            f64::from(min) * scale,
            f64::from(min) * scale,
            f64::from(max) * scale,
            f64::from(max) * scale,
        );
        let grid = SpatialGrid::build(&[p1, p2], &bounds);
        let all = query_element_indices(&grid, &bounds);
        assert_eq!(all, vec![0, 1]);
    }

    #[test]
    fn query_point_reuses_buffer() {
        let scale = 1e-9;
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], &bounds);
        let mut buf = Vec::new();
        let _ = grid.query_point(50.0 * scale, 50.0 * scale, &mut buf);
        let first_len = buf.len();
        let _ = grid.query_point(900.0 * scale, 900.0 * scale, &mut buf);
        assert!(
            buf.len() != first_len || first_len == 0,
            "buffer should be reused and cleared between queries"
        );
    }

    #[test]
    fn world_bbox_reference_with_grid() {
        let poly = gdsr::Polygon::new(
            vec![
                gdsr::Point::default_integer(0, 0),
                gdsr::Point::default_integer(10, 0),
                gdsr::Point::default_integer(10, 10),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = gdsr::Grid::default()
            .with_columns(2)
            .with_rows(1)
            .with_spacing_x(Some(gdsr::Point::default_integer(20, 0)));
        let reference = Element::Reference(gdsr::Reference::new(poly).with_grid(grid));
        let bbox = reference.world_bbox().expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 0.0).abs() < 1e-15);
        assert!((bbox.min_y - 0.0).abs() < 1e-15);
        // Second copy at x+20, so max_x should be (20+10)*scale = 30*scale
        assert!((bbox.max_x - 30.0 * scale).abs() < 1e-15);
        assert!((bbox.max_y - 10.0 * scale).abs() < 1e-15);
    }
}
