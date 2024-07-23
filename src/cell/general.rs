use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::element::Element;

use super::Cell;

#[pymethods]
impl Cell {
    #[new]
    pub fn new(name: String) -> Self {
        Cell {
            name,
            polygons: Vec::new(),
            paths: Vec::new(),
            cell_references: Vec::new(),
            element_references: Vec::new(),
            texts: Vec::new(),
        }
    }

    #[pyo3(signature=(*elements))]
    pub fn add(&mut self, elements: &Bound<'_, PyTuple>) -> PyResult<()> {
        for element in elements.iter() {
            let element: Element = element.extract()?;

            match element {
                Element::Polygon(polygon) => {
                    self.polygons.push(polygon);
                }
                Element::Path(path) => {
                    self.paths.push(path);
                }
                Element::CellReference(reference) => {
                    self.cell_references.push(reference);
                }
                Element::Text(text) => {
                    self.texts.push(text);
                }
                Element::ElementReference(element_reference) => {
                    self.element_references.push(*element_reference);
                }
            }
        }
        Ok(())
    }

    #[pyo3(signature=(*elements))]
    pub fn remove(&mut self, elements: &Bound<'_, PyTuple>) -> PyResult<()> {
        for element in elements.iter() {
            let element: Element = element.extract()?;

            match element {
                Element::Polygon(polygon) => {
                    self.polygons.retain(|x| x != &polygon);
                }
                Element::Path(path) => {
                    self.paths.retain(|x| x != &path);
                }
                Element::CellReference(reference) => {
                    self.cell_references.retain(|x| x != &reference);
                }
                Element::Text(text) => {
                    self.texts.retain(|x| x != &text);
                }
                Element::ElementReference(element_reference) => {
                    self.element_references.retain(|x| x != &*element_reference);
                }
            }
        }
        Ok(())
    }

    fn copy(&self) -> PyResult<Self> {
        Ok(self.clone())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
