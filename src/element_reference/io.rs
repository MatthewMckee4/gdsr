use pyo3::prelude::*;
use std::fs::File;

use crate::traits::ToGds;

use super::ElementReference;

impl ToGds for ElementReference {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        for column_index in 0..self.grid.columns {
            for row_index in 0..self.grid.rows {
                let origin = self.grid.origin
                    + self.grid.spacing_x * column_index as f64
                    + self.grid.spacing_y * row_index as f64;
            }
        }

        Ok(file)
    }
}
