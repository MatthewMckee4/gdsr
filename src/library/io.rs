use std::fs::File;
use std::io::Write;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

use crate::utils::gds_format::{write_gds_head_to_file, write_gds_tail_to_file};

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
        let mut file = File::create(file_name)
            .map_err(|_| PyIOError::new_err("Could not open file for writing"))?;

        file = write_gds_head_to_file(&self.name, units, precision, file)?;

        file = self._to_gds(file, units, precision)?;

        file = write_gds_tail_to_file(file)?;

        file.flush()?;

        Ok(())
    }
}
