use crate::Grid;

pub mod instance;
mod io;

pub use instance::Instance;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Reference {
    pub(crate) instance: Instance,
    pub(crate) grid: Grid,
}

impl Reference {
    pub fn new(instance: impl Into<Instance>, grid: Grid) -> Self {
        Self {
            instance: instance.into(),
            grid,
        }
    }

    #[must_use]
    pub const fn instance(&self) -> &Instance {
        &self.instance
    }

    #[must_use]
    pub const fn grid(&self) -> &Grid {
        &self.grid
    }

    // TODO: Re-implement get_elements_in_grid() after transformation traits are updated
    // pub fn get_elements_in_grid(&self, element: &Element) -> Vec<Element> {
    //     ...
    // }

    // TODO: Re-implement flatten() after Library is updated
    // pub fn flatten(self, depth: Option<usize>, library: &Library) -> Vec<Element> {
    //     ...
    // }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Reference to {} with grid {}", self.instance, self.grid)
    }
}

// TODO: Re-implement Transformable trait for Reference once traits module is updated
// impl Transformable for Reference {
//     fn transform_impl(&self, transformation: &Transformation) -> Self {
//         let mut new_self = self.clone();
//         new_self.grid = new_self.grid.transform_impl(transformation);
//         new_self
//     }
// }

// TODO: Re-implement Movable trait for Reference once traits module is updated
// impl Movable for Reference {
//     fn move_to(&self, target: Point) -> Self {
//         let mut new_self = self.clone();
//         new_self.grid = new_self.grid.move_to(target);
//         new_self
//     }
// }

// impl<T: CoordNum> Dimensions<T> for Reference<T> {
//     fn bounding_box(&self) -> (Point<T>, Point<T>) {
//         let mut min_x = f64::INFINITY;
//         let mut min_y = f64::INFINITY;
//         let mut max_x = f64::NEG_INFINITY;
//         let mut max_y = f64::NEG_INFINITY;

//         let grid = &self.grid;

//         let corners = vec![
//             grid.origin,
//             grid.origin + grid.spacing_x * (grid.columns as f64).into(),
//             grid.origin + grid.spacing_y * (grid.rows as f64).into(),
//             grid.origin
//                 + grid.spacing_x * (grid.columns as f64).into()
//                 + grid.spacing_y * (grid.rows as f64).into(),
//         ];

//         for corner in corners {
//             let new_instance = self.instance.clone();

//             let mut transformation = Transformation::default();
//             transformation = transformation
//                 .with_scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin)
//                 .with_scale(grid.magnification, grid.origin)
//                 .with_rotation(grid.angle, grid.origin)
//                 .with_translation(Point::new(
//                     corner.x() - grid.origin.x(),
//                     corner.y() - grid.origin.y(),
//                 ));

//             let mut grid = Grid::default();

//             grid.transform(&transformation);

//             let reference = Reference::new(new_instance, grid);

//             let (new_instance_min, new_instance_max) = reference.bounding_box();

//             min_x = min_x.min(new_instance_min.x().into());
//             min_y = min_y.min(new_instance_min.y().into());
//             max_x = max_x.max(new_instance_max.x().into());
//             max_y = max_y.max(new_instance_max.y().into());
//         }

//         (
//             Point::new(min_x.into(), min_y.into()),
//             Point::new(max_x.into(), max_y.into()),
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Polygon;

    #[test]
    fn test_reference_new() {
        let polygon = Polygon::new([(0, 0), (10, 0), (10, 10)], 1, 0);
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);
        let reference = Reference::new(polygon, grid);

        assert_eq!(reference.grid().columns, 2);
        assert_eq!(reference.grid().rows, 2);
    }

    #[test]
    fn test_reference_default() {
        let reference = Reference::default();
        assert_eq!(reference.grid().columns, 1);
        assert_eq!(reference.grid().rows, 1);
    }

    #[test]
    fn test_reference_from_cell_name() {
        let grid = Grid::new((0, 0), 1, 1, (0, 0), (0, 0), 1.0, 0.0, false);
        let reference = Reference::new("test_cell", grid);

        match reference.instance() {
            Instance::Cell(name) => assert_eq!(name, "test_cell"),
            Instance::Element(_) => panic!("Expected Cell instance"),
        }
    }

    #[test]
    fn test_reference_display() {
        let grid = Grid::new((0, 0), 1, 1, (0, 0), (0, 0), 1.0, 0.0, false);
        let reference = Reference::new("test_cell", grid);

        let display_str = format!("{reference}");
        assert!(display_str.contains("Reference to"));
        assert!(display_str.contains("test_cell"));
    }

    #[test]
    fn test_reference_clone() {
        let polygon = Polygon::new([(0, 0), (10, 0), (10, 10)], 1, 0);
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);
        let reference = Reference::new(polygon, grid);

        let cloned = reference.clone();
        assert_eq!(reference, cloned);
    }
}
