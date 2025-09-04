use crate::{
    CoordNum,
    cell::Cell,
    utils::io::{from_gds, write_gds},
};

use std::{collections::HashMap, io};

#[derive(Default)]
pub struct Library<DatabaseUnitT: CoordNum> {
    pub name: String,
    pub cells: HashMap<String, Cell<DatabaseUnitT>>,
}

impl<DatabaseUnitT: CoordNum> Library<DatabaseUnitT> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    pub fn add(&mut self, cell: Cell<DatabaseUnitT>) {
        self.cells.insert(cell.name.clone(), cell);
    }

    pub fn remove(&mut self, cells: Vec<Cell<DatabaseUnitT>>) {
        for cell in cells {
            self.cells.remove(&cell.name);
        }
    }

    pub fn contains(&self, cell: Cell<DatabaseUnitT>) -> bool {
        self.cells.contains_key(&cell.name)
    }

    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> io::Result<()> {
        write_gds(
            file_name.to_string(),
            &self.name,
            units,
            precision,
            self.cells.values().map(|cell| cell.clone()).collect(),
        )
    }

    pub fn from_gds(file_name: String) -> io::Result<Self> {
        from_gds(file_name)
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
