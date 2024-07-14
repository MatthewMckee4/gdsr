use std::fs::File;
use std::io::Write;

use chrono::{Datelike, Local, Timelike};

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::array_reference::ArrayReference;
use crate::config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord};
use crate::node::Node;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::r#box::Box;
use crate::reference::Reference;
use crate::text::Text;
use crate::utils::gds_format::{
    write_gds_head_to_file, write_gds_tail_to_file, write_u16_array_to_file,
};

use super::Cell;

impl Cell {
    pub fn add_element(&mut self, element: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(array_reference) = element.extract::<ArrayReference>() {
            self.array_references.push(array_reference);
        } else if let Ok(polygon) = element.extract::<Polygon>() {
            self.polygons.push(polygon);
        } else if let Ok(r#box) = element.extract::<Box>() {
            self.boxes.push(r#box);
        } else if let Ok(node) = element.extract::<Node>() {
            self.nodes.push(node);
        } else if let Ok(path) = element.extract::<Path>() {
            self.paths.push(path);
        } else if let Ok(reference) = element.extract::<Reference>() {
            self.references.push(reference);
        } else if let Ok(text) = element.extract::<Text>() {
            self.texts.push(text);
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err("Invalid element"));
        }
        Ok(())
    }

    pub fn _to_gds(&self, mut file: File, precision: f64, units: f64) -> PyResult<File> {
        let now = Local::now();
        let timestamp = now.naive_utc();

        let len = self.name.len() + if self.name.len() % 2 != 0 { 1 } else { 0 };

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
            (4 + len) as u16,
            combine_record_and_data_type(GDSRecord::StrName, GDSDataType::AsciiString),
        ];

        write_u16_array_to_file(&mut cell_head, &mut file)?;
        file.write_all(self.name.as_bytes())?;

        for polygon in &self.polygons {
            file = polygon._to_gds(file, units / precision)?;
        }

        let mut cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];
        write_u16_array_to_file(&mut cell_tail, &mut file)?;

        Ok(file)
    }
}

#[pymethods]
impl Cell {
    #[new]
    pub fn new(name: String) -> Self {
        Cell {
            name,
            array_references: Vec::new(),
            polygons: Vec::new(),
            boxes: Vec::new(),
            nodes: Vec::new(),
            paths: Vec::new(),
            references: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(self.__repr__().unwrap())
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.name.to_string())
    }

    #[pyo3(signature=(*elements))]
    pub fn add(&mut self, elements: &Bound<'_, PyTuple>) -> PyResult<()> {
        for item in elements.iter() {
            self.add_element(&item)?;
        }
        Ok(())
    }

    pub fn to_gds(&self, file_name: &str, precision: f64, units: f64) -> PyResult<()> {
        let mut file = File::create(file_name)
            .map_err(|_| PyIOError::new_err("Could not open file for writing"))?;

        file = write_gds_head_to_file(&self.name, precision, units, file)?;

        file = write_gds_tail_to_file(file)?;

        file.flush()?;

        Ok(())
    }
}
