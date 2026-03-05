use quickcheck::{Arbitrary, Gen};

use crate::*;

const MAX_VALUE: i32 = 10_000;
const MAX_ANGLE: f64 = std::f64::consts::TAU;
const MAX_SCALE: f64 = 100.0;
const MAX_COORD: i32 = 10_000;
const MIN_POLYGON_VERTICES: usize = 3;
const MAX_EXTRA_VERTICES: usize = 20;
const MIN_PATH_POINTS: usize = 2;
const MAX_EXTRA_POINTS: usize = 18;

impl Arbitrary for units::IntegerUnit {
    fn arbitrary(g: &mut Gen) -> Self {
        let value = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];
        Self { value, units }
    }
}

impl Arbitrary for units::FloatUnit {
    fn arbitrary(g: &mut Gen) -> Self {
        let raw_value = f64::arbitrary(g);
        let value = if raw_value.is_finite() {
            (raw_value % f64::from(MAX_VALUE)).clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
        } else {
            0.0
        };
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];
        Self { value, units }
    }
}

impl Arbitrary for Unit {
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Self::Integer(units::IntegerUnit::arbitrary(g))
        } else {
            Self::Float(units::FloatUnit::arbitrary(g))
        }
    }
}

impl Arbitrary for Point {
    fn arbitrary(g: &mut Gen) -> Self {
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];

        if bool::arbitrary(g) {
            let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
            Self::integer(x, y, units)
        } else {
            let raw_x = f64::arbitrary(g);
            let raw_y = f64::arbitrary(g);
            let x = if raw_x.is_finite() {
                (raw_x % f64::from(MAX_VALUE)).clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
            } else {
                0.0
            };
            let y = if raw_y.is_finite() {
                (raw_y % f64::from(MAX_VALUE)).clamp(f64::from(-MAX_VALUE), f64::from(MAX_VALUE))
            } else {
                0.0
            };
            Self::float(x, y, units)
        }
    }
}

impl Arbitrary for Reflection {
    fn arbitrary(g: &mut Gen) -> Self {
        let raw_angle = f64::arbitrary(g);
        let angle = if raw_angle.is_finite() {
            raw_angle % MAX_ANGLE
        } else {
            0.0
        };
        let centre = Point::integer(
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            1e-9,
        );
        Self::new(angle, centre)
    }
}

impl Arbitrary for Rotation {
    fn arbitrary(g: &mut Gen) -> Self {
        let raw_angle = f64::arbitrary(g);
        let angle = if raw_angle.is_finite() {
            raw_angle % MAX_ANGLE
        } else {
            0.0
        };
        let centre = Point::integer(
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            1e-9,
        );
        Self::new(angle, centre)
    }
}

impl Arbitrary for Scale {
    fn arbitrary(g: &mut Gen) -> Self {
        let raw_factor = f64::arbitrary(g);
        let factor = if raw_factor.is_finite() {
            (raw_factor % MAX_SCALE).clamp(-MAX_SCALE, MAX_SCALE)
        } else {
            1.0
        };
        let centre = Point::integer(
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            1e-9,
        );
        Self::new(factor, centre)
    }
}

impl Arbitrary for Translation {
    fn arbitrary(g: &mut Gen) -> Self {
        let delta = Point::integer(
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD),
            1e-9,
        );
        Self::new(delta)
    }
}

impl Arbitrary for Transformation {
    fn arbitrary(g: &mut Gen) -> Self {
        let reflection = if bool::arbitrary(g) {
            Some(Reflection::arbitrary(g))
        } else {
            None
        };
        let rotation = if bool::arbitrary(g) {
            Some(Rotation::arbitrary(g))
        } else {
            None
        };
        let scale = if bool::arbitrary(g) {
            Some(Scale::arbitrary(g))
        } else {
            None
        };
        let translation = if bool::arbitrary(g) {
            Some(Translation::arbitrary(g))
        } else {
            None
        };
        let mut t = Self::default();
        t.with_reflection(reflection);
        t.with_rotation(rotation);
        t.with_scale(scale);
        t.with_translation(translation);
        t
    }
}

impl Arbitrary for Polygon {
    fn arbitrary(g: &mut Gen) -> Self {
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];
        let num_vertices = MIN_POLYGON_VERTICES + (usize::arbitrary(g) % (MAX_EXTRA_VERTICES + 1));
        let points: Vec<Point> = (0..num_vertices)
            .map(|_| {
                let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                Point::integer(x, y, units)
            })
            .collect();
        let layer = Layer::new(u16::arbitrary(g));
        let data_type = DataType::new(u16::arbitrary(g));
        Self::new(points, layer, data_type)
    }
}

impl Arbitrary for PathType {
    fn arbitrary(g: &mut Gen) -> Self {
        let types = [Self::Square, Self::Round, Self::Overlap];
        types[usize::arbitrary(g) % types.len()]
    }
}

impl Arbitrary for Path {
    fn arbitrary(g: &mut Gen) -> Self {
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];
        let num_points = MIN_PATH_POINTS + (usize::arbitrary(g) % (MAX_EXTRA_POINTS + 1));
        let points: Vec<Point> = (0..num_points)
            .map(|_| {
                let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                Point::integer(x, y, units)
            })
            .collect();
        let layer = Layer::new(u16::arbitrary(g));
        let data_type = DataType::new(u16::arbitrary(g));
        let path_type = if bool::arbitrary(g) {
            Some(PathType::arbitrary(g))
        } else {
            None
        };
        let width = if bool::arbitrary(g) {
            let w = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
            Some(Unit::integer(w, units))
        } else {
            None
        };
        let begin_extension = if bool::arbitrary(g) {
            let e = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
            Some(Unit::integer(e, units))
        } else {
            None
        };
        let end_extension = if bool::arbitrary(g) {
            let e = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
            Some(Unit::integer(e, units))
        } else {
            None
        };
        Self::new(
            points,
            layer,
            data_type,
            path_type,
            width,
            begin_extension,
            end_extension,
        )
    }
}

impl Arbitrary for Text {
    fn arbitrary(g: &mut Gen) -> Self {
        let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
        let units = units_options[usize::arbitrary(g) % units_options.len()];
        let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
        let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
        let origin = Point::integer(x, y, units);
        let len = 1 + (usize::arbitrary(g) % 20);
        let value: String = (0..len)
            .map(|_| (b'a' + (u8::arbitrary(g) % 26)) as char)
            .collect();
        let layer = Layer::new(u16::arbitrary(g) % 256);
        let datatype = DataType::new(u16::arbitrary(g) % 256);
        let vp_options = [
            HorizontalPresentation::Left,
            HorizontalPresentation::Centre,
            HorizontalPresentation::Right,
        ];
        let hp_options = [
            VerticalPresentation::Top,
            VerticalPresentation::Middle,
            VerticalPresentation::Bottom,
        ];
        Self::new(
            &value,
            origin,
            layer,
            datatype,
            1.0,
            0.0,
            false,
            hp_options[usize::arbitrary(g) % hp_options.len()],
            vp_options[usize::arbitrary(g) % vp_options.len()],
        )
    }
}

impl Arbitrary for Grid {
    fn arbitrary(g: &mut Gen) -> Self {
        let cols = 1 + (u32::arbitrary(g) % 10);
        let rows = 1 + (u32::arbitrary(g) % 10);
        let x = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let y = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let sx = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        let sy = (i32::arbitrary(g) % MAX_COORD).clamp(-MAX_COORD, MAX_COORD);
        Self::new(
            Point::integer(x, y, 1e-9),
            cols,
            rows,
            Some(Point::integer(sx, 0, 1e-9)),
            Some(Point::integer(0, sy, 1e-9)),
            1.0,
            0.0,
            false,
        )
    }
}

const GDS_NAME_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_$?";

pub(super) fn arb_layer(g: &mut Gen) -> Layer {
    Layer::new(u16::arbitrary(g) % 256)
}

pub(super) fn arb_data_type(g: &mut Gen) -> DataType {
    DataType::new(u16::arbitrary(g) % 256)
}

pub(super) fn arb_structure_name(g: &mut Gen) -> String {
    let len = 1 + (usize::arbitrary(g) % 32);
    (0..len)
        .map(|_| GDS_NAME_CHARS[usize::arbitrary(g) % GDS_NAME_CHARS.len()] as char)
        .collect()
}

pub(super) fn arb_text_string(g: &mut Gen) -> String {
    let len = 1 + (usize::arbitrary(g) % 512);
    (0..len)
        .map(|_| (b'a' + (u8::arbitrary(g) % 26)) as char)
        .collect()
}

pub(super) fn arb_integer_point(g: &mut Gen) -> Point {
    let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
    let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
    Point::integer(x, y, 1e-9)
}

pub(super) fn arb_gds_polygon(g: &mut Gen) -> Polygon {
    let num_vertices = 3 + (usize::arbitrary(g) % 48);
    let points: Vec<Point> = (0..num_vertices).map(|_| arb_integer_point(g)).collect();
    Polygon::new(points, arb_layer(g), arb_data_type(g))
}

pub(super) fn arb_gds_path(g: &mut Gen) -> Path {
    let num_points = 2 + (usize::arbitrary(g) % 19);
    let points: Vec<Point> = (0..num_points).map(|_| arb_integer_point(g)).collect();
    let path_type = if bool::arbitrary(g) {
        Some(PathType::arbitrary(g))
    } else {
        None
    };
    let width = if bool::arbitrary(g) {
        let w = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
        Some(Unit::integer(w, 1e-9))
    } else {
        None
    };
    let begin_extension = if bool::arbitrary(g) {
        let e = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
        Some(Unit::integer(e, 1e-9))
    } else {
        None
    };
    let end_extension = if bool::arbitrary(g) {
        let e = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
        Some(Unit::integer(e, 1e-9))
    } else {
        None
    };
    Path::new(
        points,
        arb_layer(g),
        arb_data_type(g),
        path_type,
        width,
        begin_extension,
        end_extension,
    )
}

/// GDS `TextType` is always written as 0, so we use 0 here for roundtrip compatibility.
pub(super) fn arb_gds_text(g: &mut Gen) -> Text {
    let origin = arb_integer_point(g);
    let value = arb_text_string(g);
    let vp_options = [
        HorizontalPresentation::Left,
        HorizontalPresentation::Centre,
        HorizontalPresentation::Right,
    ];
    let hp_options = [
        VerticalPresentation::Top,
        VerticalPresentation::Middle,
        VerticalPresentation::Bottom,
    ];
    Text::new(
        &value,
        origin,
        arb_layer(g),
        DataType::new(0),
        1.0,
        0.0,
        false,
        hp_options[usize::arbitrary(g) % hp_options.len()],
        vp_options[usize::arbitrary(g) % vp_options.len()],
    )
}
