use std::fs::File;
use std::io::Write;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

use crate::cell::Cell;
use crate::library::Library;
use crate::utils::gds_format::{write_gds_head_to_file, write_gds_tail_to_file};

#[pymethods]
impl Library {
    #[new]
    pub fn new(name: String) -> Self {
        Library {
            name,
            cells: Vec::new(),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Library(name={}, cells={})",
            self.name,
            self.cells.len()
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    #[pyo3(signature=(*cells))]
    pub fn add(
        &mut self,
        #[pyo3(from_py_with = "input_cells_to_correct_format")] cells: Vec<Cell>,
    ) -> PyResult<()> {
        for cell in cells {
            self.cells.push(cell);
        }
        Ok(())
    }

    #[pyo3(signature=(file_name, units=1e-6, precision=1e-10))]
    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> PyResult<()> {
        let mut file = File::create(file_name)
            .map_err(|_| PyIOError::new_err("Could not open file for writing"))?;

        file = write_gds_head_to_file(&self.name, units, precision, file)?;

        for cell in &self.cells {
            file = cell._to_gds(file, precision, units)?;
        }

        file = write_gds_tail_to_file(file)?;

        file.flush()?;

        Ok(())
    }
}
