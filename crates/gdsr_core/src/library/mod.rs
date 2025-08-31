use std::collections::HashMap;

use crate::cell::Cell;

#[derive(Default)]
pub struct Library {
    pub name: String,
    pub cells: HashMap<String, Cell>,
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library({})", self.name)
    }
}
