use crate::{Element, Library, Path, Polygon, Reference, Text, Transformable, Transformation};

mod io;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    name: String,
    polygons: Vec<Polygon>,
    paths: Vec<Path>,
    texts: Vec<Text>,
    references: Vec<Reference>,
}

impl Cell {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            polygons: Vec::new(),
            paths: Vec::new(),
            texts: Vec::new(),
            references: Vec::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    #[must_use]
    pub const fn polygons(&self) -> &Vec<Polygon> {
        &self.polygons
    }

    #[must_use]
    pub const fn paths(&self) -> &Vec<Path> {
        &self.paths
    }

    #[must_use]
    pub const fn texts(&self) -> &Vec<Text> {
        &self.texts
    }

    #[must_use]
    pub const fn references(&self) -> &Vec<Reference> {
        &self.references
    }

    pub fn add(&mut self, element: impl Into<Element>) {
        match element.into() {
            Element::Path(path) => self.paths.push(path),
            Element::Polygon(polygon) => self.polygons.push(polygon),
            Element::Reference(reference) => self.references.push(reference),
            Element::Text(text) => self.texts.push(text),
        }
    }

    #[must_use]
    pub fn get_elements(&self, depth: Option<usize>, library: &Library) -> Vec<Element> {
        let depth = depth.unwrap_or(usize::MAX);
        let mut elements: Vec<Element> = Vec::new();

        for polygon in &self.polygons {
            elements.push(Element::Polygon(polygon.clone()));
        }

        for path in &self.paths {
            elements.push(Element::Path(path.clone()));
        }

        for text in &self.texts {
            elements.push(Element::Text(text.clone()));
        }

        for reference in &self.references {
            let reference_elements = reference.clone().flatten(Some(depth), library);
            for referenced_element in reference_elements {
                elements.push(referenced_element);
            }
        }

        elements
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell '{}' with {} polygon(s), {} path(s), {} text(s)",
            self.name,
            self.polygons.len(),
            self.paths.len(),
            self.texts.len(),
        )
    }
}

impl Transformable for Cell {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();

        new_self.polygons = new_self
            .polygons
            .into_iter()
            .map(|polygon| polygon.transform_impl(transformation))
            .collect();

        new_self.paths = new_self
            .paths
            .into_iter()
            .map(|path| path.transform_impl(transformation))
            .collect();

        new_self.texts = new_self
            .texts
            .into_iter()
            .map(|text| text.transform_impl(transformation))
            .collect();

        new_self.references = new_self
            .references
            .into_iter()
            .map(|reference| reference.transform_impl(transformation))
            .collect();

        new_self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Point,
        elements::{
            path::PathType,
            text::presentation::{HorizontalPresentation, VerticalPresentation},
        },
    };

    #[test]
    fn test_cell_new() {
        let cell: Cell = Cell::new("test_cell");
        assert_eq!(cell.name, "test_cell");
        assert!(cell.polygons.is_empty());
        assert!(cell.paths.is_empty());
        assert!(cell.texts.is_empty());
        assert!(cell.references.is_empty());
    }

    #[test]
    fn test_cell_default() {
        let cell: Cell = Cell::default();
        assert_eq!(cell.name, "");
        assert!(cell.polygons.is_empty());
        assert!(cell.paths.is_empty());
        assert!(cell.texts.is_empty());
        assert!(cell.references.is_empty());
    }

    #[test]
    fn test_add_polygon() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
                Point::integer(0, 10, 1e-9),
            ],
            1,
            0,
        );

        cell.add(polygon.clone());
        assert_eq!(cell.polygons.len(), 1);
        assert_eq!(cell.polygons[0], polygon);
    }

    #[test]
    fn test_add_path() {
        let mut cell = Cell::new("test_cell");
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            Some(PathType::Square),
            Some(2.0),
        );

        cell.add(path.clone());
        assert_eq!(cell.paths.len(), 1);
        assert_eq!(cell.paths[0], path);
    }

    #[test]
    fn test_add_text() {
        let mut cell = Cell::new("test_cell");
        let text = Text::new(
            "Test Text".to_string(),
            Point::integer(5, 5, 1e-9),
            1,
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        );

        cell.add(text.clone());
        assert_eq!(cell.texts.len(), 1);
        assert_eq!(cell.texts[0], text);
    }

    #[test]
    fn test_cell_display() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
                Point::integer(0, 10, 1e-9),
            ],
            1,
            0,
        );
        cell.add(polygon);

        let display_str = format!("{cell}");
        assert!(display_str.contains("Cell 'test_cell'"));
        assert!(display_str.contains("1 polygon(s)"));
    }

    #[test]
    fn test_cell_clone() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
                Point::integer(0, 10, 1e-9),
            ],
            1,
            0,
        );
        cell.add(polygon);

        let cloned = cell.clone();
        assert_eq!(cell, cloned);
    }
}
