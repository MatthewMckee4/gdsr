use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    element::Element,
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
    utils::transformations::py_any_to_point,
};

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

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
    }

    #[pyo3(signature=(*elements))]
    pub fn add(&mut self, elements: Vec<Element>) -> PyResult<()> {
        for element in elements {
            match element {
                Element::Polygon(polygon) => {
                    self.polygons.push(polygon);
                }
                Element::Path(path) => {
                    self.paths.push(path);
                }
                Element::Reference(reference) => {
                    self.references.push(reference);
                }
                Element::Text(text) => {
                    self.texts.push(text);
                }
            }
        }
        Ok(())
    }

    #[pyo3(signature=(*elements))]
    pub fn remove(&mut self, elements: Vec<Element>) -> PyResult<()> {
        Python::with_gil(|py| {
            for element in elements {
                match element {
                    Element::Polygon(polygon) => {
                        self.polygons
                            .retain(|x| !x.borrow(py).eq(&polygon.borrow(py)));
                    }
                    Element::Path(path) => {
                        self.paths.retain(|x| !x.borrow(py).eq(&path.borrow(py)));
                    }
                    Element::Reference(reference) => {
                        self.references
                            .retain(|x| !x.borrow(py).eq(&reference.borrow(py)));
                    }
                    Element::Text(text) => {
                        self.texts.retain(|x| !x.borrow(py).eq(&text.borrow(py)));
                    }
                }
            }
        });
        Ok(())
    }

    pub fn contains(&self, element: Element) -> bool {
        Python::with_gil(|py| match element {
            Element::Polygon(polygon) => {
                for p in &self.polygons {
                    if p.borrow(py).eq(&polygon.borrow(py)) {
                        return true;
                    }
                }
                false
            }
            Element::Path(path) => {
                for p in &self.paths {
                    if p.borrow(py).eq(&path.borrow(py)) {
                        return true;
                    }
                }
                false
            }
            Element::Reference(reference) => {
                for r in &self.references {
                    if r.borrow(py).eq(&reference.borrow(py)) {
                        return true;
                    }
                }
                false
            }
            Element::Text(text) => {
                for t in &self.texts {
                    if t.borrow(py).eq(&text.borrow(py)) {
                        return true;
                    }
                }
                false
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.polygons.is_empty()
            && self.paths.is_empty()
            && self.references.is_empty()
            && self.texts.is_empty()
    }

    fn move_to(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_to(slf.deref_mut(), point);
        slf
    }

    fn move_by(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_by(slf.deref_mut(), vector);
        slf
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    fn rotate(
        mut slf: PyRefMut<'_, Self>,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Rotatable::rotate(slf.deref_mut(), angle, centre);
        slf
    }

    #[pyo3(signature = (factor, centre=Point::default()))]
    fn scale(
        mut slf: PyRefMut<'_, Self>,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Scalable::scale(slf.deref_mut(), factor, centre);
        slf
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            if self.name != other.name {
                return false;
            }

            if self.polygons.len() != other.polygons.len() {
                return false;
            }

            for (self_polygon, other_polygon) in self.polygons.iter().zip(other.polygons.iter()) {
                if !self_polygon.borrow(py).eq(&other_polygon.borrow(py)) {
                    return false;
                }
            }

            if self.paths.len() != other.paths.len() {
                return false;
            }

            for (self_path, other_path) in self.paths.iter().zip(other.paths.iter()) {
                if !self_path.borrow(py).eq(&other_path.borrow(py)) {
                    return false;
                }
            }

            if self.references.len() != other.references.len() {
                return false;
            }

            for (self_reference, other_reference) in
                self.references.iter().zip(other.references.iter())
            {
                if !self_reference.borrow(py).eq(&other_reference.borrow(py)) {
                    return false;
                }
            }

            if self.texts.len() != other.texts.len() {
                return false;
            }

            for (self_text, other_text) in self.texts.iter().zip(other.texts.iter()) {
                if !self_text.borrow(py).eq(&other_text.borrow(py)) {
                    return false;
                }
            }
            true
        })
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
