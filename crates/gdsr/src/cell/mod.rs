use crate::{
    Element, Library, Movable, Path, Polygon, Reference, Text, Transformable, Transformation,
};

mod io;

/// A named cell containing polygons, paths, texts, and references to other cells or elements.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    name: String,
    polygons: Vec<Polygon>,
    paths: Vec<Path>,
    texts: Vec<Text>,
    references: Vec<Reference>,
}

impl Cell {
    /// Creates a new empty cell with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            polygons: Vec::new(),
            paths: Vec::new(),
            texts: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Returns the cell name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Returns the polygons in this cell.
    pub const fn polygons(&self) -> &Vec<Polygon> {
        &self.polygons
    }

    /// Returns the paths in this cell.
    pub const fn paths(&self) -> &Vec<Path> {
        &self.paths
    }

    /// Returns the texts in this cell.
    pub const fn texts(&self) -> &Vec<Text> {
        &self.texts
    }

    /// Returns the references in this cell.
    pub const fn references(&self) -> &Vec<Reference> {
        &self.references
    }

    /// Adds an element (polygon, path, text, or reference) to the cell.
    pub fn add(&mut self, element: impl Into<Element>) {
        match element.into() {
            Element::Path(path) => self.paths.push(path),
            Element::Polygon(polygon) => self.polygons.push(polygon),
            Element::Reference(reference) => self.references.push(reference),
            Element::Text(text) => self.texts.push(text),
        }
    }

    /// Returns all elements in this cell, recursively flattening references up to the given depth.
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
    fn transform_impl(mut self, transformation: &Transformation) -> Self {
        self.polygons = self
            .polygons
            .into_iter()
            .map(|polygon| polygon.transform_impl(transformation))
            .collect();

        self.paths = self
            .paths
            .into_iter()
            .map(|path| path.transform_impl(transformation))
            .collect();

        self.texts = self
            .texts
            .into_iter()
            .map(|text| text.transform_impl(transformation))
            .collect();

        self.references = self
            .references
            .into_iter()
            .map(|reference| reference.transform_impl(transformation))
            .collect();

        self
    }
}

impl Movable for Cell {
    fn move_to(mut self, target: crate::Point) -> Self {
        self.polygons = self
            .polygons
            .into_iter()
            .map(|polygon| polygon.move_to(target))
            .collect();

        self.paths = self
            .paths
            .into_iter()
            .map(|path| path.move_to(target))
            .collect();

        self.texts = self
            .texts
            .into_iter()
            .map(|text| text.move_to(target))
            .collect();

        self.references = self
            .references
            .into_iter()
            .map(|reference| reference.move_to(target))
            .collect();

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;

    #[test]
    fn test_cell_new() {
        let cell: Cell = Cell::new("test_cell");
        assert_eq!(cell.name, "test_cell");
        assert!(cell.polygons().is_empty());
        assert!(cell.paths().is_empty());
        assert!(cell.texts().is_empty());
        assert!(cell.references().is_empty());
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
        let polygon = Polygon::default();

        cell.add(polygon.clone());
        assert_eq!(cell.polygons.len(), 1);
        assert_eq!(cell.polygons[0], polygon);
    }

    #[test]
    fn test_add_path() {
        let mut cell = Cell::new("test_cell");
        let path = Path::default();

        cell.add(path.clone());
        assert_eq!(cell.paths.len(), 1);
        assert_eq!(cell.paths[0], path);
    }

    #[test]
    fn test_add_text() {
        let mut cell = Cell::new("test_cell");
        let text = Text::default();

        cell.add(text.clone());
        assert_eq!(cell.texts.len(), 1);
        assert_eq!(cell.texts[0], text);
    }

    #[test]
    fn test_cell_display() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::default();
        cell.add(polygon);

        insta::assert_snapshot!(cell.to_string(), @"Cell 'test_cell' with 1 polygon(s), 0 path(s), 0 text(s)");
    }

    #[test]
    fn test_cell_get_elements() {
        let library = Library::new("main");

        let polygon = Polygon::default();

        let reference = Reference::new(polygon.clone());

        let mut cell = Cell::new("test_cell");

        let path = Path::default();

        let text = Text::default();

        cell.add(reference);
        cell.add(polygon);
        cell.add(path);
        cell.add(text);

        let elements = cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(elements);

        let moved_cell = cell.move_to(Point::integer(5, 5, 1e-9));

        let moved_elements = moved_cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(moved_elements);

        let rotated_cell = moved_cell.rotate(90.0, Point::integer(5, 5, 1e-9));

        let rotated_elements = rotated_cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(rotated_elements);
    }
}
