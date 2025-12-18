use crate::{Element, Grid, Library, Movable, Point, Transformable, Transformation};

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

    pub const fn instance(&self) -> &Instance {
        &self.instance
    }

    pub const fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn get_elements_in_grid(&self, element: &Element) -> Vec<Element> {
        let grid = self.grid();

        let mut elements: Vec<Element> =
            Vec::with_capacity((grid.columns() * grid.rows()) as usize);

        for column_index in 0..grid.columns() {
            let column_origin = grid.origin() + (grid.spacing_x() * column_index);
            for row_index in 0..grid.rows() {
                let origin = column_origin + (grid.spacing_y() * row_index);

                let mut new_element = element.clone();

                if grid.x_reflection() {
                    new_element = new_element.reflect(0.0, Point::integer(1, 0, 1e-9));
                }
                new_element = new_element.rotate(grid.angle(), Point::default());
                new_element = new_element.scale(grid.magnification(), Point::default());

                let move_point = origin.rotate_around_point(grid.angle(), &grid.origin());

                new_element = new_element.move_by(move_point);

                elements.push(new_element.clone());
            }
        }

        elements
    }

    pub fn flatten(self, depth: Option<usize>, library: &Library) -> Vec<Element> {
        let depth = depth.unwrap_or(usize::MAX);
        let mut elements: Vec<Element> = Vec::new();
        if depth == 0 {
            return [Element::Reference(self)].to_vec();
        }
        match &self.instance {
            Instance::Cell(cell_name) => {
                if let Some(cell) = library.get_cell(cell_name) {
                    let flattened_cell_elements = cell.get_elements(Some(depth - 1), library);
                    for cell_element in flattened_cell_elements {
                        elements.extend(self.get_elements_in_grid(&cell_element));
                    }
                }
            }
            Instance::Element(element) => match element.as_ref().as_ref() {
                Element::Path(_) | Element::Polygon(_) | Element::Text(_) => {
                    elements.extend(self.get_elements_in_grid(element));
                }

                Element::Reference(reference) => {
                    let flattened_reference_elements =
                        reference.clone().flatten(Some(depth - 1), library);

                    for reference_element in flattened_reference_elements {
                        elements.extend(self.get_elements_in_grid(&reference_element).into_iter());
                    }
                }
            },
        }

        elements
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Reference to {} with grid {}", self.instance, self.grid)
    }
}

impl Transformable for Reference {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.grid = new_self.grid.transform_impl(transformation);
        new_self
    }
}

impl Movable for Reference {
    fn move_to(&self, target: Point) -> Self {
        let mut new_self = self.clone();
        new_self.grid = new_self.grid.move_to(target);
        new_self
    }
}

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
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let grid = Grid::new(
            Point::integer(0, 0, 1e-9),
            2,
            2,
            Point::integer(10, 0, 1e-9),
            Point::integer(0, 10, 1e-9),
            1.0,
            0.0,
            false,
        );
        let reference = Reference::new(polygon, grid);

        assert_eq!(reference.grid().columns(), 2);
        assert_eq!(reference.grid().rows(), 2);
    }

    #[test]
    fn test_reference_default() {
        let reference = Reference::default();
        assert_eq!(reference.grid().columns(), 1);
        assert_eq!(reference.grid().rows(), 1);
    }

    #[test]
    fn test_reference_from_cell_name() {
        let grid = Grid::new(
            Point::integer(0, 0, 1e-9),
            1,
            1,
            Point::integer(0, 0, 1e-9),
            Point::integer(0, 0, 1e-9),
            1.0,
            0.0,
            false,
        );
        let reference = Reference::new("test_cell", grid);

        match reference.instance() {
            Instance::Cell(name) => assert_eq!(name, "test_cell"),
            Instance::Element(_) => panic!("Expected Cell instance"),
        }
    }

    #[test]
    fn test_reference_display() {
        let grid = Grid::new(
            Point::integer(0, 0, 1e-9),
            1,
            1,
            Point::integer(0, 0, 1e-9),
            Point::integer(0, 0, 1e-9),
            1.0,
            0.0,
            false,
        );
        let reference = Reference::new("test_cell", grid);

        let display_str = format!("{reference}");
        assert!(display_str.contains("Reference to"));
        assert!(display_str.contains("test_cell"));
    }

    #[test]
    fn test_reference_flatten() {
        let library = Library::new("main");
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let grid = Grid::new(
            Point::integer(0, 0, 1e-9),
            2,
            2,
            Point::integer(10, 0, 1e-9),
            Point::integer(0, 10, 1e-9),
            1.0,
            0.0,
            false,
        );
        let reference = Reference::new(polygon, grid);

        let flattened = reference.flatten(None, &library);
        assert_eq!(flattened.len(), 4);

        insta::assert_debug_snapshot!(flattened);
    }
}
