use crate::{
    Point,
    path::Path,
    polygon::Polygon,
    text::Text,
    traits::{Dimensions, Transformable},
    transformation::Transformation,
};

#[derive(Clone, Default)]
pub struct Cell {
    pub name: String,
    pub polygons: Vec<Polygon>,
    pub paths: Vec<Path>,
    pub texts: Vec<Text>,
}

impl Cell {
    pub fn new(name: String) -> Self {
        Self {
            name,
            polygons: Vec::new(),
            paths: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        self.polygons.push(polygon);
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn add_text(&mut self, text: Text) {
        self.texts.push(text);
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell: {} with {} polygons, {} paths, and {} texts",
            self.name,
            self.polygons.len(),
            self.paths.len(),
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
        self.name == other.name
            && self.polygons == other.polygons
            && self.paths == other.paths
            && self.texts == other.texts
    }
}

impl Transformable for Cell {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.transform(transformation);
        }

        for path in &mut self.paths {
            path.transform(transformation);
        }

        for text in &mut self.texts {
            text.transform(transformation);
        }

        self
    }
}

impl Dimensions for Cell {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for polygon in &self.polygons {
            let (polygon_min, polygon_max) = polygon.bounding_box();
            min_x = min_x.min(polygon_min.x());
            min_y = min_y.min(polygon_min.y());
            max_x = max_x.max(polygon_max.x());
            max_y = max_y.max(polygon_max.y());
        }

        for path in &self.paths {
            let (path_min, path_max) = path.bounding_box();
            min_x = min_x.min(path_min.x());
            min_y = min_y.min(path_min.y());
            max_x = max_x.max(path_max.x());
            max_y = max_y.max(path_max.y());
        }

        for text in &self.texts {
            let (text_min, text_max) = text.bounding_box();
            min_x = min_x.min(text_min.x());
            min_y = min_y.min(text_min.y());
            max_x = max_x.max(text_max.x());
            max_y = max_y.max(text_max.y());
        }

        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }
}
