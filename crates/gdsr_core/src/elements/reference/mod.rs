use crate::{
    CoordNum, DatabaseIntegerUnit, Point, elements::Element, grid::Grid, traits::Transformable,
    transformation::Transformation, utils::general::point_to_database_float,
};

pub mod instance;
pub mod io;

pub use instance::Instance;

#[derive(Clone, Debug, PartialEq)]
pub struct Reference<T: CoordNum> {
    instance: Instance<T>,
    grid: Grid<T>,
}

impl<DatabaseUnitT: CoordNum> Reference<DatabaseUnitT> {
    pub fn new(instance: Instance<DatabaseUnitT>, grid: Grid<DatabaseUnitT>) -> Self {
        Self { instance, grid }
    }

    pub fn instance(&self) -> &Instance<DatabaseUnitT> {
        &self.instance
    }

    pub fn grid(&self) -> &Grid<DatabaseUnitT> {
        &self.grid
    }

    pub fn _get_elements_in_grid(
        &self,
        element: &Element<DatabaseUnitT>,
    ) -> Vec<Element<DatabaseUnitT>> {
        let grid = self.grid();

        let mut elements: Vec<Element<DatabaseUnitT>> =
            Vec::with_capacity((grid.columns * grid.rows) as usize);

        for column_index in 0..grid.columns {
            let column_origin = point_to_database_float(grid.origin)
                + (point_to_database_float(grid.spacing_x) * column_index as f64);
            for row_index in 0..grid.rows {
                let origin = point_to_database_float(column_origin)
                    + (point_to_database_float(grid.spacing_y) * row_index as f64);

                let mut new_element = element.clone();

                if grid.x_reflection {
                    new_element.reflect(0.0, Point::new(1.0, 0.0));
                }

                new_element.rotate(grid.angle, Point::default());
                new_element.scale(grid.magnification, Point::default());

                new_element.move_by(origin.rotate(grid.angle, grid.origin));

                elements.push(new_element.copy());
            }
        }

        elements
    }

    pub fn flatten(
        &mut self,
        layer_data_types: Vec<(i32, i32)>,
        depth: Option<usize>,
    ) -> Vec<Element<DatabaseUnitT>> {
        let depth = depth.unwrap_or(usize::MAX);
        let flatten_all = layer_data_types.is_empty();
        let mut elements: Vec<Element<DatabaseUnitT>> = Vec::new();
        if depth == 0 {
            return [Element::Reference(self.clone())].to_vec();
        }
        match &self.instance {
            Instance::Cell(cell) => {
                let flattened_cell_elements = cell.get_elements(layer_data_types, Some(depth - 1));
                for cell_element in flattened_cell_elements {
                    elements.extend(self._get_elements_in_grid(cell_element));
                }
            }
            Instance::Element(element) => match element.as_ref().as_ref() {
                Element::Path(_) | Element::Polygon(_) | Element::Text(_) => {
                    elements.extend(self._get_elements_in_grid(element));
                }

                Element::Reference(reference) => {
                    let flattened_reference_elements =
                        reference.flatten(layer_data_types, Some(depth - 1));

                    let flattened_copied_elements = flattened_reference_elements
                        .iter()
                        .map(|element| element.clone())
                        .collect::<Vec<Element<_>>>();

                    for reference_element in flattened_copied_elements {
                        elements.extend(self._get_elements_in_grid(&reference_element).into_iter());
                    }
                }
            },
        }

        elements
    }
}

impl Transformable for Reference<DatabaseIntegerUnit> {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.grid.transform(transformation);
        self
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
