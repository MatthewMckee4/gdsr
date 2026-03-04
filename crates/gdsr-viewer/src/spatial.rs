use std::collections::HashMap;

use gdsr::Element;

use crate::viewport::element_bbox;

const GRID_SIZE: usize = 256;

pub struct GridCell {
    pub indices: Vec<u32>,
    /// Tight bounding box: `[min_x, min_y, max_x, max_y]`
    pub bbox: [f64; 4],
    pub dominant_layer: (u16, u16),
}

pub struct SpatialGrid {
    cells: Vec<Option<GridCell>>,
    world_min_x: f64,
    world_min_y: f64,
    cell_width: f64,
    cell_height: f64,
}

fn element_layer(element: &Element) -> (u16, u16) {
    match element {
        Element::Polygon(p) => (p.layer(), p.data_type()),
        Element::Path(p) => (p.layer(), p.data_type()),
        Element::Text(t) => (t.layer(), 0),
        Element::Reference(_) => (0, 0),
    }
}

impl SpatialGrid {
    pub fn build(elements: &[Element], bounds: (f64, f64, f64, f64)) -> Self {
        let (min_x, min_y, max_x, max_y) = bounds;
        let epsilon = 1e-12;
        let cell_width = (max_x - min_x + epsilon) / GRID_SIZE as f64;
        let cell_height = (max_y - min_y + epsilon) / GRID_SIZE as f64;

        let mut cells: Vec<Option<GridCell>> = (0..GRID_SIZE * GRID_SIZE).map(|_| None).collect();
        let mut layer_counts: HashMap<usize, HashMap<(u16, u16), u32>> = HashMap::new();

        for (i, element) in elements.iter().enumerate() {
            let Some(bbox) = element_bbox(element) else {
                continue;
            };

            // Insert into every grid cell the element's bbox overlaps
            let col_min = ((bbox[0] - min_x) / cell_width) as usize;
            let col_max = ((bbox[2] - min_x) / cell_width) as usize;
            let row_min = ((bbox[1] - min_y) / cell_height) as usize;
            let row_max = ((bbox[3] - min_y) / cell_height) as usize;
            let col_min = col_min.min(GRID_SIZE - 1);
            let col_max = col_max.min(GRID_SIZE - 1);
            let row_min = row_min.min(GRID_SIZE - 1);
            let row_max = row_max.min(GRID_SIZE - 1);

            let layer = element_layer(element);
            for row in row_min..=row_max {
                for col in col_min..=col_max {
                    let cell_idx = row * GRID_SIZE + col;
                    let cell = cells[cell_idx].get_or_insert_with(|| GridCell {
                        indices: Vec::new(),
                        bbox: [f64::MAX, f64::MAX, f64::MIN, f64::MIN],
                        dominant_layer: (0, 0),
                    });

                    cell.indices.push(i as u32);
                    cell.bbox[0] = cell.bbox[0].min(bbox[0]);
                    cell.bbox[1] = cell.bbox[1].min(bbox[1]);
                    cell.bbox[2] = cell.bbox[2].max(bbox[2]);
                    cell.bbox[3] = cell.bbox[3].max(bbox[3]);

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
            world_min_x: min_x,
            world_min_y: min_y,
            cell_width,
            cell_height,
        }
    }

    pub fn query_visible(&self, visible: &[f64; 4]) -> impl Iterator<Item = &GridCell> {
        let col_min = ((visible[0] - self.world_min_x) / self.cell_width).floor() as isize - 1;
        let col_max = ((visible[2] - self.world_min_x) / self.cell_width).ceil() as isize + 1;
        let row_min = ((visible[1] - self.world_min_y) / self.cell_height).floor() as isize - 1;
        let row_max = ((visible[3] - self.world_min_y) / self.cell_height).ceil() as isize + 1;

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
        let grid = SpatialGrid::build(&[], (0.0, 0.0, 1.0, 1.0));
        assert!(grid.cells.iter().all(Option::is_none));
    }

    /// Collects all unique element indices found across queried cells.
    fn query_element_indices(grid: &SpatialGrid, visible: &[f64; 4]) -> Vec<u32> {
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
        let bounds = (0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9);
        let grid = SpatialGrid::build(&[poly], bounds);

        // Element should be findable via full-extent query
        let all = query_element_indices(&grid, &[0.0, 0.0, 1000.0 * 1e-9, 1000.0 * 1e-9]);
        assert_eq!(all, vec![0]);

        // Every non-empty cell should reference element 0 with layer (1, 0)
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

        let bounds = (0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], bounds);

        // Both elements should be findable
        let all = query_element_indices(&grid, &[0.0, 0.0, 10000.0 * scale, 10000.0 * scale]);
        assert_eq!(all, vec![0, 1]);

        // They should not share any cells (far apart, small relative to grid)
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

        let bounds = (0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], bounds);

        // Query only the bottom-left corner — should find p1 but not p2
        let visible = [0.0, 0.0, 1000.0 * scale, 1000.0 * scale];
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));
        assert!(!indices.contains(&1));
    }

    #[test]
    fn query_full_extent_returns_all() {
        let scale = 1e-9;
        let p1 = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let p2 = make_polygon(vec![(9900, 9900), (10000, 9900), (10000, 10000)], 2, 0);

        let bounds = (0.0, 0.0, 10000.0 * scale, 10000.0 * scale);
        let grid = SpatialGrid::build(&[p1, p2], bounds);

        let visible = [0.0, 0.0, 10000.0 * scale, 10000.0 * scale];
        let indices = query_element_indices(&grid, &visible);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn query_finds_element_partially_overlapping_viewport() {
        let scale = 1e-9;
        // Element spans from 400..600 in a 0..1000 world
        let p = make_polygon(vec![(400, 400), (600, 400), (600, 600), (400, 600)], 1, 0);
        let bounds = (0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[p], bounds);

        // Viewport covers 0..500 — partially overlaps the element
        let visible = [0.0, 0.0, 500.0 * scale, 500.0 * scale];
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
        let bounds = (0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&elems, bounds);

        // All 4 elements overlap the same region; every cell containing them should
        // have dominant layer (5, 0) since 3 of 4 elements are on that layer.
        for cell in grid.cells.iter().flatten() {
            assert_eq!(cell.dominant_layer, (5, 0));
        }
    }

    #[test]
    fn build_skips_references() {
        let reference = Element::Reference(gdsr::Reference::default());
        let poly = make_polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let scale = 1e-9;
        let bounds = (0.0, 0.0, 100.0 * scale, 100.0 * scale);
        let grid = SpatialGrid::build(&[reference, poly], bounds);

        let all = query_element_indices(&grid, &[0.0, 0.0, 100.0 * scale, 100.0 * scale]);
        // Only the polygon (index 1) should appear; reference (index 0) is skipped
        assert_eq!(all, vec![1]);
    }

    #[test]
    fn cell_bbox_is_tight() {
        let scale = 1e-9;
        let poly = make_polygon(vec![(100, 200), (300, 200), (300, 400), (100, 400)], 1, 0);
        let bounds = (0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[poly], bounds);

        for cell in grid.cells.iter().flatten() {
            assert!((cell.bbox[0] - 100.0 * scale).abs() < 1e-15);
            assert!((cell.bbox[1] - 200.0 * scale).abs() < 1e-15);
            assert!((cell.bbox[2] - 300.0 * scale).abs() < 1e-15);
            assert!((cell.bbox[3] - 400.0 * scale).abs() < 1e-15);
        }
    }

    #[test]
    fn large_element_found_from_opposite_edge() {
        let scale = 1e-9;
        // Element spans the entire world
        let p = make_polygon(vec![(0, 0), (1000, 0), (1000, 1000), (0, 1000)], 1, 0);
        let bounds = (0.0, 0.0, 1000.0 * scale, 1000.0 * scale);
        let grid = SpatialGrid::build(&[p], bounds);

        // Query just the top-right corner
        let visible = [900.0 * scale, 900.0 * scale, 1000.0 * scale, 1000.0 * scale];
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));

        // Query just the bottom-left corner
        let visible = [0.0, 0.0, 100.0 * scale, 100.0 * scale];
        let indices = query_element_indices(&grid, &visible);
        assert!(indices.contains(&0));
    }

    #[test]
    fn element_layer_polygon() {
        let poly = Polygon::new(
            vec![
                Point::default_integer(0, 0),
                Point::default_integer(1, 0),
                Point::default_integer(1, 1),
            ],
            5,
            3,
        );
        assert_eq!(element_layer(&Element::Polygon(poly)), (5, 3));
    }

    #[test]
    fn element_layer_path() {
        let path = Path::new(
            vec![Point::default_integer(0, 0), Point::default_integer(1, 1)],
            7,
            2,
            None,
            None,
        );
        assert_eq!(element_layer(&Element::Path(path)), (7, 2));
    }

    #[test]
    fn element_layer_text() {
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
        assert_eq!(element_layer(&Element::Text(text)), (4, 0));
    }

    #[test]
    fn element_layer_reference() {
        let reference = Element::Reference(gdsr::Reference::default());
        assert_eq!(element_layer(&reference), (0, 0));
    }

    #[test]
    fn element_bbox_polygon() {
        let poly = Polygon::new(
            vec![
                Point::default_integer(100, 200),
                Point::default_integer(300, 400),
                Point::default_integer(500, 100),
            ],
            1,
            0,
        );
        let bbox = element_bbox(&Element::Polygon(poly)).expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox[0] - 100.0 * scale).abs() < 1e-15);
        assert!((bbox[1] - 100.0 * scale).abs() < 1e-15);
        assert!((bbox[2] - 500.0 * scale).abs() < 1e-15);
        assert!((bbox[3] - 400.0 * scale).abs() < 1e-15);
    }

    #[test]
    fn element_bbox_path() {
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
        let bbox = element_bbox(&Element::Path(path)).expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox[0] - 10.0 * scale).abs() < 1e-15);
        assert!((bbox[1] - 20.0 * scale).abs() < 1e-15);
        assert!((bbox[2] - 30.0 * scale).abs() < 1e-15);
        assert!((bbox[3] - 40.0 * scale).abs() < 1e-15);
    }

    #[test]
    fn element_bbox_text() {
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
        let bbox = element_bbox(&Element::Text(text)).expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox[0] - 500.0 * scale).abs() < 1e-15);
        assert!((bbox[1] - 600.0 * scale).abs() < 1e-15);
        assert_eq!(bbox[0], bbox[2]);
        assert_eq!(bbox[1], bbox[3]);
    }

    #[test]
    fn element_bbox_reference() {
        let reference = Element::Reference(gdsr::Reference::default());
        assert!(element_bbox(&reference).is_none());
    }
}
