use std::collections::HashMap;

use quickcheck_macros::quickcheck;

use crate::*;

#[quickcheck]
fn remap_polygon_identity(polygon: Polygon) -> bool {
    let mut remapped = polygon.clone();
    remapped.remap_layers(&HashMap::new());
    remapped == polygon
}

#[quickcheck]
fn remap_polygon_applies_mapping(polygon: Polygon) -> bool {
    let old = (polygon.layer(), polygon.data_type());
    let new = (Layer::new(200), DataType::new(201));
    let mapping: LayerMapping = [(old, new)].into_iter().collect();
    let mut remapped = polygon.clone();
    remapped.remap_layers(&mapping);
    remapped.layer() == new.0 && remapped.data_type() == new.1
}

#[quickcheck]
fn remap_path_applies_mapping(path: Path) -> bool {
    let old = (path.layer(), path.data_type());
    let new = (Layer::new(100), DataType::new(99));
    let mapping: LayerMapping = [(old, new)].into_iter().collect();
    let mut remapped = path.clone();
    remapped.remap_layers(&mapping);
    remapped.layer() == new.0 && remapped.data_type() == new.1
}

#[quickcheck]
fn remap_gds_box_applies_mapping(gds_box: GdsBox) -> bool {
    let old = (gds_box.layer(), gds_box.box_type());
    let new = (Layer::new(50), DataType::new(51));
    let mapping: LayerMapping = [(old, new)].into_iter().collect();
    let mut remapped = gds_box.clone();
    remapped.remap_layers(&mapping);
    remapped.layer() == new.0 && remapped.box_type() == new.1
}

#[quickcheck]
fn remap_preserves_geometry(polygon: Polygon) -> bool {
    let mapping: LayerMapping = [(
        (polygon.layer(), polygon.data_type()),
        (Layer::new(42), DataType::new(43)),
    )]
    .into_iter()
    .collect();
    let mut remapped = polygon.clone();
    remapped.remap_layers(&mapping);
    remapped.points() == polygon.points()
}

#[quickcheck]
fn remap_unmatched_is_noop(polygon: Polygon) -> bool {
    let mapping: LayerMapping = [(
        (Layer::new(60000), DataType::new(60001)),
        (Layer::new(1), DataType::new(2)),
    )]
    .into_iter()
    .collect();
    let mut remapped = polygon.clone();
    remapped.remap_layers(&mapping);
    remapped.layer() == polygon.layer() && remapped.data_type() == polygon.data_type()
}
