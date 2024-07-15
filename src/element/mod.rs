use pyo3::FromPyObject;

use crate::array_reference::ArrayReference;
use crate::node::Node;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::r#box::Box;
use crate::reference::Reference;
use crate::text::Text;

#[derive(FromPyObject)]
pub enum Element {
    ArrayReference(ArrayReference),
    Box(Box),
    Node(Node),
    Path(Path),
    Polygon(Polygon),
    Reference(Reference),
    Text(Text),
}
