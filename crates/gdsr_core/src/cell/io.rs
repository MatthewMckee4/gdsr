use std::fs::File;
use std::{collections::HashSet, io};

use chrono::{Datelike, Local, Timelike};

use crate::{
    Cell, CoordNum,
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    elements::{Element, Reference, reference::Instance},
    traits::ToGds,
    utils::io::{write_string_with_record_to_file, write_u16_array_to_file},
};

impl<DatabaseUnitT: CoordNum> Cell<DatabaseUnitT> {
    pub fn to_gds_impl(
        &self,
        file: &mut File,
        units: f64,
        precision: f64,
        written_cell_names: &mut HashSet<String>,
    ) -> io::Result<()> {
        let now = Local::now();
        let timestamp = now.naive_utc();

        let mut cells_to_write: Vec<Self> = Vec::new();

        let cell_head = [
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

        write_u16_array_to_file(file, &cell_head)?;

        write_string_with_record_to_file(file, GDSRecord::StrName, &self.name)?;

        for path in &self.paths {
            path.to_gds_impl(file, units / precision)?;
        }

        for polygon in &self.polygons {
            polygon.to_gds_impl(file, units / precision)?;
        }

        for text in &self.texts {
            text.to_gds_impl(file, units / precision)?;
        }

        for reference in &self.references {
            get_child_cells(reference, &mut cells_to_write, written_cell_names);
            reference.to_gds_impl(file, units / precision)?;
        }

        let cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(file, &cell_tail)?;

        for cell in cells_to_write {
            cell.to_gds_impl(file, units, precision, written_cell_names)?;
        }

        Ok(())
    }
}

fn get_child_cells<DatabaseUnitT: CoordNum>(
    reference: &Reference<DatabaseUnitT>,
    child_cells: &mut Vec<Cell<DatabaseUnitT>>,
    written_cell_names: &mut HashSet<String>,
) {
    match &reference.instance() {
        Instance::Cell(child_cell) => {
            if !written_cell_names.contains(&child_cell.name) {
                written_cell_names.insert(child_cell.name.clone());
                child_cells.push(child_cell.clone());
            }
        }
        Instance::Element(element) => match element.as_ref().as_ref() {
            Element::Path(_) | Element::Polygon(_) | Element::Text(_) => {}
            Element::Reference(reference) => {
                get_child_cells(reference, child_cells, written_cell_names);
            }
        },
    }
}
