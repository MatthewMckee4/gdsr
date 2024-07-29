use pyo3::prelude::*;
use std::fs::File;

use crate::traits::{Movable, Rotatable, Scalable, ToGds};

use super::ElementReference;

impl ToGds for ElementReference {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        for column_index in 0..self.grid.columns {
            for row_index in 0..self.grid.rows {
                let origin = self.grid.origin
                    + self.grid.spacing_x * column_index as f64
                    + self.grid.spacing_y * row_index as f64;

                let new_element = self
                    .element
                    .copy()
                    .scale(
                        if self.grid.x_reflection { -1.0 } else { 1.0 },
                        self.grid.origin,
                    )
                    .scale(self.grid.magnification, self.grid.origin)
                    .rotate(self.grid.angle, self.grid.origin)
                    .move_by(origin.rotate(self.grid.angle, self.grid.origin).scale(
                        if self.grid.x_reflection { -1.0 } else { 1.0 },
                        self.grid.origin,
                    ))
                    .copy();

                file = new_element._to_gds(file, scale)?;
            }
        }

        Ok(file)
    }
}
