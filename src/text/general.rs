use pyo3::prelude::*;

use crate::{
    point::Point,
    text::presentation::{HorizontalPresentation, VerticalPresentation},
    traits::Movable,
    validation::input::check_layer_valid,
};

use super::Text;

use crate::point::utils::*;

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (
        text,
        origin=Point { x: 0.0, y: 0.0 },
        layer=0,
        magnification=1.0,
        angle=0.0,
        x_reflection=false,
        vertical_presentation=VerticalPresentation::Middle,
        horizontal_presentation=HorizontalPresentation::Centre
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        text: String,
        #[pyo3(from_py_with = "py_any_to_point")] origin: Point,
        layer: i32,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
        vertical_presentation: VerticalPresentation,
        horizontal_presentation: HorizontalPresentation,
    ) -> PyResult<Self> {
        check_layer_valid(layer)?;

        Ok(Text {
            text,
            origin,
            layer,
            magnification,
            angle,
            x_reflection,
            vertical_presentation,
            horizontal_presentation,
        })
    }

    #[setter]
    fn set_origin(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_point")] origin: Point,
    ) -> PyResult<()> {
        self.origin = origin;
        Ok(())
    }

    #[setter]
    fn set_layer(&mut self, layer: i32) -> PyResult<()> {
        check_layer_valid(layer)?;
        self.layer = layer;
        Ok(())
    }

    fn copy(&self) -> PyResult<Self> {
        Ok(self.clone())
    }

    fn move_to(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyResult<Self> {
        Movable::move_to(self, point);
        Ok(self.clone())
    }

    fn move_by(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyResult<Self> {
        Movable::move_by(self, vector);
        Ok(self.clone())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
