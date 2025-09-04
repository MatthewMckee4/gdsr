use crate::{
    CoordNum, DataType, DatabaseIntegerUnit, Layer, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Polygon<DatabaseUnitT: CoordNum> {
    pub(crate) points: Vec<Point<DatabaseUnitT>>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
}

impl<DatabaseUnitT: CoordNum> Default for Polygon<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            points: Default::default(),
            layer: Default::default(),
            data_type: Default::default(),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Polygon<DatabaseUnitT> {
    pub fn new(points: Vec<Point<DatabaseUnitT>>, layer: Layer, data_type: DataType) -> Self {
        Self {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        }
    }

    pub fn points(&self) -> &[Point<DatabaseUnitT>] {
        &self.points
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }
}

impl<DatabaseUnitT: CoordNum> std::fmt::Display for Polygon<DatabaseUnitT> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Polygon with {} point(s), starting at ({:?}, {:?}) on layer {:?}, data type {:?}",
            self.points().len(),
            self.points()[0].x(),
            self.points()[0].y(),
            self.layer(),
            self.data_type()
        )
    }
}

impl<DatabaseUnitT: CoordNum> Transformable for Polygon<DatabaseUnitT> {
    fn transform(self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.points = new_self
            .points
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Polygon<DatabaseUnitT> {
    fn move_to(self, target: Point<DatabaseIntegerUnit>) -> Self {
        let first_point = &self.points()[0];
        let delta = Point::new(
            DatabaseIntegerUnit::from_float(target.x().to_float() - first_point.x().to_float()),
            DatabaseIntegerUnit::from_float(target.y().to_float() - first_point.y().to_float()),
        );
        self.move_by(delta)
    }
}

impl<DatabaseUnitT: CoordNum> Dimensions<DatabaseUnitT> for Polygon<DatabaseUnitT> {
    fn bounding_box(&self) -> (Point<DatabaseUnitT>, Point<DatabaseUnitT>) {
        bounding_box(&self.points())
    }
}
