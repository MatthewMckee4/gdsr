use crate::{
    CoordNum, DatabaseIntegerUnit,
    elements::{Element, Path, Polygon, Reference, Text},
    traits::Transformable,
    transformation::Transformation,
};

mod io;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) name: String,
    pub(crate) polygons: Vec<Polygon<DatabaseUnitT>>,
    pub(crate) paths: Vec<Path<DatabaseUnitT>>,
    pub(crate) texts: Vec<Text<DatabaseUnitT>>,
    pub(crate) references: Vec<Reference<DatabaseUnitT>>,
}

impl<DatabaseUnitT: CoordNum> Default for Cell<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            name: String::default(),
            polygons: Vec::default(),
            paths: Vec::default(),
            texts: Vec::default(),
            references: Vec::default(),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Cell<DatabaseUnitT> {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
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

    pub fn add(&mut self, element: impl Into<Element<DatabaseUnitT>>) {
        match element.into() {
            Element::Path(path) => self.add_path(path),
            Element::Polygon(polygon) => self.add_polygon(polygon),
            Element::Reference(reference) => self.add_reference(reference),
            Element::Text(text) => self.add_text(text),
        }
    }

    pub(crate) fn get_elements(&self, _depth: Option<usize>) -> Vec<&Element<DatabaseUnitT>> {
        todo!()
    }
}

impl<DatabaseUnitT: CoordNum> Transformable for Cell<DatabaseUnitT> {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();

        new_self.polygons = new_self
            .polygons
            .into_iter()
            .map(|polygon| polygon.transform_impl(transformation))
            .collect();

        new_self.paths = new_self
            .paths
            .into_iter()
            .map(|path| path.transform_impl(transformation))
            .collect();

        new_self.texts = new_self
            .texts
            .into_iter()
            .map(|text| text.transform_impl(transformation))
            .collect();

        new_self.references = new_self
            .references
            .into_iter()
            .map(|reference| reference.transform_impl(transformation))
            .collect();

        new_self
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
