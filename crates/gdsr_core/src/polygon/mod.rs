use crate::{
    CoordNum, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
mod utils;

#[derive(Clone, Default, PartialEq)]
pub struct Polygon<T: CoordNum> {
    pub points: Vec<Point<T>>,
    pub layer: i32,
    pub data_type: i32,
}

impl<T: CoordNum> Polygon<T> {
    pub fn new(points: Vec<Point<T>>, layer: i32, data_type: i32) -> Self {
        Self {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        }
    }
}

impl<T: CoordNum> std::fmt::Display for Polygon<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Polygon with {} point(s), starting at ({:?}, {:?}) on layer {:?}, data type {:?}",
            self.points.len(),
            self.points[0].x(),
            self.points[0].y(),
            self.layer,
            self.data_type
        )
    }
}

impl<T: CoordNum> std::fmt::Debug for Polygon<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.points.as_slice() {
            [] => {
                write!(f, "Polygon([], {:?}, {:?})", self.layer, self.data_type)
            }
            [first_point] => write!(
                f,
                "Polygon([({:?}, {:?}), ..., ({:?}, {:?})], {:?}, {:?})",
                first_point.x(),
                first_point.y(),
                first_point.x(),
                first_point.y(),
                self.layer,
                self.data_type
            ),
            [first_point, _] => write!(
                f,
                "Polygon([({:?}, {:?}), ..., ({:?}, {:?})], {:?}, {:?})",
                first_point.x(),
                first_point.y(),
                first_point.x(),
                first_point.y(),
                self.layer,
                self.data_type
            ),
            [first_point, .., second_last_point, _] => write!(
                f,
                "Polygon([({:?}, {:?}), ..., ({:?}, {:?})], {:?}, {:?})",
                first_point.x(),
                first_point.y(),
                second_last_point.x(),
                second_last_point.y(),
                self.layer,
                self.data_type
            ),
        }
    }
}

impl<T: CoordNum> Transformable for Polygon<T> {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.points = self
            .points
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        self
    }
}

impl<T: CoordNum> Movable for Polygon<T> {
    fn move_to(&mut self, target: Point<T>) -> &mut Self {
        let first_point = &self.points[0];
        let delta = Point::new(target.x() - first_point.x(), target.y() - first_point.y());
        self.move_by(delta)
    }
}

impl<T: CoordNum> Dimensions for Polygon<T> {
    fn bounding_box(&self) -> (Point<T>, Point<T>) {
        bounding_box(&self.points)
    }
}
