pub mod helpers {
    use gdsr::{
        DataType, Element, HorizontalPresentation, Layer, Path, Point, Polygon, Text, Unit,
        VerticalPresentation,
    };

    pub fn polygon(points: Vec<(i32, i32)>, layer: u16, data_type: u16) -> Element {
        Element::Polygon(Polygon::new(
            points
                .into_iter()
                .map(|(x, y)| Point::default_integer(x, y)),
            Layer::new(layer),
            DataType::new(data_type),
        ))
    }

    pub fn path(
        points: Vec<(i32, i32)>,
        layer: u16,
        data_type: u16,
        width: Option<i32>,
    ) -> Element {
        Element::Path(Path::new(
            points
                .into_iter()
                .map(|(x, y)| Point::default_integer(x, y)),
            Layer::new(layer),
            DataType::new(data_type),
            None,
            width.map(Unit::default_integer),
            None,
            None,
        ))
    }

    pub fn text(value: &str, x: i32, y: i32, layer: u16) -> Element {
        Element::Text(Text::new(
            value,
            Point::default_integer(x, y),
            Layer::new(layer),
            DataType::new(0),
            1.0,
            0.0,
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        ))
    }

    pub fn reference() -> Element {
        Element::Reference(gdsr::Reference::default())
    }
}
