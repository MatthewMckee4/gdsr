use std::collections::HashMap;
use std::fmt::Write;

use crate::{
    Cell, DataType, Dimensions, Element, GdsBox, Layer, Library, Node, Path, Point, Polygon,
    Reference, Text,
};

const PALETTE: [(u8, u8, u8); 16] = [
    (230, 25, 75),
    (60, 180, 75),
    (255, 225, 25),
    (0, 130, 200),
    (245, 130, 48),
    (145, 30, 180),
    (70, 240, 240),
    (240, 50, 230),
    (210, 245, 60),
    (250, 190, 212),
    (0, 128, 128),
    (220, 190, 255),
    (170, 110, 40),
    (255, 250, 200),
    (128, 0, 0),
    (128, 128, 0),
];

/// Assigns colors to (layer, datatype) pairs from a fixed palette, matching the viewer.
struct LayerColorMap {
    map: HashMap<(Layer, DataType), (u8, u8, u8)>,
    next_index: usize,
}

impl LayerColorMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            next_index: 0,
        }
    }

    fn get(&mut self, layer: Layer, datatype: DataType) -> (u8, u8, u8) {
        *self.map.entry((layer, datatype)).or_insert_with(|| {
            let color = PALETTE[self.next_index % PALETTE.len()];
            self.next_index += 1;
            color
        })
    }

    fn hex(&mut self, layer: Layer, datatype: DataType) -> String {
        let (r, g, b) = self.get(layer, datatype);
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// Shared context passed to [`ToSvg::to_svg_impl`] implementations.
pub struct SvgContext<'a> {
    colors: LayerColorMap,
    /// The larger of the bounding box width/height, used to scale text and markers.
    pub extent: f64,
    /// The database unit divisor applied to all coordinates.
    pub dbu: f64,
    /// The library used to resolve cell references.
    pub library: &'a Library,
}

impl<'a> SvgContext<'a> {
    fn new(extent: f64, dbu: f64, library: &'a Library) -> Self {
        Self {
            colors: LayerColorMap::new(),
            extent,
            dbu,
            library,
        }
    }

    fn hex(&mut self, layer: Layer, datatype: DataType) -> String {
        self.colors.hex(layer, datatype)
    }

    fn scale(&self, p: &Point) -> (f64, f64) {
        (
            p.x().absolute_value() / self.dbu,
            p.y().absolute_value() / self.dbu,
        )
    }
}

/// Trait for types that can be rendered to SVG elements.
pub trait ToSvg {
    /// Appends SVG element(s) for this value to `out`.
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String);
}

impl ToSvg for Polygon {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let color = ctx.hex(self.layer(), self.data_type());
        let _ = write!(out, "    <polygon points=\"");
        for (i, p) in self.points().iter().enumerate() {
            let (x, y) = ctx.scale(p);
            if i > 0 {
                out.push(' ');
            }
            let _ = write!(out, "{},{}", fmt(x), fmt(y));
        }
        let _ = writeln!(
            out,
            "\" fill=\"{color}\" fill-opacity=\"0.6\" stroke=\"{color}\" stroke-width=\"0\" />"
        );
    }
}

impl ToSvg for Path {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let color = ctx.hex(self.layer(), self.data_type());
        let stroke_width = self
            .width()
            .map_or(ctx.extent * 0.005, |w| w.absolute_value() / ctx.dbu);
        let _ = write!(out, "    <polyline points=\"");
        for (i, p) in self.points().iter().enumerate() {
            let (x, y) = ctx.scale(p);
            if i > 0 {
                out.push(' ');
            }
            let _ = write!(out, "{},{}", fmt(x), fmt(y));
        }
        let _ = writeln!(
            out,
            "\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" />",
            fmt(stroke_width)
        );
    }
}

impl ToSvg for GdsBox {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let color = ctx.hex(self.layer(), self.box_type());
        let (x, y) = ctx.scale(&self.bottom_left());
        let (x2, y2) = ctx.scale(&self.top_right());
        let w = x2 - x;
        let h = y2 - y;
        let _ = writeln!(
            out,
            "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{color}\" fill-opacity=\"0.6\" stroke=\"{color}\" stroke-width=\"0\" />",
            fmt(x),
            fmt(y),
            fmt(w),
            fmt(h)
        );
    }
}

impl ToSvg for Text {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let color = ctx.hex(self.layer(), self.data_type());
        let (x, y) = ctx.scale(self.origin());
        let escaped = escape_xml(self.text());
        let font_size = ctx.extent * 0.03;
        // Counter-flip text so it reads correctly despite the parent Y-flip
        let _ = writeln!(
            out,
            "    <text x=\"{}\" y=\"{}\" fill=\"{color}\" font-size=\"{}\" font-family=\"monospace\" transform=\"scale(1,-1) translate(0,{})\">{escaped}</text>",
            fmt(x),
            fmt(y),
            fmt(font_size),
            fmt(-2.0 * y)
        );
    }
}

impl ToSvg for Node {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let color = ctx.hex(self.layer(), self.node_type());
        let r = ctx.extent * 0.005;
        for p in self.points() {
            let (x, y) = ctx.scale(p);
            let _ = writeln!(
                out,
                "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{color}\" />",
                fmt(x),
                fmt(y),
                fmt(r)
            );
        }
    }
}

impl ToSvg for Reference {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        let flattened = self.clone().flatten(None, ctx.library);
        for element in &flattened {
            element.to_svg_impl(ctx, out);
        }
    }
}

impl ToSvg for Element {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        match self {
            Self::Polygon(v) => v.to_svg_impl(ctx, out),
            Self::Path(v) => v.to_svg_impl(ctx, out),
            Self::Box(v) => v.to_svg_impl(ctx, out),
            Self::Text(v) => v.to_svg_impl(ctx, out),
            Self::Node(v) => v.to_svg_impl(ctx, out),
            Self::Reference(v) => v.to_svg_impl(ctx, out),
        }
    }
}

impl ToSvg for Cell {
    fn to_svg_impl(&self, ctx: &mut SvgContext, out: &mut String) {
        for element in self.iter_elements() {
            element.to_svg_impl(ctx, out);
        }
    }
}

/// Exports a cell to SVG, flattening all references using the library.
///
/// `dbu` is the database unit (e.g. `1e-9` for nanometers) used to scale coordinates
/// so the SVG contains human-readable numbers. For example, a point at 1000 nm
/// with `dbu = 1e-9` produces an SVG coordinate of `1000`.
///
/// Returns the SVG document as a string. The viewport is automatically sized
/// to the cell's bounding box with a small margin.
pub fn cell_to_svg(cell: &Cell, library: &Library, dbu: f64) -> String {
    let elements = cell.get_elements(None, library);
    let (min, max) = bounding_box_of_elements(&elements);
    render_svg(cell, min, max, dbu, library)
}

fn bounding_box_of_elements(elements: &[Element]) -> (Point, Point) {
    let points: Vec<Point> = elements
        .iter()
        .flat_map(|e| {
            let (min, max) = e.bounding_box();
            [min, max]
        })
        .collect();
    if points.is_empty() {
        (Point::default(), Point::default())
    } else {
        crate::geometry::bounding_box(&points)
    }
}

fn render_svg(cell: &Cell, min: Point, max: Point, dbu: f64, library: &Library) -> String {
    let (min_x, min_y) = (
        min.x().absolute_value() / dbu,
        min.y().absolute_value() / dbu,
    );
    let (max_x, max_y) = (
        max.x().absolute_value() / dbu,
        max.y().absolute_value() / dbu,
    );

    let width = max_x - min_x;
    let height = max_y - min_y;

    let margin_x = if width == 0.0 { 1.0 } else { width * 0.05 };
    let margin_y = if height == 0.0 { 1.0 } else { height * 0.05 };

    let vb_x = min_x - margin_x;
    let vb_y = min_y - margin_y;
    let vb_w = width + 2.0 * margin_x;
    let vb_h = height + 2.0 * margin_y;

    let mut out = String::new();
    let _ = writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
        fmt(vb_x),
        fmt(vb_y),
        fmt(vb_w),
        fmt(vb_h)
    );
    // Flip Y axis: GDS is Y-up, SVG is Y-down
    let _ = writeln!(
        out,
        "  <g transform=\"scale(1,-1) translate(0,{})\">",
        fmt(-(vb_y * 2.0 + vb_h))
    );

    let extent = vb_w.max(vb_h);
    let mut ctx = SvgContext::new(extent, dbu, library);
    cell.to_svg_impl(&mut ctx, &mut out);

    let _ = writeln!(out, "  </g>");
    let _ = writeln!(out, "</svg>");
    out
}

/// Formats a float for SVG output with limited precision, stripping trailing zeros.
fn fmt(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    // Round to 6 significant digits via scientific notation round-trip
    let s = format!("{v:.6e}");
    let parsed: f64 = s.parse().unwrap_or(v);
    // Format with enough decimals to show all significant digits
    let mag = parsed.abs().log10().floor() as i32;
    let decimals = (6 - mag).max(0) as usize;
    let mut fixed = format!("{parsed:.decimals$}");
    // Strip trailing zeros after decimal point
    if fixed.contains('.') {
        fixed = fixed.trim_end_matches('0').to_string();
        fixed = fixed.trim_end_matches('.').to_string();
    }
    fixed
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Layer, Point};

    const UNITS: f64 = 1e-6;

    fn p(x: f64, y: f64) -> Point {
        Point::float(x, y, UNITS)
    }

    #[test]
    fn empty_cell_produces_valid_svg() {
        let cell = Cell::new("empty");
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-1 -1 2 2">
          <g transform="scale(1,-1) translate(0,0)">
          </g>
        </svg>
        "#);
    }

    #[test]
    fn polygon_renders_as_svg_polygon() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.5 -0.5 11 11">
          <g transform="scale(1,-1) translate(0,-10)">
            <polygon points="0,0 10,0 10,10 0,0" fill="#e6194b" fill-opacity="0.6" stroke="#e6194b" stroke-width="0" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn path_renders_as_svg_polyline() {
        let mut cell = Cell::new("test");
        cell.add(Path::new(
            vec![p(0.0, 0.0), p(5.0, 5.0)],
            Layer::new(2),
            DataType::new(0),
            None,
            Some(crate::Unit::float(1.0, UNITS)),
            None,
            None,
        ));
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.25 -0.25 5.5 5.5">
          <g transform="scale(1,-1) translate(0,-5)">
            <polyline points="0,0 5,5" fill="none" stroke="#e6194b" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn box_renders_as_svg_rect() {
        let mut cell = Cell::new("test");
        cell.add(GdsBox::new(
            p(0.0, 0.0),
            p(10.0, 5.0),
            Layer::new(1),
            DataType::new(0),
        ));
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.5 -0.25 11 5.5">
          <g transform="scale(1,-1) translate(0,-5)">
            <rect x="0" y="0" width="10" height="5" fill="#e6194b" fill-opacity="0.6" stroke="#e6194b" stroke-width="0" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn text_renders_as_svg_text() {
        let mut cell = Cell::new("test");
        cell.add(
            Text::default()
                .set_text("hello".to_string())
                .set_origin(p(1.0, 2.0)),
        );
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 1 2 2">
          <g transform="scale(1,-1) translate(0,-4)">
            <text x="1" y="2" fill="#e6194b" font-size="0.06" font-family="monospace" transform="scale(1,-1) translate(0,-4)">hello</text>
          </g>
        </svg>
        "##);
    }

    #[test]
    fn node_renders_as_svg_circle() {
        let mut cell = Cell::new("test");
        cell.add(Node::new(
            vec![p(3.0, 4.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="2 3 2 2">
          <g transform="scale(1,-1) translate(0,-8)">
            <circle cx="3" cy="4" r="0.01" fill="#e6194b" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn reference_is_flattened() {
        let mut library = Library::new("lib");

        let mut inner = Cell::new("inner");
        inner.add(Polygon::new(
            [p(0.0, 0.0), p(5.0, 0.0), p(5.0, 5.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(inner);

        let mut top = Cell::new("top");
        top.add(Reference::new("inner".to_string()));
        library.add_cell(top);

        let svg = cell_to_svg(library.get_cell("top").unwrap(), &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.25 -0.25 5.5 5.5">
          <g transform="scale(1,-1) translate(0,-5)">
            <polygon points="0,0 5,0 5,5 0,0" fill="#e6194b" fill-opacity="0.6" stroke="#e6194b" stroke-width="0" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn different_layers_get_different_colors() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Polygon::new(
            [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)],
            Layer::new(2),
            DataType::new(0),
        ));
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.5 -0.5 11 11">
          <g transform="scale(1,-1) translate(0,-10)">
            <polygon points="0,0 10,0 10,10 0,0" fill="#e6194b" fill-opacity="0.6" stroke="#e6194b" stroke-width="0" />
            <polygon points="0,0 10,0 10,10 0,0" fill="#3cb44b" fill-opacity="0.6" stroke="#3cb44b" stroke-width="0" />
          </g>
        </svg>
        "##);
    }

    #[test]
    fn text_xml_escaping() {
        let mut cell = Cell::new("test");
        cell.add(
            Text::default()
                .set_text("<script>&\"test\"</script>".to_string())
                .set_origin(p(0.0, 0.0)),
        );
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        insta::assert_snapshot!(svg, @r##"
        <?xml version="1.0" encoding="UTF-8"?>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="-1 -1 2 2">
          <g transform="scale(1,-1) translate(0,0)">
            <text x="0" y="0" fill="#e6194b" font-size="0.06" font-family="monospace" transform="scale(1,-1) translate(0,0)">&lt;script&gt;&amp;&quot;test&quot;&lt;/script&gt;</text>
          </g>
        </svg>
        "##);
    }

    #[test]
    fn svg_has_y_flip_transform() {
        let cell = Cell::new("empty");
        let library = Library::new("lib");
        let svg = cell_to_svg(&cell, &library, UNITS);
        assert!(svg.contains("scale(1,-1)"));
    }
}
