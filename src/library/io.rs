use std::fs::File;

use pyo3::prelude::*;

use crate::utils::io::write_gds;

use super::Library;

impl Library {
    pub fn _to_gds(&self, mut file: File, units: f64, precision: f64) -> PyResult<File> {
        for cell in &self.cells {
            file = cell._to_gds(file, units, precision)?;
        }

        Ok(file)
    }
}

#[pymethods]
impl Library {
    #[pyo3(signature=(file_name, units=1e-6, precision=1e-10))]
    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> PyResult<()> {
        write_gds(file_name, &self.name, units, precision, self.cells.clone())
    }
}
