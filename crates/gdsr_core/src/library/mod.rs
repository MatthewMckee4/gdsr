use crate::{CoordNum, cell::Cell};

use std::collections::HashMap;

#[derive(Default)]
pub struct Library<T: CoordNum> {
    pub name: String,
    pub cells: HashMap<String, Cell<T>>,
}

impl<T: CoordNum> Library<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            cells: HashMap::new(),
        }
    }

    pub fn add(&mut self, cells: Vec<Cell<T>>, replace_pre_existing: bool) -> Result<(), String> {
        for cell in cells {
            if !replace_pre_existing && self.cells.contains_key(&cell.name) {
                return Err(format!(
                    "Cell with name {} already exists in library",
                    cell.name
                ));
            }
            self.cells.insert(cell.name.clone(), cell);
        }
        Ok(())
    }

    pub fn remove(&mut self, cells: Vec<Cell<T>>) -> Result<(), String> {
        for cell in cells {
            self.cells.remove(&cell.name);
        }
        Ok(())
    }

    pub fn contains(&self, cell: Cell<T>) -> bool {
        self.cells.contains_key(&cell.name)
    }
}

impl<T: CoordNum> std::fmt::Display for Library<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

impl<T: CoordNum> std::fmt::Debug for Library<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library({})", self.name)
    }
}
