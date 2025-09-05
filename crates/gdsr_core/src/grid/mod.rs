use crate::{
    CoordNum, DatabaseIntegerUnit, Point,
    traits::{Movable, Transformable},
    transformation::Transformation,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) origin: Point<DatabaseUnitT>,
    pub(crate) columns: u32,
    pub(crate) rows: u32,
    pub(crate) spacing_x: Point<DatabaseUnitT>,
    pub(crate) spacing_y: Point<DatabaseUnitT>,
    pub(crate) magnification: f64,
    pub(crate) angle: f64,
    pub(crate) x_reflection: bool,
}

impl<DatabaseUnitT: CoordNum> Grid<DatabaseUnitT> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: impl Into<Point<DatabaseUnitT>>,
        columns: u32,
        rows: u32,
        spacing_x: impl Into<Point<DatabaseUnitT>>,
        spacing_y: impl Into<Point<DatabaseUnitT>>,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
    ) -> Self {
        Self {
            origin: origin.into(),
            columns,
            rows,
            spacing_x: spacing_x.into(),
            spacing_y: spacing_y.into(),
            magnification,
            angle,
            x_reflection,
        }
    }
}

impl<T: CoordNum> Default for Grid<T> {
    fn default() -> Self {
        Self {
            origin: Point::new(T::zero(), T::zero()),
            columns: 1,
            rows: 1,
            spacing_x: Point::new(T::zero(), T::zero()),
            spacing_y: Point::new(T::zero(), T::zero()),
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
        }
    }
}

impl<T: CoordNum> std::fmt::Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid at {:?} with {} columns and {} rows, spacing ({:?}, {:?}), magnification {:?}, angle {:?}, x_reflection {}",
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

impl<DatabaseUnitT: CoordNum> Transformable for Grid<DatabaseUnitT> {
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

impl<DatabaseUnitT: CoordNum> Movable for Grid<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let mut new_self = self.clone();
        new_self.origin = Point::new(
            DatabaseUnitT::from_float(target.x().to_float()),
            DatabaseUnitT::from_float(target.y().to_float()),
        );
        new_self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);

        assert_eq!(grid.origin, Point::new(10, 20));
        assert_eq!(grid.columns, 3);
        assert_eq!(grid.rows, 4);
        assert_eq!(grid.spacing_x, Point::new(5, 0));
        assert_eq!(grid.spacing_y, Point::new(0, 5));
        assert_eq!(grid.magnification, 1.5);
        assert_eq!(grid.angle, 45.0);
        assert!(grid.x_reflection);
    }

    #[test]
    fn test_grid_default() {
        let grid: Grid = Grid::default();
        assert_eq!(grid.origin, Point::new(0, 0));
        assert_eq!(grid.columns, 1);
        assert_eq!(grid.rows, 1);
        assert_eq!(grid.spacing_x, Point::new(0, 0));
        assert_eq!(grid.spacing_y, Point::new(0, 0));
        assert_eq!(grid.magnification, 1.0);
        assert_eq!(grid.angle, 0.0);
        assert!(!grid.x_reflection);
    }

    #[test]
    fn test_grid_display() {
        let grid = Grid::new((10, 20), 2, 3, (5, 0), (0, 5), 1.0, 0.0, false);

        let display_str = format!("{grid}");
        assert!(display_str.contains("Grid at"));
        assert!(display_str.contains("2 columns"));
        assert!(display_str.contains("3 rows"));
        assert!(display_str.contains("magnification 1"));
        assert!(display_str.contains("angle 0"));
        assert!(display_str.contains("x_reflection false"));
    }

    #[test]
    fn test_grid_clone() {
        let grid = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);

        let cloned = grid.clone();
        assert_eq!(grid, cloned);
    }

    #[test]
    fn test_grid_translate() {
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let translated = grid.translate(Point::new(5, 5));
        assert_eq!(translated.origin, Point::new(5, 5));
        assert_eq!(translated.columns, 2);
        assert_eq!(translated.rows, 2);
    }

    #[test]
    fn test_grid_move_to() {
        let grid = Grid::new((10, 10), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let moved = grid.move_to(Point::new(20, 30));
        assert_eq!(moved.origin, Point::new(20, 30));
        assert_eq!(moved.columns, 2);
        assert_eq!(moved.rows, 2);
    }

    #[test]
    fn test_grid_move_by() {
        let grid = Grid::new((10, 10), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let moved = grid.move_by(Point::new(5, 5));
        assert_eq!(moved.origin, Point::new(15, 15));
        assert_eq!(moved.columns, 2);
        assert_eq!(moved.rows, 2);
    }

    #[test]
    fn test_grid_rotate() {
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let rotated = grid.rotate(90.0, Point::new(0, 0));
        assert_eq!(rotated.angle, 90.0);
        assert_eq!(rotated.columns, 2);
        assert_eq!(rotated.rows, 2);
    }

    #[test]
    fn test_grid_rotate_angle_normalization() {
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 45.0, false);

        let rotated_450 = grid.rotate(450.0, Point::new(0, 0));
        assert_eq!(rotated_450.angle, 135.0);

        let rotated_negative = grid.rotate(-45.0, Point::new(0, 0));
        assert_eq!(rotated_negative.angle, 0.0);
    }

    #[test]
    fn test_grid_scale() {
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let scaled = grid.scale(2.0, Point::new(0, 0));
        assert_eq!(scaled.magnification, 2.0);
        assert_eq!(scaled.columns, 2);
        assert_eq!(scaled.rows, 2);
    }

    #[test]
    fn test_grid_reflect() {
        let grid = Grid::new((0, 0), 2, 2, (10, 0), (0, 10), 1.0, 0.0, false);

        let reflected = grid.reflect(0.0, Point::new(0, 0));
        assert!(reflected.x_reflection);

        let double_reflected = reflected.reflect(0.0, Point::new(0, 0));
        assert!(!double_reflected.x_reflection);
    }

    #[test]
    fn test_grid_partial_eq() {
        let grid1 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);

        let grid2 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);

        let grid3 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, false);

        assert_eq!(grid1, grid2);
        assert_ne!(grid1, grid3);
    }
}
