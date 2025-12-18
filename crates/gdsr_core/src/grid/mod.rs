use crate::{AngleInRadians, Movable, Point, Transformable, Transformation};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    origin: Point,
    columns: u32,
    rows: u32,
    spacing_x: Point,
    spacing_y: Point,
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
        spacing_x: Point,
        spacing_y: Point,
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

    pub const fn spacing_x(&self) -> Point {
        self.spacing_x
    }

    pub const fn spacing_y(&self) -> Point {
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

    pub const fn set_columns(&mut self, columns: u32) {
        self.columns = columns;
    }

    pub const fn set_rows(&mut self, rows: u32) {
        self.rows = rows;
    }

    pub const fn set_spacing_x(&mut self, spacing_x: Point) {
        self.spacing_x = spacing_x;
    }

    pub const fn set_spacing_y(&mut self, spacing_y: Point) {
        self.spacing_y = spacing_y;
    }

    pub const fn set_magnification(&mut self, magnification: f64) {
        self.magnification = magnification;
    }

    pub const fn set_angle(&mut self, angle: AngleInRadians) {
        self.angle = angle;
    }

    pub const fn set_x_reflection(&mut self, x_reflection: bool) {
        self.x_reflection = x_reflection;
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(
            Point::integer(0, 0, 1e-9),
            1,
            1,
            Point::integer(0, 0, 1e-9),
            Point::integer(0, 0, 1e-9),
            1.0,
            0.0,
            false,
        )
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid at {} with {} columns and {} rows, spacing ({}, {}), magnification {:?}, angle {:?}, x_reflection {}",
            self.origin,
            self.columns,
            self.rows,
            self.spacing_x,
            self.spacing_y,
            self.magnification,
            self.angle,
            self.x_reflection,
        )
    }
}

impl Transformable for Grid {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.origin = transformation.apply_to_point(&new_self.origin);
        new_self.spacing_x = transformation.apply_to_point(&new_self.spacing_x);
        new_self.spacing_y = transformation.apply_to_point(&new_self.spacing_y);

        // Apply scale and rotation to grid properties
        if let Some(scale) = &transformation.scale {
            new_self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            new_self.angle += rotation.angle();
            let result = new_self.angle % 360.0;
            new_self.angle = if result < 0.0 { result + 360.0 } else { result };
        }

        // Handle reflection
        if transformation.reflection.is_some() {
            new_self.x_reflection = !new_self.x_reflection;
        }

        new_self
    }
}

impl Movable for Grid {
    fn move_to(&self, target: Point) -> Self {
        let mut new_self = self.clone();
        new_self.origin = target;
        new_self
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
            1.5,
            45.0,
            true,
        );

        assert_eq!(grid.origin, Point::integer(10, 20, 1e-9));
        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows, 4);
        assert_eq!(grid.spacing_x, Point::integer(5, 0, 1e-9));
        assert_eq!(grid.spacing_y, Point::integer(0, 5, 1e-9));
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
        assert_eq!(grid.spacing_x, Point::integer(0, 0, 1e-9));
        assert_eq!(grid.spacing_y, Point::integer(0, 0, 1e-9));
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
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
            1.0,
            0.0,
            false,
        );

        assert_snapshot!(format!("{grid}"), @"Grid at Point(10 (1.000e-9), 20 (1.000e-9)) with 2 columns and 3 rows, spacing (Point(5 (1.000e-9), 0 (1.000e-9)), Point(0 (1.000e-9), 5 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");
    }

    #[test]
    fn test_grid_clone() {
        let grid = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
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
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
            1.5,
            45.0,
            true,
        );
        let grid2 = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
            1.5,
            45.0,
            true,
        );
        let grid3 = Grid::new(
            Point::integer(10, 20, 1e-9),
            3,
            4,
            Point::integer(5, 0, 1e-9),
            Point::integer(0, 5, 1e-9),
            1.5,
            45.0,
            false,
        );

        assert_eq!(grid1, grid2);
        assert_ne!(grid1, grid3);
    }
}
