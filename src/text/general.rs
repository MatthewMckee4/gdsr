use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    point::Point,
    text::presentation::{HorizontalPresentation, VerticalPresentation},
    traits::{Movable, Rotatable, Scalable},
    utils::transformations::py_any_to_point,
    validation::input::check_layer_valid,
};

use super::Text;

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (
        text,
        origin=Point::default(),
        layer=0,
        magnification=1.0,
        angle=0.0,
        x_reflection=false,
        vertical_presentation=VerticalPresentation::default(),
        horizontal_presentation=HorizontalPresentation::default()
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

    pub fn copy(&self) -> Self {
        self.clone()
    }

    fn move_to(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_to(slf.deref_mut(), point);
        slf
    }

    fn move_by(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_by(slf.deref_mut(), vector);
        slf
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    fn rotate(
        mut slf: PyRefMut<'_, Self>,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Rotatable::rotate(slf.deref_mut(), angle, centre);
        slf
    }

    #[pyo3(signature = (factor, centre=Point::default()))]
    fn scale(
        mut slf: PyRefMut<'_, Self>,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Scalable::scale(slf.deref_mut(), factor, centre);
        slf
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
