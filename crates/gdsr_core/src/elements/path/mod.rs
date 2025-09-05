use path_type::PathType;

use crate::{
    CoordNum, DataType, DatabaseFloatUnit, DatabaseIntegerUnit, Layer, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
pub mod path_type;

pub(crate) type Width = f64;

#[derive(Clone, Debug, PartialEq)]
pub struct Path<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) points: Vec<Point<DatabaseUnitT>>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
    pub(crate) r#type: Option<PathType>,
    pub(crate) width: Option<Width>,
}

impl<DatabaseUnitT: CoordNum> Default for Path<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            points: Vec::default(),
            layer: Default::default(),
            data_type: Default::default(),
            r#type: None,
            width: None,
        }
    }
}

impl<DatabaseUnitT: CoordNum> Path<DatabaseUnitT> {
    pub fn new(
        points: Vec<Point<DatabaseUnitT>>,
        layer: Layer,
        data_type: DataType,
        path_type: Option<PathType>,
        width: Option<Width>,
    ) -> Self {
        Self {
            points,
            layer,
            data_type,
            r#type: path_type,
            width,
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

    pub fn path_type(&self) -> &Option<PathType> {
        &self.r#type
    }

    pub fn width(&self) -> Option<Width> {
        self.width
    }
}

impl<DatabaseUnitT: CoordNum> std::fmt::Display for Path<DatabaseUnitT> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Path with {} points on layer {} with data type {}, {:?} and width {}",
            self.points().len(),
            self.layer(),
            self.data_type(),
            self.path_type().unwrap_or_default(),
            self.width().unwrap_or_default()
        )
    }
}

impl<DatabaseUnitT: CoordNum> Transformable for Path<DatabaseUnitT> {
    fn transform(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.points = new_self
            .points()
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Path<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let first_point = &self.points()[0];
        let delta = Point::new(
            DatabaseIntegerUnit::from_float(target.x().to_float() - first_point.x().to_float()),
            DatabaseIntegerUnit::from_float(target.y().to_float() - first_point.y().to_float()),
        );
        self.move_by(delta)
    }
}

impl Dimensions<DatabaseFloatUnit> for Path<DatabaseIntegerUnit> {
    fn bounding_box(&self) -> (Point<DatabaseFloatUnit>, Point<DatabaseFloatUnit>) {
        let to_database_float = |point: Point<DatabaseIntegerUnit>| {
            Point::new(
                point.x() as DatabaseFloatUnit,
                point.y() as DatabaseFloatUnit,
            )
        };

        if let Some(width) = self.width()
            && width > 0.0
        {
            // For paths with width, we need to consider the extended points
            let half_width = width / 2.0;

            // Create extended points considering the width
            let mut extended_points = Vec::new();

            for i in 0..self.points().len() {
                let point = to_database_float(self.points()[i]);

                // Add points offset by half-width in perpendicular directions
                if i > 0 {
                    let prev = to_database_float(self.points()[i - 1]);

                    let dx = point.x() - prev.x();
                    let dy = point.y() - prev.y();
                    let len = ((dx * dx + dy * dy) as f64).sqrt();
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }

                if i < self.points().len() - 1 {
                    let next = to_database_float(self.points()[i + 1]);

                    let dx = next.x() - point.x();
                    let dy = next.y() - point.y();
                    let len = (dx * dx + dy * dy).sqrt();
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }
            }

            bounding_box(&extended_points)
        } else {
            // For paths without width, use the standard bounding box
            let (bottom_left, bottom_right) = bounding_box(self.points());

            (
                to_database_float(bottom_left),
                to_database_float(bottom_right),
            )
        }
    }
}
