use pyo3::prelude::*;

use crate::path::Path;
use crate::point::Point;
use crate::polygon::Polygon;
use crate::reference::Reference;
use crate::text::Text;
use crate::traits::{Dimensions, Movable, Rotatable, Scalable};

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, PartialEq, Default)]
pub struct Cell {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub polygons: Vec<Polygon>,
    #[pyo3(get)]
    pub paths: Vec<Path>,
    #[pyo3(get)]
    pub references: Vec<Reference>,
    #[pyo3(get)]
    pub texts: Vec<Text>,
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

impl Movable for Cell {
    fn move_to(&mut self, point: Point) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.move_to(point);
        }

        for path in &mut self.paths {
            path.move_to(point);
        }

        for reference in &mut self.references {
            reference.move_to(point);
        }

        for text in &mut self.texts {
            text.move_to(point);
        }

        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.move_by(vector);
        }

        for path in &mut self.paths {
            path.move_by(vector);
        }

        for reference in &mut self.references {
            reference.move_by(vector);
        }

        for text in &mut self.texts {
            text.move_by(vector);
        }

        self
    }
}

impl Rotatable for Cell {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.rotate(angle, centre);
        }

        for path in &mut self.paths {
            path.rotate(angle, centre);
        }

        for reference in &mut self.references {
            reference.rotate(angle, centre);
        }

        for text in &mut self.texts {
            text.rotate(angle, centre);
        }

        self
    }
}

// Implementing the Scalable trait for Cell
impl Scalable for Cell {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.scale(factor, centre);
        }

        for path in &mut self.paths {
            path.scale(factor, centre);
        }

        for reference in &mut self.references {
            reference.scale(factor, centre);
        }

        for text in &mut self.texts {
            text.scale(factor, centre);
        }

        self
    }
}

impl Dimensions for Cell {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min = Point::new(f64::INFINITY, f64::INFINITY);
        let mut max = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);

        for polygon in &self.polygons {
            let (polygon_min, polygon_max) = polygon.bounding_box();
            min.x = min.x.min(polygon_min.x);
            min.y = min.y.min(polygon_min.y);
            max.x = max.x.max(polygon_max.x);
            max.y = max.y.max(polygon_max.y);
        }

        for path in &self.paths {
            let (path_min, path_max) = path.bounding_box();
            min.x = min.x.min(path_min.x);
            min.y = min.y.min(path_min.y);
            max.x = max.x.max(path_max.x);
            max.y = max.y.max(path_max.y);
        }

        for reference in &self.references {
            let (reference_min, reference_max) = reference.bounding_box();
            min.x = min.x.min(reference_min.x);
            min.y = min.y.min(reference_min.y);
            max.x = max.x.max(reference_max.x);
            max.y = max.y.max(reference_max.y);
        }

        for text in &self.texts {
            let (text_min, text_max) = text.bounding_box();
            min.x = min.x.min(text_min.x);
            min.y = min.y.min(text_min.y);
            max.x = max.x.max(text_max.x);
            max.y = max.y.max(text_max.y);
        }

        (min, max)
    }
}
