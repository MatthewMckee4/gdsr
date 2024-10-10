use pyo3::prelude::*;
use std::fs::File;

use crate::{boolean::ExternalPolygonGroup, point::Point};

pub trait ToGds {
    fn _to_gds(&self, file: File, scale: f64) -> PyResult<File>;
}

pub trait Movable {
    fn move_to(&mut self, point: Point) -> &mut Self;
    fn move_by(&mut self, vector: Point) -> &mut Self;
}

pub trait Rotatable {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self;
}

pub trait Scalable {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self;
}

pub trait Dimensions {
    fn bounding_box(&self) -> (Point, Point);
}

pub trait Reflect {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self;
}

pub trait LayerDataTypeMatches {
    fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool;
}

pub trait Simplifiable {
    fn simplify(&mut self) -> &mut Self;
}

pub trait ToExternalPolygonGroup {
    fn to_external_polygon_group(&self) -> PyResult<ExternalPolygonGroup>;
}

pub trait FromExternalPolygonGroup {
    fn from_external_polygon_group(
        external_polygon_group: ExternalPolygonGroup,
        layer: i32,
        data_type: i32,
    ) -> PyResult<Vec<Self>>
    where
        Self: std::marker::Sized;
}
