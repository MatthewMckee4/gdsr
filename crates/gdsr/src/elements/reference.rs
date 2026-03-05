use std::sync::Arc;
use std::sync::mpsc;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::elements::Element;
use crate::error::GdsError;
use crate::traits::{Movable, ToGds, Transformable};
use crate::utils::io::{
    validate_col_row, write_element_tail_to_file, write_points_to_file,
    write_string_with_record_to_file, write_transformation_to_file, write_u16_array_to_file,
};
use crate::{Cell, Grid, Library, Point, Transformation};

/// The target of a [`Reference`](crate::Reference): either a cell name or an inline element.
#[derive(Clone, Debug, PartialEq)]
pub enum Instance {
    /// A reference to a named cell in the library.
    Cell(String),
    /// An inline element stored directly in the reference.
    Element(Arc<Box<Element>>),
}

impl Instance {
    /// Returns the cell name if this is a `Cell` instance, or `None`.
    pub fn as_cell(&self) -> Option<&String> {
        if let Self::Cell(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the element if this is an `Element` instance, or `None`.
    pub fn as_element(&self) -> Option<&Arc<Box<Element>>> {
        if let Self::Element(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::Cell(String::new())
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cell(name) => write!(f, "Cell instance: {name}"),
            Self::Element(element) => write!(f, "Element instance: {element}"),
        }
    }
}

// From implementations for common element types
impl From<&Cell> for Instance {
    fn from(value: &Cell) -> Self {
        Self::Cell(value.name().to_string())
    }
}

impl From<String> for Instance {
    fn from(value: String) -> Self {
        Self::Cell(value)
    }
}

impl From<&str> for Instance {
    fn from(value: &str) -> Self {
        Self::Cell(value.to_string())
    }
}

/// A reference to an instance (cell or element) placed with a grid layout.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Reference {
    pub(crate) instance: Instance,
    pub(crate) grid: Grid,
}

impl Reference {
    /// Creates a new reference to the given instance with a default grid.
    pub fn new(instance: impl Into<Instance>) -> Self {
        Self {
            instance: instance.into(),
            grid: Grid::default(),
        }
    }

    /// Returns the referenced instance.
    pub const fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Returns the grid layout configuration.
    pub const fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Sets the grid layout and returns the modified reference.
    #[must_use]
    pub const fn with_grid(mut self, grid: Grid) -> Self {
        self.grid = grid;
        self
    }

    /// Converts grid to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            grid: self.grid.to_integer_unit(),
            ..self
        }
    }

    /// Converts grid to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            grid: self.grid.to_float_unit(),
            ..self
        }
    }

    /// Expands a single element across the grid, returning one element per grid position.
    pub fn get_elements_in_grid(&self, element: &Element) -> Vec<Element> {
        let grid = self.grid();

        let mut elements: Vec<Element> =
            Vec::with_capacity((grid.columns() * grid.rows()) as usize);

        let spacing_x = grid.spacing_x().unwrap_or_default();
        let spacing_y = grid.spacing_y().unwrap_or_default();

        for column_index in 0..grid.columns() {
            for row_index in 0..grid.rows() {
                // Calculate offset from grid origin
                let offset = (spacing_x * column_index) + (spacing_y * row_index);

                // Rotate offset if grid is rotated
                let rotated_offset = offset.rotate_around_point(grid.angle(), &Point::default());

                // Calculate final position for this instance
                let final_position = grid.origin() + rotated_offset;

                let mut new_element = element.clone();

                // Apply transformations around grid origin
                if grid.x_reflection() {
                    new_element = new_element.reflect(0.0, grid.origin());
                }

                new_element = new_element.rotate(grid.angle(), grid.origin());
                new_element = new_element.scale(grid.magnification(), grid.origin());

                // Move element to final position
                new_element = new_element.move_by(final_position - grid.origin());

                elements.push(new_element);
            }
        }

        elements
    }

    /// Sends each element in the grid through the channel. Returns `Err(())` if the receiver
    /// has been dropped.
    fn send_elements_in_grid(
        &self,
        element: &Element,
        tx: &mpsc::Sender<Element>,
    ) -> Result<(), ()> {
        for el in self.get_elements_in_grid(element) {
            tx.send(el).map_err(|_| ())?;
        }
        Ok(())
    }

    /// Like [`flatten`](Self::flatten) but sends elements through a channel as they're produced,
    /// enabling progressive rendering. Returns `Err(())` if the receiver is dropped.
    #[expect(
        clippy::result_unit_err,
        reason = "() signals cancellation via dropped receiver"
    )]
    pub fn stream_flatten(
        self,
        depth: Option<usize>,
        library: &Library,
        tx: &mpsc::Sender<Element>,
    ) -> Result<(), ()> {
        let depth = depth.unwrap_or(usize::MAX);
        if depth == 0 {
            tx.send(Element::Reference(self)).map_err(|_| ())?;
            return Ok(());
        }
        match &self.instance {
            Instance::Cell(cell_name) => {
                if let Some(cell) = library.get_cell(cell_name) {
                    for polygon in cell.polygons() {
                        self.send_elements_in_grid(&Element::Polygon(polygon.clone()), tx)?;
                    }
                    for path in cell.paths() {
                        self.send_elements_in_grid(&Element::Path(path.clone()), tx)?;
                    }
                    for text in cell.texts() {
                        self.send_elements_in_grid(&Element::Text(text.clone()), tx)?;
                    }
                    for reference in cell.references() {
                        for grid_el in
                            self.get_elements_in_grid(&Element::Reference(reference.clone()))
                        {
                            if let Element::Reference(grid_ref) = grid_el {
                                grid_ref.stream_flatten(Some(depth - 1), library, tx)?;
                            }
                        }
                    }
                }
            }
            Instance::Element(element) => match element.as_ref().as_ref() {
                Element::Path(_) | Element::Polygon(_) | Element::Text(_) => {
                    self.send_elements_in_grid(element, tx)?;
                }
                Element::Reference(reference) => {
                    for grid_el in self.get_elements_in_grid(&Element::Reference(reference.clone()))
                    {
                        if let Element::Reference(grid_ref) = grid_el {
                            grid_ref.stream_flatten(Some(depth - 1), library, tx)?;
                        }
                    }
                }
            },
        }
        Ok(())
    }

    /// Recursively flattens this reference into concrete elements, resolving cell references
    /// up to the given depth (or fully if `None`).
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
    fn transform_impl(mut self, transformation: &Transformation) -> Self {
        self.grid = self.grid.transform_impl(transformation);
        self
    }
}

impl Movable for Reference {
    fn move_to(mut self, target: Point) -> Self {
        self.grid = self.grid.move_to(target);
        self
    }
}

impl ToGds for Reference {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        match &self.instance {
            Instance::Cell(cell_name) => self.to_gds_impl_with_cell(database_units, cell_name),
            Instance::Element(element) => {
                self.to_gds_impl_with_element(database_units, element.as_ref().as_ref())
            }
        }
    }
}

impl Reference {
    fn to_gds_impl_with_element(
        &self,
        database_units: f64,
        element: &Element,
    ) -> Result<Vec<u8>, GdsError> {
        let grid = self.grid();
        let spacing_x = grid.spacing_x().unwrap_or_default();
        let spacing_y = grid.spacing_y().unwrap_or_default();

        let mut buf = Vec::new();
        for column_index in 0..grid.columns() {
            for row_index in 0..grid.rows() {
                let offset = (spacing_x * column_index) + (spacing_y * row_index);
                let rotated_offset =
                    offset.rotate_around_point(grid.angle(), &crate::Point::default());
                let final_position = grid.origin() + rotated_offset;

                let mut new_element = element.clone();
                if grid.x_reflection() {
                    new_element = new_element.reflect(0.0, grid.origin());
                }
                new_element = new_element.rotate(grid.angle(), grid.origin());
                new_element = new_element.scale(grid.magnification(), grid.origin());
                new_element = new_element.move_by(final_position - grid.origin());

                buf.extend_from_slice(&new_element.to_gds_impl(database_units)?);
            }
        }
        Ok(buf)
    }

    fn to_gds_impl_with_cell(
        &self,
        database_units: f64,
        cell_name: &str,
    ) -> Result<Vec<u8>, GdsError> {
        validate_col_row(self.grid().columns(), self.grid().rows())?;

        let is_single_instance = self.grid().columns() == 1 && self.grid().rows() == 1;

        let record = if is_single_instance {
            GDSRecord::SRef
        } else {
            GDSRecord::ARef
        };

        let mut buffer = Vec::new();

        let buffer_start = [4, combine_record_and_data_type(record, GDSDataType::NoData)];

        write_u16_array_to_file(&mut buffer, &buffer_start)?;

        write_string_with_record_to_file(&mut buffer, GDSRecord::SName, cell_name)?;

        let angle = self.grid().angle();
        let magnification = self.grid().magnification();
        let x_reflection = self.grid().x_reflection();

        write_transformation_to_file(&mut buffer, angle, magnification, x_reflection)?;

        if is_single_instance {
            let origin = self.grid().origin();
            let reference_points = [origin];
            write_points_to_file(&mut buffer, &reference_points, database_units)?;
        } else {
            let buffer_array = [
                8,
                combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
                self.grid().columns() as u16,
                self.grid().rows() as u16,
            ];

            write_u16_array_to_file(&mut buffer, &buffer_array)?;

            let origin = self
                .grid()
                .origin()
                .rotate_around_point(self.grid().angle(), &self.grid().origin());

            match (self.grid.spacing_x(), self.grid.spacing_y()) {
                (Some(spacing_x), Some(spacing_y)) => {
                    let point2 = ((origin + spacing_x) * self.grid().columns())
                        .rotate_around_point(self.grid().angle(), &origin);

                    let point3 = ((origin + spacing_y) * self.grid().rows())
                        .rotate_around_point(self.grid().angle(), &origin);

                    let reference_points = [origin, point2, point3];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                (Some(spacing_x), None) => {
                    let point2 = ((origin + spacing_x) * self.grid().columns())
                        .rotate_around_point(self.grid().angle(), &origin);
                    let reference_points = [origin, point2, origin];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                (None, Some(spacing_y)) => {
                    let point3 = ((origin + spacing_y) * self.grid().rows())
                        .rotate_around_point(self.grid().angle(), &origin);
                    let reference_points = [origin, origin, point3];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                _ => {
                    let reference_points = [origin];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
            }
        }

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Polygon;
    use crate::{DataType, Layer};

    mod instance {
        use super::*;
        use crate::{Path, Point, Polygon, Text};

        #[test]
        fn test_instance_cell() {
            let instance = Instance::from("test_cell");
            assert_eq!(instance, Instance::Cell("test_cell".to_string()));
        }

        #[test]
        fn test_instance_from_string() {
            let instance = Instance::from(String::from("my_cell"));
            assert_eq!(instance, Instance::Cell("my_cell".to_string()));
        }

        #[test]
        fn test_instance_from_polygon() {
            let polygon = Polygon::new(
                [
                    Point::integer(0, 0, 1e-9),
                    Point::integer(10, 0, 1e-9),
                    Point::integer(10, 10, 1e-9),
                ],
                Layer::new(1),
                DataType::new(0),
            );
            assert!(Instance::from(polygon).as_element().is_some());
        }

        #[test]
        fn test_instance_default() {
            let instance = Instance::default();
            assert_eq!(instance, Instance::Cell(String::new()));
        }

        #[test]
        fn test_instance_display() {
            let instance = Instance::from("test_cell");
            insta::assert_snapshot!(instance.to_string(), @"Cell instance: test_cell");
        }

        #[test]
        fn test_instance_clone() {
            let instance = Instance::from("test_cell");
            let cloned = instance.clone();
            assert_eq!(instance, cloned);
        }

        #[test]
        fn test_instance_from_path() {
            let path = Path::new(
                vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
                Layer::new(1),
                DataType::new(0),
                None,
                None,
            );

            let instance = Instance::from(path);

            assert!(instance.as_element().unwrap().as_path().is_some());
        }

        #[test]
        fn test_instance_from_text() {
            use crate::{HorizontalPresentation, VerticalPresentation};

            let text = Text::new(
                "test",
                Point::integer(0, 0, 1e-9),
                Layer::new(1),
                DataType::new(0),
                1.0,
                0.0,
                false,
                VerticalPresentation::Middle,
                HorizontalPresentation::Centre,
            );

            let instance = Instance::from(text);

            assert!(instance.as_element().unwrap().as_text().is_some());
        }

        #[test]
        fn test_instance_from_cell() {
            let cell = Cell::new("my_cell");
            let instance = Instance::from(&cell);

            assert_eq!(instance, Instance::Cell("my_cell".to_string()));
        }

        #[test]
        fn test_instance_display_element() {
            let polygon = Polygon::new(
                [
                    Point::integer(0, 0, 1e-9),
                    Point::integer(10, 0, 1e-9),
                    Point::integer(10, 10, 1e-9),
                ],
                Layer::new(1),
                DataType::new(0),
            );
            let instance = Instance::from(polygon);
            insta::assert_snapshot!(instance.to_string(), @"Element instance: Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0");
        }
    }

    #[test]
    fn test_reference_new() {
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

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
        let grid = Grid::default();

        let reference = Reference::new("test_cell").with_grid(grid);

        let cell = reference.instance().as_cell().unwrap();
        assert_eq!(cell, "test_cell");

        assert!(reference.instance().as_element().is_none());
    }

    #[test]
    fn test_reference_from_polygon() {
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let reference = Reference::new(polygon.clone());

        let polygon_from_reference = reference
            .instance()
            .as_element()
            .unwrap()
            .as_polygon()
            .unwrap()
            .clone();

        assert_eq!(polygon, polygon_from_reference);

        assert!(reference.instance().as_cell().is_none());
    }

    #[test]
    fn test_reference_display() {
        let grid = Grid::default();

        let reference = Reference::new("test_cell").with_grid(grid);

        insta::assert_snapshot!(reference.to_string(), @"Reference to Cell instance: test_cell with grid Grid at Point(0 (1.000e-9), 0 (1.000e-9)) with 1 columns and 1 rows, spacing (None, None), magnification 1.0, angle 0.0, x_reflection false");
    }

    #[test]
    fn test_reference_to_integer_unit() {
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_origin(Point::float(1.5, 2.5, 1e-6))
            .with_spacing_x(Some(Point::float(10.0, 0.0, 1e-6)));

        let reference = Reference::new(polygon).with_grid(grid);
        let converted = reference.to_integer_unit();

        assert_eq!(
            converted.grid().origin(),
            Point::float(1.5, 2.5, 1e-6).to_integer_unit()
        );
        assert_eq!(
            converted.grid().spacing_x(),
            Some(Point::float(10.0, 0.0, 1e-6).to_integer_unit())
        );
    }

    #[test]
    fn test_reference_to_float_unit() {
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_origin(Point::integer(10, 20, 1e-9))
            .with_spacing_x(Some(Point::integer(5, 0, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);
        let converted = reference.to_float_unit();

        assert_eq!(
            converted.grid().origin(),
            Point::integer(10, 20, 1e-9).to_float_unit()
        );
        assert_eq!(
            converted.grid().spacing_x(),
            Some(Point::integer(5, 0, 1e-9).to_float_unit())
        );
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
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let flattened = reference.flatten(None, &library);
        assert_eq!(flattened.len(), 4);

        insta::assert_debug_snapshot!(flattened);
    }

    #[test]
    fn test_reference_flatten_with_depth() {
        let library = Library::new("main");
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let flattened = reference.flatten(Some(0), &library);
        // Depth 0 should return the reference itself
        assert_eq!(flattened.len(), 1);
    }

    #[test]
    fn test_reference_get_elements_in_grid() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(3)
            .with_rows(3)
            .with_spacing_x(Some(Point::integer(20, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 20, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 9); // 3x3 grid
    }

    #[test]
    fn test_reference_with_x_reflection() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)))
            .with_x_reflection(true);

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 4);
    }

    #[test]
    fn test_reference_with_rotation_and_magnification() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)))
            .with_magnification(2.0)
            .with_angle(std::f64::consts::PI / 2.0)
            .with_x_reflection(false);

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 4);
    }

    #[test]
    fn test_reference_transform() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let centre = Point::integer(5, 5, 1e-9);
        let transformed = reference.rotate(std::f64::consts::PI / 2.0, centre);

        assert!(transformed.grid().angle() != 0.0);
    }

    #[test]
    fn test_reference_move_to() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let target = Point::integer(20, 20, 1e-9);
        let moved = reference.move_to(target);

        assert_eq!(moved.grid().origin(), target);
    }

    #[test]
    fn test_reference_flatten_cell_reference() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let mut cell = crate::Cell::new("test_cell");
        cell.add(polygon);
        library.add_cell(cell);

        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new("test_cell").with_grid(grid);

        let flattened = reference.flatten(None, &library);
        assert_eq!(flattened.len(), 4); // 2x2 grid
    }

    #[test]
    fn test_reference_1x1_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(1)
            .with_rows(1)
            .with_spacing_x(Some(Point::integer(20, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 20, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon.clone());
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 1);
        let result = elements[0].as_polygon().unwrap();
        assert_eq!(result.points(), polygon.points());
    }

    #[test]
    fn test_reference_asymmetric_1x5_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(1)
            .with_rows(5)
            .with_spacing_x(Some(Point::integer(20, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 20, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 5);
        for (i, el) in elements.iter().enumerate() {
            let p = el.as_polygon().unwrap();
            let expected_y_offset = (i as i32) * 20;
            assert_eq!(p.points()[0], Point::integer(0, expected_y_offset, 1e-9));
        }
    }

    #[test]
    fn test_reference_asymmetric_5x1_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(5)
            .with_rows(1)
            .with_spacing_x(Some(Point::integer(20, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 20, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 5);
        for (i, el) in elements.iter().enumerate() {
            let p = el.as_polygon().unwrap();
            let expected_x_offset = (i as i32) * 20;
            assert_eq!(p.points()[0], Point::integer(expected_x_offset, 0, 1e-9));
        }
    }

    #[test]
    fn test_reference_none_spacing_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default().with_columns(3).with_rows(3);

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon.clone());
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 9);
        for el in &elements {
            let p = el.as_polygon().unwrap();
            assert_eq!(p.points(), polygon.points());
        }
    }

    #[test]
    fn test_reference_zero_spacing_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(3)
            .with_rows(3)
            .with_spacing_x(Some(Point::integer(0, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 0, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon.clone());
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 9);
        for el in &elements {
            let p = el.as_polygon().unwrap();
            assert_eq!(p.points(), polygon.points());
        }
    }

    #[test]
    fn test_reference_negative_spacing_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(3)
            .with_rows(1)
            .with_spacing_x(Some(Point::integer(-20, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, -20, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 3);
        for (i, el) in elements.iter().enumerate() {
            let p = el.as_polygon().unwrap();
            let expected_x_offset = (i as i32) * -20;
            assert_eq!(p.points()[0], Point::integer(expected_x_offset, 0, 1e-9));
        }
    }

    #[test]
    fn test_reference_large_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(1, 0, 1e-9),
                Point::integer(1, 1, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(10)
            .with_rows(10)
            .with_spacing_x(Some(Point::integer(5, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 5, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert_eq!(elements.len(), 100);
    }

    #[test]
    fn test_reference_flatten_nonexistent_cell() {
        let library = Library::new("main");
        let reference = Reference::new("nonexistent_cell");

        let flattened = reference.flatten(None, &library);
        assert!(flattened.is_empty());
    }

    #[test]
    fn test_reference_flatten_nonexistent_cell_with_grid() {
        let library = Library::new("main");
        let grid = Grid::default()
            .with_columns(3)
            .with_rows(3)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new("missing_cell").with_grid(grid);

        let flattened = reference.flatten(None, &library);
        assert!(flattened.is_empty());
    }

    #[test]
    fn test_reference_flatten_empty_cell() {
        let mut library = Library::new("main");
        let cell = crate::Cell::new("empty_cell");
        library.add_cell(cell);

        let reference = Reference::new("empty_cell");

        let flattened = reference.flatten(None, &library);
        assert!(flattened.is_empty());
    }

    #[test]
    fn test_reference_flatten_empty_cell_with_grid() {
        let mut library = Library::new("main");
        let cell = crate::Cell::new("empty_cell");
        library.add_cell(cell);

        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new("empty_cell").with_grid(grid);

        let flattened = reference.flatten(None, &library);
        assert!(flattened.is_empty());
    }

    #[test]
    fn test_reference_depth_limited_flatten_nested() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let mut inner_cell = crate::Cell::new("inner");
        inner_cell.add(polygon);
        library.add_cell(inner_cell);

        let mut outer_cell = crate::Cell::new("outer");
        outer_cell.add(Reference::new("inner"));
        library.add_cell(outer_cell);

        // depth=1 should resolve outer_cell but leave inner references unresolved
        let reference = Reference::new("outer");
        let flattened_depth_1 = reference.clone().flatten(Some(1), &library);
        assert_eq!(flattened_depth_1.len(), 1);
        assert!(flattened_depth_1[0].as_reference().is_some());

        // depth=2 should fully resolve to the polygon
        let flattened_depth_2 = reference.flatten(Some(2), &library);
        assert_eq!(flattened_depth_2.len(), 1);
        assert!(flattened_depth_2[0].as_polygon().is_some());
    }

    #[test]
    fn test_reference_depth_limited_flatten_triple_nested() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(5, 0, 1e-9),
                Point::integer(5, 5, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let mut cell_a = crate::Cell::new("a");
        cell_a.add(polygon);
        library.add_cell(cell_a);

        let mut cell_b = crate::Cell::new("b");
        cell_b.add(Reference::new("a"));
        library.add_cell(cell_b);

        let mut cell_c = crate::Cell::new("c");
        cell_c.add(Reference::new("b"));
        library.add_cell(cell_c);

        let reference = Reference::new("c");

        // depth=0 returns the reference itself
        let d0 = reference.clone().flatten(Some(0), &library);
        assert_eq!(d0.len(), 1);
        assert!(d0[0].as_reference().is_some());

        // depth=1 resolves c -> ref(b), still a reference
        let d1 = reference.clone().flatten(Some(1), &library);
        assert_eq!(d1.len(), 1);
        assert!(d1[0].as_reference().is_some());

        // depth=2 resolves c -> b -> ref(a), still a reference
        let d2 = reference.clone().flatten(Some(2), &library);
        assert_eq!(d2.len(), 1);
        assert!(d2[0].as_reference().is_some());

        // depth=3 fully resolves to the polygon
        let d3 = reference.flatten(Some(3), &library);
        assert_eq!(d3.len(), 1);
        assert!(d3[0].as_polygon().is_some());
    }

    #[test]
    fn test_reference_zero_column_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(0)
            .with_rows(3)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert!(elements.is_empty());
    }

    #[test]
    fn test_reference_zero_row_grid_expansion() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(3)
            .with_rows(0)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element = Element::Polygon(polygon);
        let elements = reference.get_elements_in_grid(&element);

        assert!(elements.is_empty());
    }

    #[test]
    fn test_reference_flatten_with_zero_column_grid() {
        let library = Library::new("main");
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(0)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let flattened = reference.flatten(None, &library);
        assert!(flattened.is_empty());
    }

    #[test]
    fn stream_flatten_matches_flatten() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let mut cell = crate::Cell::new("test_cell");
        cell.add(polygon.clone());
        library.add_cell(cell);

        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new("test_cell").with_grid(grid);

        let flattened = reference.clone().flatten(None, &library);

        let (tx, rx) = mpsc::channel();
        reference
            .stream_flatten(None, &library, &tx)
            .expect("stream should succeed");
        drop(tx);
        let streamed: Vec<Element> = rx.iter().collect();

        assert_eq!(flattened.len(), streamed.len());
        assert_eq!(flattened, streamed);
    }

    #[test]
    fn stream_flatten_element_reference_matches_flatten() {
        let library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);

        let flattened = reference.clone().flatten(None, &library);

        let (tx, rx) = mpsc::channel();
        reference
            .stream_flatten(None, &library, &tx)
            .expect("stream should succeed");
        drop(tx);
        let streamed: Vec<Element> = rx.iter().collect();

        assert_eq!(flattened, streamed);
    }

    #[test]
    fn stream_flatten_cancellation_returns_err() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );

        let mut cell = crate::Cell::new("test_cell");
        cell.add(polygon);
        library.add_cell(cell);

        let reference = Reference::new("test_cell");

        let (tx, rx) = mpsc::channel();
        drop(rx);
        let result = reference.stream_flatten(None, &library, &tx);
        assert!(result.is_err());
    }
}
