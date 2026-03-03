#![allow(dead_code)]

use crate::elements::{Path, Polygon, Reference, Text};
use crate::{Grid, Point};

pub const UNITS: f64 = 1e-9;

pub fn p(x: i32, y: i32) -> Point {
    Point::integer(x, y, UNITS)
}

pub fn pf(x: f64, y: f64) -> Point {
    Point::float(x, y, 1e-6)
}

pub fn origin() -> Point {
    p(0, 0)
}

pub fn triangle_points() -> Vec<Point> {
    vec![p(0, 0), p(10, 0), p(10, 10)]
}

pub fn square_points() -> Vec<Point> {
    vec![p(0, 0), p(10, 0), p(10, 10), p(0, 10)]
}

pub fn simple_polygon() -> Polygon {
    Polygon::new(triangle_points(), 1, 0)
}

pub fn square_polygon() -> Polygon {
    Polygon::new(square_points(), 1, 0)
}

pub fn simple_path() -> Path {
    Path::new(vec![p(0, 0), p(10, 10)], 1, 0, None, None)
}

pub fn simple_grid() -> Grid {
    Grid::default()
        .with_columns(2)
        .with_rows(2)
        .with_spacing_x(Some(p(10, 0)))
        .with_spacing_y(Some(p(0, 10)))
}

pub fn simple_reference() -> Reference {
    Reference::new(simple_polygon())
}

pub fn simple_text() -> Text {
    Text::default()
}
