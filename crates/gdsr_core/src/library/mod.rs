use std::{collections::HashMap, io};

use crate::{
    cell::Cell,
    utils::io::{from_gds, write_gds},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub name: String,
    pub cells: HashMap<String, Cell>,
}

impl Library {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.insert(cell.name().to_string(), cell);
    }

    pub fn remove(&mut self, cells: Vec<Cell>) {
        for cell in cells {
            self.cells.remove(cell.name());
        }
    }

    #[must_use]
    pub fn get_cell(&self, name: &str) -> Option<&Cell> {
        self.cells.get(name)
    }

    #[must_use]
    pub fn contains(&self, cell: &Cell) -> bool {
        self.cells.contains_key(cell.name())
    }

    pub fn to_gds(&self, file_name: &str, user_units: f64, database_units: f64) -> io::Result<()> {
        write_gds(
            file_name.to_string(),
            &self.name,
            user_units,
            database_units,
            self.cells.values(),
        )
    }

    pub fn from_gds(file_name: &str, units: Option<f64>) -> io::Result<Self> {
        from_gds(file_name.to_string(), units)
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
        let library: Library = Library::new("test_lib");
        assert_eq!(library.name, "test_lib");
        assert!(library.cells.is_empty());
    }

    #[test]
    fn test_library_add_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");

        library.add(cell.clone());
        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("test_cell"));
        assert_eq!(library.cells.get("test_cell"), Some(&cell));
    }

    #[test]
    fn test_library_add_multiple_cells() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add(cell1);
        library.add(cell2);

        assert_eq!(library.cells.len(), 2);
        assert!(library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_add_duplicate_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("test_cell");
        let cell2 = Cell::new("test_cell");

        library.add(cell1);
        library.add(cell2.clone());

        assert_eq!(library.cells.len(), 1);
        assert_eq!(library.cells.get("test_cell"), Some(&cell2));
    }

    #[test]
    fn test_library_remove_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add(cell1.clone());
        library.add(cell2);

        library.remove(vec![cell1]);

        assert_eq!(library.cells.len(), 1);
        assert!(!library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_remove_nonexistent_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add(cell1);
        library.remove(vec![cell2]);

        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("cell1"));
    }

    #[test]
    fn test_library_contains() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        let other_cell = Cell::new("other_cell");

        library.add(cell.clone());

        assert!(library.contains(&cell));
        assert!(!library.contains(&other_cell));
    }

    #[test]
    fn test_library_display() {
        let mut library: Library = Library::new("my_library");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add(cell1);
        library.add(cell2);

        let display_str = format!("{library}");
        assert_eq!(display_str, "Library 'my_library' with 2 cells");
    }

    #[test]
    fn test_library_display_empty() {
        let library: Library = Library::new("empty_lib");
        let display_str = format!("{library}");
        assert_eq!(display_str, "Library 'empty_lib' with 0 cells");
    }

    #[test]
    fn test_library_clone() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        library.add(cell);

        let cloned = library.clone();
        assert_eq!(library, cloned);
        assert_eq!(library.name, cloned.name);
        assert_eq!(library.cells.len(), cloned.cells.len());
    }

    #[test]
    fn test_library_debug() {
        let library: Library = Library::new("debug_lib");
        let debug_str = format!("{library:?}");
        assert!(debug_str.contains("debug_lib"));
        assert!(debug_str.contains("Library"));
    }
}
