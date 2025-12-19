use std::f64::consts::PI;

use crate::{AngleInRadians, Movable, Point, Transformable, Transformation};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    origin: Point,
    columns: u32,
    rows: u32,
    spacing_x: Option<Point>,
    spacing_y: Option<Point>,
    magnification: f64,
    angle: AngleInRadians,
    x_reflection: bool,
}

impl Grid {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        origin: Point,
        columns: u32,
        rows: u32,
        spacing_x: Option<Point>,
        spacing_y: Option<Point>,
        magnification: f64,
        angle: AngleInRadians,
        x_reflection: bool,
    ) -> Self {
        Self {
            origin,
            columns,
            rows,
            spacing_x,
            spacing_y,
            magnification,
            angle,
            x_reflection,
        }
    }

    pub const fn origin(&self) -> Point {
        self.origin
    }

    pub const fn columns(&self) -> u32 {
        self.columns
    }

    pub const fn rows(&self) -> u32 {
        self.rows
    }

    pub const fn spacing_x(&self) -> Option<Point> {
        self.spacing_x
    }

    pub const fn spacing_y(&self) -> Option<Point> {
        self.spacing_y
    }

    pub const fn magnification(&self) -> f64 {
        self.magnification
    }

    pub const fn angle(&self) -> f64 {
        self.angle
    }

    pub const fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    pub const fn set_origin(&mut self, origin: Point) {
        self.origin = origin;
    }

    #[must_use]
    pub const fn with_origin(mut self, origin: Point) -> Self {
        self.origin = origin;
        self
    }

    pub const fn set_columns(&mut self, columns: u32) {
        self.columns = columns;
    }

    #[must_use]
    pub const fn with_columns(mut self, columns: u32) -> Self {
        self.columns = columns;
        self
    }

    pub const fn set_rows(&mut self, rows: u32) {
        self.rows = rows;
    }

    #[must_use]
    pub const fn with_rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    pub const fn set_spacing_x(&mut self, spacing_x: Option<Point>) {
        self.spacing_x = spacing_x;
    }

    #[must_use]
    pub const fn with_spacing_x(mut self, spacing_x: Option<Point>) -> Self {
        self.spacing_x = spacing_x;
        self
    }

    pub const fn set_spacing_y(&mut self, spacing_y: Option<Point>) {
        self.spacing_y = spacing_y;
    }

    #[must_use]
    pub const fn with_spacing_y(mut self, spacing_y: Option<Point>) -> Self {
        self.spacing_y = spacing_y;
        self
    }

    pub const fn set_magnification(&mut self, magnification: f64) {
        self.magnification = magnification;
    }

    #[must_use]
    pub const fn with_magnification(mut self, magnification: f64) -> Self {
        self.magnification = magnification;
        self
    }

    pub const fn set_angle(&mut self, angle: AngleInRadians) {
        self.angle = angle;
    }

    #[must_use]
    pub const fn with_angle(mut self, angle: AngleInRadians) -> Self {
        self.angle = angle;
        self
    }

    pub const fn set_x_reflection(&mut self, x_reflection: bool) {
        self.x_reflection = x_reflection;
    }

    #[must_use]
    pub const fn with_x_reflection(mut self, x_reflection: bool) -> Self {
        self.x_reflection = x_reflection;
        self
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(
            Point::integer(0, 0, 1e-9),
            1,
            1,
            None,
            None,
            1.0,
            0.0,
            false,
        )
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let spacing_x_str = self
            .spacing_x
            .map_or_else(|| "None".to_string(), |p| p.to_string());
        let spacing_y_str = self
            .spacing_y
            .map_or_else(|| "None".to_string(), |p| p.to_string());
        write!(
            f,
            "Grid at {} with {} columns and {} rows, spacing ({}, {}), magnification {:?}, angle {:?}, x_reflection {}",
            self.origin,
            self.columns,
            self.rows,
            spacing_x_str,
            spacing_y_str,
            self.magnification,
            self.angle,
            self.x_reflection,
        )
    }
}

impl Transformable for Grid {
    fn transform_impl(mut self, transformation: &Transformation) -> Self {
        self.origin = transformation.apply_to_point(&self.origin);
        self.spacing_x = self.spacing_x.map(|p| transformation.apply_to_point(&p));
        self.spacing_y = self.spacing_y.map(|p| transformation.apply_to_point(&p));

        // Apply scale and rotation to grid properties
        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            self.angle += rotation.angle();
            let result = self.angle % (PI * 2.0);
            self.angle = if result < 0.0 {
                result + PI * 2.0
            } else {
                result
            };
        }

        // Handle reflection
        if transformation.reflection.is_some() {
            self.x_reflection = !self.x_reflection;
        }

        self
    }
}

impl Movable for Grid {
    fn move_to(mut self, target: Point) -> Self {
        self.origin = target;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            true,
        );

        assert_eq!(grid.origin, Point::integer(10, 20, 1e-9));
        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows, 4);
        assert_eq!(grid.spacing_x, Some(Point::integer(5, 0, 1e-9)));
        assert_eq!(grid.spacing_y, Some(Point::integer(0, 5, 1e-9)));
        assert_eq!(grid.magnification, 1.5);
        assert_eq!(grid.angle, 45.0);
        assert!(grid.x_reflection);
    }

    #[test]
    fn test_grid_default() {
        let grid: Grid = Grid::default();
        assert_eq!(grid.origin, Point::integer(0, 0, 1e-9));
        assert_eq!(grid.columns, 1);
        assert_eq!(grid.rows, 1);
        assert_eq!(grid.spacing_x, None);
        assert_eq!(grid.spacing_y, None);
        assert_eq!(grid.magnification, 1.0);
        assert_eq!(grid.angle, 0.0);
        assert!(!grid.x_reflection);
    }

    #[test]
    fn test_grid_display() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            3,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        assert_snapshot!(format!("{grid}"), @"Grid at Point(10 (1.000e-9), 20 (1.000e-9)) with 2 columns and 3 rows, spacing (Point(5 (1.000e-9), 0 (1.000e-9)), Point(0 (1.000e-9), 5 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");

        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            3,
            Some(Point::integer(5, 0, 1e-9)),
            None,
            1.0,
            0.0,
            false,
        );

        assert_snapshot!(format!("{grid}"), @"Grid at Point(10 (1.000e-9), 20 (1.000e-9)) with 2 columns and 3 rows, spacing (Point(5 (1.000e-9), 0 (1.000e-9)), None), magnification 1.0, angle 0.0, x_reflection false");

        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            3,
            None,
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        assert_snapshot!(format!("{grid}"), @"Grid at Point(10 (1.000e-9), 20 (1.000e-9)) with 2 columns and 3 rows, spacing (None, Point(0 (1.000e-9), 5 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");
    }

    #[test]
    fn test_grid_clone() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            true,
        );

        let cloned = grid.clone();
        assert_eq!(grid, cloned);
    }

    #[test]
    fn test_grid_partial_eq() {
        let grid1 = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            true,
        );
        let grid2 = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            true,
        );
        let grid3 = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            false,
        );

        assert_eq!(grid1, grid2);
        assert_ne!(grid1, grid3);
    }

    #[test]
    fn test_grid_transform_with_scale() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = grid.scale(2.0, centre);

        assert_eq!(transformed.magnification, 2.0);
    }

    #[test]
    fn test_grid_transform_with_rotation() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = grid.rotate(FRAC_PI_2, centre);

        assert_eq!(transformed.angle, FRAC_PI_2);
    }

    #[test]
    fn test_grid_transform_with_reflection() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = grid.reflect(0.0, centre);

        assert!(transformed.x_reflection);
    }

    #[test]
    fn test_grid_transform_with_translation() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        let delta = Point::integer(5, 5, 1e-9);
        let transformed = grid.translate(delta);

        assert_eq!(transformed.origin, Point::integer(15, 25, 1e-9));
    }

    #[test]
    fn test_grid_move_to() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        let target = Point::integer(100, 200, 1e-9);
        let moved = grid.move_to(target);

        assert_eq!(moved.origin, target);
    }

    #[test]
    fn test_grid_angle_normalization() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            FRAC_PI_2,
            false,
        );

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = grid.rotate(PI * 2.0, centre);

        // 3/2 * PI + PI = 5/2 * PI, which should normalize to PI
        assert!((transformed.angle - FRAC_PI_2).abs() < 0.001);
    }

    #[test]
    fn test_grid_getters() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.5,
            45.0,
            true,
        );

        assert_eq!(grid.origin(), Point::integer(10, 20, 1e-9));
        assert_eq!(grid.columns(), 3);
        assert_eq!(grid.rows(), 4);
        assert_eq!(grid.spacing_x(), Some(Point::integer(5, 0, 1e-9)));
        assert_eq!(grid.spacing_y(), Some(Point::integer(0, 5, 1e-9)));
        assert_eq!(grid.magnification(), 1.5);
        assert_eq!(grid.angle(), 45.0);
        assert!(grid.x_reflection());
    }

    #[test]
    fn test_grid_setters() {
        let mut grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        );

        grid.set_origin(Point::integer(100, 200, 1e-9));
        grid.set_columns(5);
        grid.set_rows(6);
        grid.set_spacing_x(Some(Point::integer(10, 0, 1e-9)));
        grid.set_spacing_y(Some(Point::integer(0, 10, 1e-9)));
        grid.set_magnification(2.0);
        grid.set_angle(90.0);
        grid.set_x_reflection(true);

        assert_eq!(grid.origin, Point::integer(100, 200, 1e-9));
        assert_eq!(grid.columns, 5);
        assert_eq!(grid.rows, 6);
        assert_eq!(grid.spacing_x, Some(Point::integer(10, 0, 1e-9)));
        assert_eq!(grid.spacing_y, Some(Point::integer(0, 10, 1e-9)));
        assert_eq!(grid.magnification, 2.0);
        assert_eq!(grid.angle, 90.0);
        assert!(grid.x_reflection);
    }
    #[test]
    fn test_grid_with_setters() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            2,
            2,
            Some(Point::integer(5, 0, 1e-9)),
            Some(Point::integer(0, 5, 1e-9)),
            1.0,
            0.0,
            false,
        )
        .with_origin(Point::integer(100, 200, 1e-9))
        .with_columns(5)
        .with_rows(6)
        .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
        .with_spacing_y(Some(Point::integer(0, 10, 1e-9)))
        .with_magnification(2.0)
        .with_angle(90.0)
        .with_x_reflection(true);

        assert_eq!(grid.origin, Point::integer(100, 200, 1e-9));
        assert_eq!(grid.columns, 5);
        assert_eq!(grid.rows, 6);
        assert_eq!(grid.spacing_x, Some(Point::integer(10, 0, 1e-9)));
        assert_eq!(grid.spacing_y, Some(Point::integer(0, 10, 1e-9)));
        assert_eq!(grid.magnification, 2.0);
        assert_eq!(grid.angle, 90.0);
        assert!(grid.x_reflection);
    }
}
