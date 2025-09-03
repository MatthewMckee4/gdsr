use crate::{
    CoordNum, DataType, DatabaseIntegerUnit, Layer, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
mod utils;

#[derive(Clone, Default, Debug, PartialEq)]
struct PolygonInner<DatabaseUnitT: CoordNum> {
    points: Vec<Point<DatabaseUnitT>>,
    layer: Layer,
    data_type: DataType,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Polygon<DatabaseUnitT: CoordNum>(PolygonInner<DatabaseUnitT>);

impl<DatabaseUnitT: CoordNum> Polygon<DatabaseUnitT> {
    pub fn new(points: Vec<Point<DatabaseUnitT>>, layer: Layer, data_type: DataType) -> Self {
        Self(PolygonInner {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        })
    }

    pub fn points(&self) -> &[Point<DatabaseUnitT>] {
        &self.0.points
    }

    pub fn layer(&self) -> Layer {
        self.0.layer
    }

    pub fn data_type(&self) -> DataType {
        self.0.data_type
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

impl Transformable for Polygon<DatabaseIntegerUnit> {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.0.points = self
            .0
            .points
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        self
    }
}

impl Movable for Polygon<DatabaseIntegerUnit> {
    fn move_to(&mut self, target: Point<DatabaseIntegerUnit>) -> &mut Self {
        let first_point = &self.0.points[0];
        let delta = Point::new(target.x() - first_point.x(), target.y() - first_point.y());
        self.move_by(delta)
    }
}

impl<DatabaseUnitT: CoordNum> Dimensions<DatabaseUnitT> for Polygon<DatabaseUnitT> {
    fn bounding_box(&self) -> (Point<DatabaseUnitT>, Point<DatabaseUnitT>) {
        bounding_box(&self.0.points)
    }
}
