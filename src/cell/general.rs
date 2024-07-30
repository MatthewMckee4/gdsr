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
            references: Vec::new(),
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
                Element::Reference(reference) => {
                    self.references.push(*reference);
                }
                Element::Text(text) => {
                    self.texts.push(text);
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
                Element::Reference(reference) => {
                    self.references.retain(|x| x != &*reference);
                }
                Element::Text(text) => {
                    self.texts.retain(|x| x != &text);
                }
            }
        }
        Ok(())
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
