use pyo3::prelude::*;

use crate::{
    path::Path,
    point::Point,
    polygon::Polygon,
    reference::Reference,
    text::Text,
    traits::{Dimensions, Movable, Rotatable, Scalable},
};

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, Default)]
pub struct Cell {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub polygons: Vec<Py<Polygon>>,
    #[pyo3(get)]
    pub paths: Vec<Py<Path>>,
    #[pyo3(get)]
    pub references: Vec<Py<Reference>>,
    #[pyo3(get)]
    pub texts: Vec<Py<Text>>,
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell: {} with {} polygons, {} paths, {} references, and {} texts",
            self.name,
            self.polygons.len(),
            self.paths.len(),
            self.references.len(),
            self.texts.len()
        )
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cell({})", self.name)
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
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
}

impl Movable for Cell {
    fn move_to(&mut self, point: Point) -> &mut Self {
        Python::with_gil(|py| {
            for polygon in &mut self.polygons {
                polygon.borrow_mut(py).move_to(point);
            }

            for path in &mut self.paths {
                path.borrow_mut(py).move_to(point);
            }

            for reference in &mut self.references {
                reference.borrow_mut(py).move_to(point);
            }

            for text in &mut self.texts {
                text.borrow_mut(py).move_to(point);
            }

            self
        })
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        Python::with_gil(|py| {
            for polygon in &mut self.polygons {
                polygon.borrow_mut(py).move_by(vector);
            }

            for path in &mut self.paths {
                path.borrow_mut(py).move_by(vector);
            }

            for reference in &mut self.references {
                reference.borrow_mut(py).move_by(vector);
            }

            for text in &mut self.texts {
                text.borrow_mut(py).move_by(vector);
            }

            self
        })
    }
}

impl Rotatable for Cell {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            for polygon in &mut self.polygons {
                polygon.borrow_mut(py).rotate(angle, centre);
            }

            for path in &mut self.paths {
                path.borrow_mut(py).rotate(angle, centre);
            }

            for reference in &mut self.references {
                reference.borrow_mut(py).rotate(angle, centre);
            }

            for text in &mut self.texts {
                text.borrow_mut(py).rotate(angle, centre);
            }

            self
        })
    }
}

impl Scalable for Cell {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            for polygon in &mut self.polygons {
                polygon.borrow_mut(py).scale(factor, centre);
            }

            for path in &mut self.paths {
                path.borrow_mut(py).scale(factor, centre);
            }

            for reference in &mut self.references {
                reference.borrow_mut(py).scale(factor, centre);
            }

            for text in &mut self.texts {
                text.borrow_mut(py).scale(factor, centre);
            }

            self
        })
    }
}

impl Dimensions for Cell {
    fn bounding_box(&self) -> (Point, Point) {
        Python::with_gil(|py| {
            let mut min = Point::new(f64::INFINITY, f64::INFINITY);
            let mut max = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);

            for polygon in &self.polygons {
                let (polygon_min, polygon_max) = polygon.borrow_mut(py).bounding_box();
                min.x = min.x.min(polygon_min.x);
                min.y = min.y.min(polygon_min.y);
                max.x = max.x.max(polygon_max.x);
                max.y = max.y.max(polygon_max.y);
            }

            for path in &self.paths {
                let (path_min, path_max) = path.borrow_mut(py).bounding_box();
                min.x = min.x.min(path_min.x);
                min.y = min.y.min(path_min.y);
                max.x = max.x.max(path_max.x);
                max.y = max.y.max(path_max.y);
            }

            for reference in &self.references {
                let (reference_min, reference_max) = reference.borrow_mut(py).bounding_box();
                min.x = min.x.min(reference_min.x);
                min.y = min.y.min(reference_min.y);
                max.x = max.x.max(reference_max.x);
                max.y = max.y.max(reference_max.y);
            }

            for text in &self.texts {
                let (text_min, text_max) = text.borrow_mut(py).bounding_box();
                min.x = min.x.min(text_min.x);
                min.y = min.y.min(text_min.y);
                max.x = max.x.max(text_max.x);
                max.y = max.y.max(text_max.y);
            }

            (min, max)
        })
    }
}
