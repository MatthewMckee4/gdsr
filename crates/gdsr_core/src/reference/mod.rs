use crate::{
    Point, Rotatable,
    grid::Grid,
    traits::{Dimensions, Transformable},
    transformation::Transformation,
};

pub mod instance;

pub use instance::Instance;

#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub instance: Instance,
    pub grid: Grid,
}

impl Reference {
    pub fn new(instance: Instance, grid: Grid) -> Self {
        Self { instance, grid }
    }
}

impl Transformable for Reference {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.grid.transform(transformation);
        self
    }
}

impl Dimensions for Reference {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let grid = &self.grid;

        let corners = vec![
            grid.origin,
            Point::new(
                grid.origin.x() + grid.spacing_x * grid.columns as f64,
                grid.origin.y(),
            ),
            Point::new(
                grid.origin.x(),
                grid.origin.y() + grid.spacing_y * grid.rows as f64,
            ),
            Point::new(
                grid.origin.x() + grid.spacing_x * grid.columns as f64,
                grid.origin.y() + grid.spacing_y * grid.rows as f64,
            ),
        ];

        for corner in corners {
            let mut new_instance = self.instance.clone();

            let mut transformation = Transformation::new();
            transformation = transformation
                .with_scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin)
                .with_scale(grid.magnification, grid.origin)
                .with_rotation(grid.angle, grid.origin)
                .with_translation(Point::new(
                    corner.x() - grid.origin.x(),
                    corner.y() - grid.origin.y(),
                ));
            let grid = Grid::default().transform(&transformation);

            let mut reference = Reference::new(new_instance, *grid);

            let (new_instance_min, new_instance_max) = reference.bounding_box();

            min_x = min_x.min(new_instance_min.x());
            min_y = min_y.min(new_instance_min.y());
            max_x = max_x.max(new_instance_max.x());
            max_y = max_y.max(new_instance_max.y());
        }

        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }
}
