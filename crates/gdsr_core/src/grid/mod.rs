use crate::{Movable, Point, Transformable, Transformation};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub origin: Point,
    pub columns: u32,
    pub rows: u32,
    pub spacing_x: Point,
    pub spacing_y: Point,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
}

impl Grid {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: impl Into<Point>,
        columns: u32,
        rows: u32,
        spacing_x: impl Into<Point>,
        spacing_y: impl Into<Point>,
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

impl Default for Grid {
    fn default() -> Self {
        Self::new((0, 0), 1, 1, (0, 0), (0, 0), 1.0, 0.0, false)
    }
}

impl std::fmt::Display for Grid {
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
    use super::*;

    #[test]
    fn test_grid_new() {
        let grid = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);

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
    fn test_grid_partial_eq() {
        let grid1 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);
        let grid2 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, true);
        let grid3 = Grid::new((10, 20), 3, 4, (5, 0), (0, 5), 1.5, 45.0, false);

        assert_eq!(grid1, grid2);
        assert_ne!(grid1, grid3);
    }
}
