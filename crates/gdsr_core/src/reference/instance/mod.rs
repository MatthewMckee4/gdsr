use crate::{Path, Polygon, Text, cell::Cell};

#[derive(Clone, Debug, PartialEq)]
pub enum Instance {
    Cell(Cell),
    Text(Text),
    Polygon(Polygon),
    Path(Path),
}
