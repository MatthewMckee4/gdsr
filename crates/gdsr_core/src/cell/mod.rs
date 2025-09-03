use crate::{
    CoordNum, DatabaseIntegerUnit,
    elements::{Path, Polygon, Reference, Text},
    traits::Transformable,
    transformation::Transformation,
};

mod io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell<DatabaseUnitT: CoordNum> {
    pub name: String,
    pub polygons: Vec<Polygon<DatabaseUnitT>>,
    pub paths: Vec<Path<DatabaseUnitT>>,
    pub texts: Vec<Text<DatabaseUnitT>>,
    pub references: Vec<Reference<DatabaseUnitT>>,
}

impl<DatabaseUnitT: CoordNum> Cell<DatabaseUnitT> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            polygons: Vec::new(),
            paths: Vec::new(),
            texts: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn add_polygon(&mut self, polygon: Polygon<DatabaseUnitT>) {
        self.polygons.push(polygon);
    }

    pub fn add_path(&mut self, path: Path<DatabaseUnitT>) {
        self.paths.push(path);
    }

    pub fn add_text(&mut self, text: Text<DatabaseUnitT>) {
        self.texts.push(text);
    }

    pub fn add_reference(&mut self, reference: Reference<DatabaseUnitT>) {
        self.references.push(reference);
    }
}

impl Transformable for Cell<DatabaseIntegerUnit> {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        for polygon in &mut self.polygons {
            polygon.transform(transformation);
        }

        for path in &mut self.paths {
            path.transform(transformation);
        }

        // for text in &mut self.texts {
        //     text.transform(transformation);
        // }

        for reference in &mut self.references {
            reference.transform(transformation);
        }

        self
    }
}

// impl<T: CoordNum> Dimensions<T> for Cell<T> {
//     fn bounding_box(&self) -> (Point<T>, Point<T>) {
//         let mut min_x = f64::INFINITY;
//         let mut min_y = f64::INFINITY;
//         let mut max_x = f64::NEG_INFINITY;
//         let mut max_y = f64::NEG_INFINITY;

//         for polygon in &self.polygons {
//             let (polygon_min, polygon_max) = polygon.bounding_box();
//             min_x = min_x.min(polygon_min.x().into());
//             min_y = min_y.min(polygon_min.y().into());
//             max_x = max_x.max(polygon_max.x().into());
//             max_y = max_y.max(polygon_max.y().into());
//         }

//         for path in &self.paths {
//             let (path_min, path_max) = path.bounding_box();
//             min_x = min_x.min(path_min.x().into());
//             min_y = min_y.min(path_min.y().into());
//             max_x = max_x.max(path_max.x().into());
//             max_y = max_y.max(path_max.y().into());
//         }

//         for text in &self.texts {
//             let (text_min, text_max) = text.bounding_box();
//             min_x = min_x.min(text_min.x().into());
//             min_y = min_y.min(text_min.y().into());
//             max_x = max_x.max(text_max.x().into());
//             max_y = max_y.max(text_max.y().into());
//         }

//         for reference in &self.references {
//             let (reference_min, reference_max) = reference.bounding_box();
//             min_x = min_x.min(reference_min.x().into());
//             min_y = min_y.min(reference_min.y().into());
//             max_x = max_x.max(reference_max.x().into());
//             max_y = max_y.max(reference_max.y().into());
//         }

//         (
//             Point::new(min_x.into(), min_y.into()),
//             Point::new(max_x.into(), max_y.into()),
//         )
//     }
// }
