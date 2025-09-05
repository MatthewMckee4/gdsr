use std::{collections::HashMap, io};

use crate::{
    CoordNum, DatabaseIntegerUnit,
    cell::Cell,
    utils::io::{from_gds, write_gds},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Library<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub name: String,
    pub cells: HashMap<String, Cell<DatabaseUnitT>>,
}

impl<DatabaseUnitT: CoordNum> Library<DatabaseUnitT> {
    #[must_use]
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

    #[must_use]
    pub fn contains(&self, cell: &Cell<DatabaseUnitT>) -> bool {
        self.cells.contains_key(&cell.name)
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

    pub fn from_gds(file_name: &str) -> io::Result<Self> {
        from_gds(file_name.to_string())
    }
}

impl<T: CoordNum> std::fmt::Display for Library<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}
