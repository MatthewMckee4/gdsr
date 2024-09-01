use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    point::Point,
    text::presentation::{HorizontalPresentation, VerticalPresentation},
    traits::{Dimensions, LayerDataTypeMatches, Movable, Rotatable, Scalable},
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

    #[setter(text)]
    fn setter_text(&mut self, text: String) {
        self.text = text;
    }

    fn set_text(mut slf: PyRefMut<'_, Self>, text: String) -> PyRefMut<'_, Self> {
        slf.setter_text(text);
        slf
    }

    #[setter(origin)]
    fn setter_origin(&mut self, #[pyo3(from_py_with = "py_any_to_point")] origin: Point) {
        self.origin = origin;
    }

    fn set_origin(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] origin: Point,
    ) -> PyRefMut<'_, Self> {
        slf.setter_origin(origin);
        slf
    }

    #[setter(layer)]
    fn setter_layer(&mut self, layer: i32) -> PyResult<()> {
        check_layer_valid(layer)?;
        self.layer = layer;
        Ok(())
    }

    fn set_layer(mut slf: PyRefMut<'_, Self>, layer: i32) -> PyRefMut<'_, Self> {
        slf.setter_layer(layer).unwrap();
        slf
    }

    #[setter(magnification)]
    fn setter_magnification(&mut self, magnification: f64) {
        self.magnification = magnification;
    }

    fn set_magnification(mut slf: PyRefMut<'_, Self>, magnification: f64) -> PyRefMut<'_, Self> {
        slf.setter_magnification(magnification);
        slf
    }

    #[setter(angle)]
    fn setter_angle(&mut self, angle: f64) {
        self.angle = angle;
    }

    fn set_angle(mut slf: PyRefMut<'_, Self>, angle: f64) -> PyRefMut<'_, Self> {
        slf.setter_angle(angle);
        slf
    }

    #[setter(x_reflection)]
    fn setter_x_reflection(&mut self, x_reflection: bool) {
        self.x_reflection = x_reflection;
    }

    fn set_x_reflection(mut slf: PyRefMut<'_, Self>, x_reflection: bool) -> PyRefMut<'_, Self> {
        slf.setter_x_reflection(x_reflection);
        slf
    }

    #[setter(vertical_presentation)]
    fn setter_vertical_presentation(&mut self, vertical_presentation: VerticalPresentation) {
        self.vertical_presentation = vertical_presentation;
    }

    fn set_vertical_presentation(
        mut slf: PyRefMut<'_, Self>,
        vertical_presentation: VerticalPresentation,
    ) -> PyRefMut<'_, Self> {
        slf.setter_vertical_presentation(vertical_presentation);
        slf
    }

    #[setter(horizontal_presentation)]
    fn setter_horizontal_presentation(&mut self, horizontal_presentation: HorizontalPresentation) {
        self.horizontal_presentation = horizontal_presentation;
    }

    fn set_horizontal_presentation(
        mut slf: PyRefMut<'_, Self>,
        horizontal_presentation: HorizontalPresentation,
    ) -> PyRefMut<'_, Self> {
        slf.setter_horizontal_presentation(horizontal_presentation);
        slf
    }

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
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

    #[pyo3(signature = (*layer_data_types))]
    pub fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        LayerDataTypeMatches::is_on(self, layer_data_types)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
