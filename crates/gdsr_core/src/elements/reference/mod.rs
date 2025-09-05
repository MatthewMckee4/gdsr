use geo::Rotate;

use crate::{
    CoordNum, DatabaseIntegerUnit, Library, Movable, Point, elements::Element, grid::Grid,
    traits::Transformable, transformation::Transformation, utils::general::point_to_database_float,
};

pub mod instance;
pub mod io;

pub use instance::Instance;

#[derive(Clone, Debug, PartialEq)]
pub struct Reference<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) instance: Instance<DatabaseUnitT>,
    pub(crate) grid: Grid<DatabaseUnitT>,
}

impl<DatabaseUnitT: CoordNum> Default for Reference<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            instance: Instance::default(),
            grid: Grid::default(),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Reference<DatabaseUnitT> {
    pub fn new(instance: impl Into<Instance<DatabaseUnitT>>, grid: Grid<DatabaseUnitT>) -> Self {
        Self {
            instance: instance.into(),
            grid,
        }
    }

    pub const fn instance(&self) -> &Instance<DatabaseUnitT> {
        &self.instance
    }

    pub const fn grid(&self) -> &Grid<DatabaseUnitT> {
        &self.grid
    }

    pub fn get_elements_in_grid(
        &self,
        element: &Element<DatabaseUnitT>,
    ) -> Vec<Element<DatabaseUnitT>> {
        let grid = self.grid();

        let mut elements: Vec<Element<DatabaseUnitT>> =
            Vec::with_capacity((grid.columns * grid.rows) as usize);

        for column_index in 0..grid.columns {
            let column_origin = point_to_database_float(grid.origin)
                + (point_to_database_float(grid.spacing_x) * f64::from(column_index));
            for row_index in 0..grid.rows {
                let origin = point_to_database_float(column_origin)
                    + (point_to_database_float(grid.spacing_y) * f64::from(row_index));

                let mut new_element = element.clone();

                if grid.x_reflection {
                    new_element = new_element.reflect(0.0, Point::new(1, 0));
                }

                new_element = new_element.rotate(grid.angle, Point::default());
                new_element = new_element.scale(grid.magnification, Point::default());

                let move_point = origin.rotate_around_point(
                    grid.angle,
                    Point::new(grid.origin.x().to_float(), grid.origin.y().to_float()),
                );

                new_element = new_element.move_by(Point::new(
                    DatabaseIntegerUnit::from_float(move_point.x()),
                    DatabaseIntegerUnit::from_float(move_point.y()),
                ));

                elements.push(new_element.clone());
            }
        }

        elements
    }

    pub fn flatten(
        self,
        depth: Option<usize>,
        library: &Library<DatabaseUnitT>,
    ) -> Vec<Element<DatabaseUnitT>> {
        let depth = depth.unwrap_or(usize::MAX);
        let mut elements: Vec<Element<DatabaseUnitT>> = Vec::new();
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

impl<DatabaseUnitT: CoordNum> Transformable for Reference<DatabaseUnitT> {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.grid = new_self.grid.transform_impl(transformation);
        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Reference<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
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
