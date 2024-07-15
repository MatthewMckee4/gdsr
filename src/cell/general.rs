use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::array_reference::ArrayReference;
use crate::element::Element;
use crate::node::Node;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::r#box::Box;
use crate::reference::Reference;
use crate::text::Text;

use super::Cell;

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

    #[pyo3(signature=(*elements))]
    pub fn add(&mut self, elements: &Bound<'_, PyTuple>) -> PyResult<()> {
        for element in elements.iter() {
            let element: Element = element.extract()?;

            match element {
                Element::ArrayReference(array_reference) => {
                    self.array_references.push(array_reference.clone());
                }
                Element::Polygon(polygon) => {
                    self.polygons.push(polygon.clone());
                }
                Element::Box(r#box) => {
                    self.boxes.push(r#box.clone());
                }
                Element::Node(node) => {
                    self.nodes.push(node.clone());
                }
                Element::Path(path) => {
                    self.paths.push(path.clone());
                }
                Element::Reference(reference) => {
                    self.references.push(reference.clone());
                }
                Element::Text(text) => {
                    self.texts.push(text.clone());
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
                Element::ArrayReference(array_reference) => {
                    self.array_references.retain(|x| x != &array_reference);
                }
                Element::Polygon(polygon) => {
                    self.polygons.retain(|x| x != &polygon);
                }
                Element::Box(r#box) => {
                    self.boxes.retain(|x| x != &r#box);
                }
                Element::Node(node) => {
                    self.nodes.retain(|x| x != &node);
                }
                Element::Path(path) => {
                    self.paths.retain(|x| x != &path);
                }
                Element::Reference(reference) => {
                    self.references.retain(|x| x != &reference);
                }
                Element::Text(text) => {
                    self.texts.retain(|x| x != &text);
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
