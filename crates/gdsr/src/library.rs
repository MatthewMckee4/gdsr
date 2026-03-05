use std::collections::HashMap;

use crate::cell::Cell;
use crate::error::GdsError;
use crate::types::LayerMapping;
use crate::utils::io::{from_gds, write_gds};

/// A dangling reference: a cell contains a reference to a target that doesn't exist.
#[derive(Clone, Debug, PartialEq)]
pub struct DanglingCellReference {
    /// The cell containing the dangling reference.
    pub cell_name: String,
    /// The name of the missing target cell.
    pub target_name: String,
}

/// A GDSII library containing named cells. This is the top-level container for a GDSII design.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub(crate) name: String,
    pub(crate) cells: HashMap<String, Cell>,
}

impl Library {
    /// Creates a new empty library with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    /// Returns the library name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the map of cell names to cells.
    pub fn cells(&self) -> &HashMap<String, Cell> {
        &self.cells
    }

    /// Sets the library name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Adds a cell to the library. If a cell with the same name exists, it is replaced.
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.insert(cell.name().to_string(), cell);
    }

    /// Removes the given cells from the library by name.
    pub fn remove_cell(&mut self, cells: Vec<Cell>) {
        for cell in cells {
            self.cells.remove(cell.name());
        }
    }

    /// Returns a reference to the cell with the given name, if it exists.
    pub fn get_cell(&self, name: &str) -> Option<&Cell> {
        self.cells.get(name)
    }

    /// Returns `true` if the library contains a cell with the same name.
    pub fn contains_cell(&self, cell: &Cell) -> bool {
        self.cells.contains_key(cell.name())
    }

    /// Write the library to a GDS file.
    ///
    /// The given user units are only used when writing the GDSII header.
    /// So, when you open the GDSII file you see values in these units.
    ///
    /// The database units are used when scaling values before writing.
    ///
    /// If you have a unit value 10 with units 1e-9, and database units are 1e-10,
    /// then the scaled value will be 100.
    pub fn write_file<P: AsRef<std::path::Path>>(
        &self,
        file_name: P,
        user_units: f64,
        database_units: f64,
    ) -> Result<(), GdsError> {
        let cells: Vec<&Cell> = self.cells.values().collect();
        write_gds(file_name, &self.name, user_units, database_units, &cells)
    }

    /// Remaps layer/data type pairs on all elements in all cells using the given mapping.
    pub fn remap_layers(&mut self, mapping: &LayerMapping) {
        for cell in self.cells.values_mut() {
            cell.remap_layers(mapping);
        }
    }

    /// Returns all dangling cell references in the library.
    ///
    /// A dangling cell reference is a `Reference` whose resolved cell name
    /// (via [`Reference::referenced_cell_name`]) does not match any cell in the library.
    /// This recursively resolves through inline element wrappers.
    pub fn dangling_cell_references(&self) -> Vec<DanglingCellReference> {
        let mut dangling = Vec::new();
        for (cell_name, cell) in &self.cells {
            for target in cell.referenced_cell_names() {
                if !self.cells.contains_key(target) {
                    dangling.push(DanglingCellReference {
                        cell_name: cell_name.clone(),
                        target_name: target.to_string(),
                    });
                }
            }
        }
        dangling
    }

    /// Read a library from a GDS file.
    ///
    /// The given units are not required they are used to normalize the database units
    /// to some more readable values. For example a unit value of 10 with units 1e-9
    /// and database units of 1e-10 will result in a scaled value of 100.
    /// This means you can work with the values in a more human-readable format.
    pub fn read_file<P: AsRef<std::path::Path>>(
        file_name: P,
        units: Option<f64>,
    ) -> Result<Self, GdsError> {
        from_gds(file_name, units)
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::{Polygon, Reference};
    use crate::{DataType, Layer, Point};

    use super::*;

    #[test]
    fn test_library_new() {
        let mut library: Library = Library::new("test_lib");
        assert_eq!(library.name, "test_lib");
        assert!(library.cells.is_empty());

        library.set_name("new_name");
        assert_eq!(library.name, "new_name");
    }

    #[test]
    fn test_library_add_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");

        library.add_cell(cell.clone());
        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("test_cell"));
        assert_eq!(library.cells.get("test_cell"), Some(&cell));
    }

    #[test]
    fn test_library_add_multiple_cells() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.add_cell(cell2);

        assert_eq!(library.cells.len(), 2);
        assert!(library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_add_duplicate_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("test_cell");
        let cell2 = Cell::new("test_cell");

        library.add_cell(cell1);
        library.add_cell(cell2.clone());

        assert_eq!(library.cells.len(), 1);
        assert_eq!(library.cells.get("test_cell"), Some(&cell2));
    }

    #[test]
    fn test_library_remove_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1.clone());
        library.add_cell(cell2);

        library.remove_cell(vec![cell1]);

        assert_eq!(library.cells.len(), 1);
        assert!(!library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_remove_nonexistent_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.remove_cell(vec![cell2]);

        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("cell1"));
    }

    #[test]
    fn test_library_contains() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        let other_cell = Cell::new("other_cell");

        library.add_cell(cell.clone());

        assert!(library.contains_cell(&cell));
        assert!(!library.contains_cell(&other_cell));
    }

    #[test]
    fn test_library_display() {
        let mut library: Library = Library::new("my_library");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.add_cell(cell2);

        insta::assert_snapshot!(library.to_string(), @"Library 'my_library' with 2 cells");
    }

    #[test]
    fn test_library_display_empty() {
        let library: Library = Library::new("empty_lib");
        insta::assert_snapshot!(library.to_string(), @"Library 'empty_lib' with 0 cells");
    }

    #[test]
    fn test_library_clone() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        library.add_cell(cell);

        let cloned = library.clone();
        assert_eq!(library, cloned);
        assert_eq!(library.name, cloned.name);
        assert_eq!(library.cells.len(), cloned.cells.len());
    }

    #[test]
    fn test_library_debug() {
        let library: Library = Library::new("debug_lib");
        insta::assert_snapshot!(format!("{library:?}"), @r#"Library { name: "debug_lib", cells: {} }"#);
    }

    #[test]
    fn test_remap_layers_across_library() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(crate::Path::new(
            vec![Point::integer(0, 0, units), Point::integer(5, 5, units)],
            Layer::new(2),
            DataType::new(3),
            None,
            None,
            None,
            None,
        ));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = [
            (
                (Layer::new(1), DataType::new(0)),
                (Layer::new(10), DataType::new(20)),
            ),
            (
                (Layer::new(2), DataType::new(3)),
                (Layer::new(22), DataType::new(33)),
            ),
        ]
        .into_iter()
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        insta::assert_debug_snapshot!(
            (cell.polygons()[0].layer(), cell.polygons()[0].data_type()),
            @r#"
        (
            Layer(
                10,
            ),
            DataType(
                20,
            ),
        )
        "#
        );
        insta::assert_debug_snapshot!(
            (cell.paths()[0].layer(), cell.paths()[0].data_type()),
            @r#"
        (
            Layer(
                22,
            ),
            DataType(
                33,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_remap_layers_inline_element() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let mut cell = Cell::new("cell");
        cell.add(Reference::new(polygon));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = [(
            (Layer::new(1), DataType::new(0)),
            (Layer::new(50), DataType::new(60)),
        )]
        .into_iter()
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        let reference = &cell.references()[0];
        let inner = reference.instance().as_element().unwrap();
        let polygon = inner.as_polygon().unwrap();
        insta::assert_debug_snapshot!(
            (polygon.layer(), polygon.data_type()),
            @r#"
        (
            Layer(
                50,
            ),
            DataType(
                60,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_remap_layers_unmatched_unchanged() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(5),
            DataType::new(6),
        ));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = [(
            (Layer::new(99), DataType::new(99)),
            (Layer::new(1), DataType::new(1)),
        )]
        .into_iter()
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        insta::assert_debug_snapshot!(
            (cell.polygons()[0].layer(), cell.polygons()[0].data_type()),
            @r#"
        (
            Layer(
                5,
            ),
            DataType(
                6,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_dangling_cell_references_none() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut base = Cell::new("base");
        base.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(base);

        let mut top = Cell::new("top");
        top.add(Reference::new("base".to_string()));
        library.add_cell(top);

        assert!(library.dangling_cell_references().is_empty());
    }

    #[test]
    fn test_dangling_cell_references_detected() {
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        cell.add(Reference::new("missing_cell".to_string()));
        library.add_cell(cell);

        insta::assert_debug_snapshot!(library.dangling_cell_references(), @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "missing_cell",
            },
        ]
        "#);
    }

    #[test]
    fn test_dangling_cell_references_multiple() {
        let mut library = Library::new("lib");

        let mut cell_a = Cell::new("cell_a");
        cell_a.add(Reference::new("ghost1".to_string()));
        cell_a.add(Reference::new("ghost2".to_string()));
        library.add_cell(cell_a);

        let mut cell_b = Cell::new("cell_b");
        cell_b.add(Reference::new("ghost3".to_string()));
        library.add_cell(cell_b);

        let mut dangling = library.dangling_cell_references();
        dangling
            .sort_by(|a, b| (&a.cell_name, &a.target_name).cmp(&(&b.cell_name, &b.target_name)));

        insta::assert_debug_snapshot!(dangling, @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "ghost1",
            },
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "ghost2",
            },
            DanglingCellReference {
                cell_name: "cell_b",
                target_name: "ghost3",
            },
        ]
        "#);
    }

    #[test]
    fn test_dangling_cell_references_inline_element() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        cell.add(Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        )));
        library.add_cell(cell);

        assert!(library.dangling_cell_references().is_empty());
    }

    #[test]
    fn test_dangling_cell_references_nested_inline_element() {
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        let inner_ref = Reference::new("missing_cell".to_string());
        cell.add(Reference::new(inner_ref));
        library.add_cell(cell);

        insta::assert_debug_snapshot!(library.dangling_cell_references(), @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "missing_cell",
            },
        ]
        "#);
    }
}
