use std::collections::HashMap;

use gdsr::Element;

use crate::drawable::{Drawable, WorldBBox};

const GRID_SIZE: usize = 256;

pub struct GridCell {
    pub indices: Vec<u32>,
    /// Tight bounding box of all elements in this cell.
    pub bbox: WorldBBox,
    pub dominant_layer: (u16, u16),
}

pub struct SpatialGrid {
    cells: Vec<Option<GridCell>>,
    world_min_x: f64,
    world_min_y: f64,
    cell_width: f64,
    cell_height: f64,
}

impl SpatialGrid {
    pub fn build(elements: &[Element], bounds: &WorldBBox) -> Self {
        let epsilon = 1e-12;
        let cell_width = (bounds.max_x - bounds.min_x + epsilon) / GRID_SIZE as f64;
        let cell_height = (bounds.max_y - bounds.min_y + epsilon) / GRID_SIZE as f64;

        let mut cells: Vec<Option<GridCell>> = (0..GRID_SIZE * GRID_SIZE).map(|_| None).collect();
        let mut layer_counts: HashMap<usize, HashMap<(u16, u16), u32>> = HashMap::new();

        for (i, element) in elements.iter().enumerate() {
            let Some(bbox) = element.world_bbox() else {
                continue;
            };

            // Insert into every grid cell the element's bbox overlaps
            let col_min = ((bbox.min_x - bounds.min_x) / cell_width) as usize;
            let col_max = ((bbox.max_x - bounds.min_x) / cell_width) as usize;
            let row_min = ((bbox.min_y - bounds.min_y) / cell_height) as usize;
            let row_max = ((bbox.max_y - bounds.min_y) / cell_height) as usize;
            let col_min = col_min.min(GRID_SIZE - 1);
            let col_max = col_max.min(GRID_SIZE - 1);
            let row_min = row_min.min(GRID_SIZE - 1);
            let row_max = row_max.min(GRID_SIZE - 1);

            let layer = element.layer_key();
            for row in row_min..=row_max {
                for col in col_min..=col_max {
                    let cell_idx = row * GRID_SIZE + col;
                    let cell = cells[cell_idx].get_or_insert_with(|| GridCell {
                        indices: Vec::new(),
                        bbox: WorldBBox::new(f64::MAX, f64::MAX, f64::MIN, f64::MIN),
                        dominant_layer: (0, 0),
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

        // Set dominant layer for each cell
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

    pub fn query_visible(&self, visible: &WorldBBox) -> impl Iterator<Item = &GridCell> {
        let col_min = ((visible.min_x - self.world_min_x) / self.cell_width).floor() as isize - 1;
        let col_max = ((visible.max_x - self.world_min_x) / self.cell_width).ceil() as isize + 1;
        let row_min = ((visible.min_y - self.world_min_y) / self.cell_height).floor() as isize - 1;
        let row_max = ((visible.max_y - self.world_min_y) / self.cell_height).ceil() as isize + 1;

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
    use crate::drawable::Drawable;
    use gdsr::{HorizontalPresentation, Path, Point, Polygon, Text, Unit, VerticalPresentation};

    fn make_polygon(points: Vec<(i32, i32)>, layer: u16, data_type: u16) -> Element {
        Element::Polygon(Polygon::new(
            points
                .into_iter()
                .map(|(x, y)| Point::default_integer(x, y)),
            layer,
            data_type,
        ))
    }

    #[test]
    fn build_empty() {
        let grid = SpatialGrid::build(&[], &WorldBBox::new(0.0, 0.0, 1.0, 1.0));
        assert!(grid.cells.iter().all(Option::is_none));
    }

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
    fn build_single_element() {
        let poly = make_polygon(vec![(0, 0), (1000, 0), (1000, 1000)], 1, 0);
        let bounds = WorldBBox::new(0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9);
        let grid = SpatialGrid::build(&[poly], &bounds);

        let all = query_element_indices(
            &grid,
            &WorldBBox::new(0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9),
        );
        assert_eq!(all, vec![0]);

        for cell in grid.cells.iter().flatten() {
            assert!(cell.indices.contains(&0));
            assert_eq!(cell.dominant_layer, (1, 0));
        }
    }

    #[test]
    fn build_assigns_correct_cell() {
        let scale = 1e-9;
        let p1 = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = make_polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

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
        let p1 = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = make_polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

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
        let p1 = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = make_polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

        let bounds = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], &bounds);

        let visible = WorldBBox::new(0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let indices = query_element_indices(&grid, &visible);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn query_finds_element_partially_overlapping_viewport() {
        let scale = 1e-9;
        let p = make_polygon(vec![(400, 400), (600, 400), (600, 600), (400, 600)], 1, 0);
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
            make_polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            make_polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            make_polygon(vec![(0, 0), (10, 0), (10, 10)], 5, 0),
            make_polygon(vec![(0, 0), (10, 0), (10, 10)], 2, 0),
        ];

        let scale = 1e-9;
        let bounds = WorldBBox::new(0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&elems, &bounds);

        for cell in grid.cells.iter().flatten() {
            assert_eq!(cell.dominant_layer, (5, 0));
        }
    }

    #[test]
    fn build_skips_references() {
        let reference = Element::Reference(gdsr::Reference::default());
        let poly = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
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
        let poly = make_polygon(vec![(100, 200), (300, 200), (300, 400), (100, 400)], 1, 0);
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
        let p = make_polygon(vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000)], 1, 0);
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
    fn element_layer_key_polygon() {
        let poly = Polygon::new(
            vec![
                Point::default_integer(0, 0),
                Point::default_integer(1, 0),
                Point::default_integer(1, 1),
            ],
            5,
            3,
        );
        assert_eq!(Element::Polygon(poly).layer_key(), (5, 3));
    }

    #[test]
    fn element_layer_key_path() {
        let path = Path::new(
            vec![Point::default_integer(0, 0), Point::default_integer(1, 1)],
            7,
            2,
            None,
            None,
        );
        assert_eq!(Element::Path(path).layer_key(), (7, 2));
    }

    #[test]
    fn element_layer_key_text() {
        let text = Text::new(
            "t",
            Point::default_integer(0, 0),
            4,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        );
        assert_eq!(Element::Text(text).layer_key(), (4, 0));
    }

    #[test]
    fn element_layer_key_reference() {
        let reference = Element::Reference(gdsr::Reference::default());
        assert_eq!(reference.layer_key(), (0, 0));
    }

    #[test]
    fn world_bbox_polygon() {
        let poly = Polygon::new(
            vec![
                Point::default_integer(100, 200),
                Point::default_integer(300, 400),
                Point::default_integer(500, 100),
            ],
            1,
            0,
        );
        let bbox = Element::Polygon(poly)
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
        let path = Path::new(
            vec![
                Point::default_integer(10, 20),
                Point::default_integer(30, 40),
            ],
            1,
            0,
            None,
            Some(Unit::default_integer(5)),
        );
        let bbox = Element::Path(path).world_bbox().expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 10.0 * scale).abs() < 1e-15);
        assert!((bbox.min_y - 20.0 * scale).abs() < 1e-15);
        assert!((bbox.max_x - 30.0 * scale).abs() < 1e-15);
        assert!((bbox.max_y - 40.0 * scale).abs() < 1e-15);
    }

    #[test]
    fn world_bbox_text() {
        let text = Text::new(
            "hello",
            Point::default_integer(500, 600),
            1,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        );
        let bbox = Element::Text(text).world_bbox().expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox.min_x - 500.0 * scale).abs() < 1e-15);
        assert!((bbox.min_y - 600.0 * scale).abs() < 1e-15);
        assert_eq!(bbox.min_x, bbox.max_x);
        assert_eq!(bbox.min_y, bbox.max_y);
    }

    #[test]
    fn world_bbox_reference() {
        let reference = Element::Reference(gdsr::Reference::default());
        assert!(reference.world_bbox().is_none());
    }
}
