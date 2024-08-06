use std::collections::HashSet;
use std::fs::File;

use chrono::{Datelike, Local, Timelike};

use pyo3::prelude::*;

use crate::element::Element;
use crate::utils::io::create_temp_file;
use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    reference::ReferenceInstance,
    traits::ToGds,
    utils::{
        io::{write_gds, write_string_with_record_to_file, write_u16_array_to_file},
        transformations::py_any_path_to_string_or_temp_name,
    },
};

use super::*;

impl Cell {
    pub fn _to_gds(
        &self,
        mut file: File,
        units: f64,
        precision: f64,
        written_cell_names: &mut HashSet<String>,
    ) -> PyResult<File> {
        let now = Local::now();
        let timestamp = now.naive_utc();

        let mut cells_to_write: Vec<Cell> = Vec::new();

        let mut cell_head = [
            28,
            combine_record_and_data_type(GDSRecord::BgnStr, GDSDataType::TwoByteSignedInteger),
            timestamp.year() as u16,
            timestamp.month() as u16,
            timestamp.day() as u16,
            timestamp.hour() as u16,
            timestamp.minute() as u16,
            timestamp.second() as u16,
            timestamp.year() as u16,
            timestamp.month() as u16,
            timestamp.day() as u16,
            timestamp.hour() as u16,
            timestamp.minute() as u16,
            timestamp.second() as u16,
        ];

        file = write_u16_array_to_file(file, &mut cell_head)?;

        file = write_string_with_record_to_file(file, GDSRecord::StrName, &self.name)?;

        for path in &self.paths {
            file = path._to_gds(file, units / precision)?;
        }

        for polygon in &self.polygons {
            file = polygon._to_gds(file, units / precision)?;
        }

        for text in &self.texts {
            file = text._to_gds(file, units / precision)?;
        }

        for reference in &self.references {
            get_child_cells(reference, &mut cells_to_write, written_cell_names);
            file = reference._to_gds(file, units / precision)?;
        }

        let mut cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        file = write_u16_array_to_file(file, &mut cell_tail)?;

        for cell in cells_to_write {
            file = cell._to_gds(file, units, precision, written_cell_names)?;
        }

        Ok(file)
    }
}

fn get_child_cells(
    reference: &Reference,
    child_cells: &mut Vec<Cell>,
    written_cell_names: &mut HashSet<String>,
) {
    match &reference.instance {
        ReferenceInstance::Cell(child_cell) => {
            if !written_cell_names.contains(&child_cell.name) {
                written_cell_names.insert(child_cell.name.clone());
                child_cells.push(child_cell.clone());
            }
        }
        ReferenceInstance::Element(element) => match element {
            Element::Path(_) | Element::Polygon(_) | Element::Text(_) => {}
            Element::Reference(reference) => {
                get_child_cells(reference, child_cells, written_cell_names)
            }
        },
    }
}

#[pymethods]
impl Cell {
    #[pyo3(signature=(file_name=None, units=1e-6, precision=1e-10))]
    pub fn to_gds(
        &self,
        #[pyo3(from_py_with = "py_any_path_to_string_or_temp_name")] file_name: Option<String>,
        units: f64,
        precision: f64,
    ) -> PyResult<String> {
        write_gds(
            file_name.unwrap_or(create_temp_file()?),
            "library",
            units,
            precision,
            [self.clone()].to_vec(),
        )
    }
}
