use std::collections::HashMap;
use std::io;

use crate::cell::Cell;
use crate::utils::io::{from_gds, write_gds};

#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub(crate) name: String,
    pub(crate) cells: HashMap<String, Cell>,
}

impl Library {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn cells(&self) -> &HashMap<String, Cell> {
        &self.cells
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.insert(cell.name().to_string(), cell);
    }

    pub fn remove_cell(&mut self, cells: Vec<Cell>) {
        for cell in cells {
            self.cells.remove(cell.name());
        }
    }

    pub fn get_cell(&self, name: &str) -> Option<&Cell> {
        self.cells.get(name)
    }

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
    ) -> io::Result<()> {
        write_gds(
            file_name,
            &self.name,
            user_units,
            database_units,
            self.cells.values(),
        )
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
    ) -> io::Result<Self> {
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
}
