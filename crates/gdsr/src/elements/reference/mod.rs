use crate::{Element, Grid, Library, Movable, Point, Transformable, Transformation};

pub mod instance;
mod io;

pub use instance::Instance;

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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
            1,
            0,
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
}
