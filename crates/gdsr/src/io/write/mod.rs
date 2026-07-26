mod conversion;
mod gds_format;
pub mod svg;
pub mod validation;

use std::fmt;
use std::io::Write;

use rayon::prelude::*;

use crate::config::gds_file_types::{
    GDSDataType, GDSRecord, STRANS_X_REFLECTION, record_head, record_header,
};
use crate::elements::text::get_presentation_value;
use crate::error::GdsError;
use crate::{
    Cell, Element, GdsBox, GdsTimestampPolicy, GdsTimestamps, Instance, Library, Movable, Node,
    Path, Point, Polygon, Property, Reference, Text, Transformable,
};
use conversion::{
    CoordinateConversion, CoordinateConverter, NearestCoordinateConverter, validate_database_units,
};
pub use conversion::{GdsConversionReport, GdsQuantization, GdsRoundingPolicy, GdsWriteOptions};
use gds_format::{eight_byte_real, write_u16_array_as_big_endian};
use validation::{
    validate_col_row, validate_data_type, validate_layer, validate_node_points,
    validate_path_points, validate_point_limit, validate_polygon_points, validate_string_length,
    validate_structure_name,
};

/// Trait for customizing GDS file serialization.
///
/// Implement this trait to customize how specific elements are serialized
/// (e.g., filtering layers, transforming during write, or using a different format).
/// All methods have default implementations that delegate to the corresponding
/// free functions in this module, so custom writers only need to override the
/// methods they want to change.
pub trait GdsWriter: Sync {
    /// Serializes an entire library to GDS bytes.
    fn write_library(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
    ) -> Result<Vec<u8>, GdsError> {
        write_library(self, library, user_units, db_units)
    }

    /// Serializes a library using an explicit timestamp policy.
    fn write_library_with_timestamp_policy(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
        timestamp_policy: GdsTimestampPolicy,
    ) -> Result<Vec<u8>, GdsError> {
        write_library_with_timestamp_policy(self, library, user_units, db_units, timestamp_policy)
    }

    /// Serializes a single cell to GDS bytes.
    fn write_cell(&self, cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_cell(self, cell, db_units)
    }

    /// Serializes a single cell using an explicit timestamp policy.
    fn write_cell_with_timestamp_policy(
        &self,
        cell: &Cell,
        db_units: f64,
        timestamp_policy: GdsTimestampPolicy,
    ) -> Result<Vec<u8>, GdsError> {
        write_cell_with_timestamp_policy(self, cell, db_units, timestamp_policy)
    }

    /// Serializes a single cell with an already resolved `BGNSTR` timestamp pair.
    fn write_cell_with_timestamps(
        &self,
        cell: &Cell,
        db_units: f64,
        timestamps: GdsTimestamps,
    ) -> Result<Vec<u8>, GdsError> {
        write_cell_with_timestamps(self, cell, db_units, timestamps)
    }

    /// Serializes a single element to GDS bytes.
    fn write_element(&self, element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError> {
        match element {
            Element::Polygon(polygon) => self.write_polygon(polygon, db_units),
            Element::Path(path) => self.write_path(path, db_units),
            Element::Text(text) => self.write_text(text, db_units),
            Element::Reference(reference) => self.write_reference(reference, db_units),
            Element::Box(gds_box) => self.write_box(gds_box, db_units),
            Element::Node(node) => self.write_node(node, db_units),
        }
    }

    /// Serializes a polygon to GDS bytes.
    fn write_polygon(&self, polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_polygon(polygon, db_units)
    }

    /// Serializes a path to GDS bytes.
    fn write_path(&self, path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_path(path, db_units)
    }

    /// Serializes a text element to GDS bytes.
    fn write_text(&self, text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_text(text, db_units)
    }

    /// Serializes a reference to GDS bytes.
    fn write_reference(&self, reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_reference(reference, db_units)
    }

    /// Serializes a box to GDS bytes.
    fn write_box(&self, gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_box(gds_box, db_units)
    }

    /// Serializes a node to GDS bytes.
    fn write_node(&self, node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_node(node, db_units)
    }
}

/// Default GDS writer using standard GDSII serialization.
pub struct GdsFileWriter;

impl GdsWriter for GdsFileWriter {
    fn write_cell_with_timestamps(
        &self,
        cell: &Cell,
        db_units: f64,
        timestamps: GdsTimestamps,
    ) -> Result<Vec<u8>, GdsError> {
        write_default_cell_with_timestamps(cell, db_units, timestamps)
    }
}

/// Writes a GDS library incrementally to an output stream.
///
/// The library header is written when the stream is created. Each call to
/// [`Self::write_cell`] serializes and flushes one cell, so the complete
/// library never needs to be held in memory. Call [`Self::finish`] to write the
/// library footer and recover the output stream.
///
/// # Examples
///
/// ```
/// use gdsr::{Cell, GdsError, GdsStreamWriter};
///
/// # fn main() -> Result<(), GdsError> {
/// let mut writer = GdsStreamWriter::new(Vec::new(), "example", 1e-6, 1e-9)?;
/// writer.write_cell(&Cell::new("top"))?;
/// let bytes = writer.finish()?;
///
/// assert!(!bytes.is_empty());
/// # Ok(())
/// # }
/// ```
pub struct GdsStreamWriter<W> {
    output: W,
    database_units: f64,
    options: GdsWriteOptions,
    current_timestamps: GdsTimestamps,
    conversion_report: GdsConversionReport,
}

impl<W: Write> GdsStreamWriter<W> {
    /// Writes a library header to `output` and starts a streaming GDS library.
    pub fn new(
        output: W,
        library_name: &str,
        user_units: f64,
        database_units: f64,
    ) -> Result<Self, GdsError> {
        Self::new_with_options(
            output,
            library_name,
            user_units,
            database_units,
            GdsWriteOptions::default().without_report(),
        )
    }

    /// Starts a streaming write using explicit coordinate and timestamp options.
    pub fn new_with_options(
        output: W,
        library_name: &str,
        user_units: f64,
        database_units: f64,
        options: GdsWriteOptions,
    ) -> Result<Self, GdsError> {
        Self::start(
            output,
            library_name,
            None,
            user_units,
            database_units,
            options,
        )
    }

    /// Starts a streaming write from a library using an explicit timestamp policy.
    ///
    /// Taking the library provides the `BGNLIB` metadata required by
    /// [`GdsTimestampPolicy::Preserve`]. Cells passed to [`Self::write_cell`]
    /// must also contain timestamps when that policy is selected.
    pub fn from_library_with_timestamp_policy(
        output: W,
        library: &Library,
        user_units: f64,
        database_units: f64,
        timestamp_policy: GdsTimestampPolicy,
    ) -> Result<Self, GdsError> {
        let options = GdsWriteOptions::default()
            .with_timestamp_policy(timestamp_policy)
            .without_report();
        Self::from_library_with_options(output, library, user_units, database_units, options)
    }

    /// Starts a streaming write from a library using explicit serialization options.
    pub fn from_library_with_options(
        output: W,
        library: &Library,
        user_units: f64,
        database_units: f64,
        options: GdsWriteOptions,
    ) -> Result<Self, GdsError> {
        Self::start(
            output,
            library.name(),
            library.timestamps(),
            user_units,
            database_units,
            options,
        )
    }

    fn start(
        mut output: W,
        library_name: &str,
        library_timestamps: Option<GdsTimestamps>,
        user_units: f64,
        database_units: f64,
        options: GdsWriteOptions,
    ) -> Result<Self, GdsError> {
        let current_timestamps = GdsTimestamps::current();
        let timestamps = resolve_timestamps(
            options.timestamp_policy(),
            library_timestamps,
            current_timestamps,
            "library",
            library_name,
        )?;
        write_gds_head_to_file(
            library_name,
            user_units,
            database_units,
            timestamps,
            &mut output,
        )?;
        output.flush()?;
        Ok(Self {
            output,
            database_units,
            options,
            current_timestamps,
            conversion_report: GdsConversionReport::default(),
        })
    }

    /// Serializes and flushes one cell to the library.
    ///
    /// If this fails, cells written by earlier calls remain in the output. The
    /// caller should discard the partial stream when atomic output is required.
    pub fn write_cell(&mut self, cell: &Cell) -> Result<(), GdsError> {
        let timestamps = resolve_timestamps(
            self.options.timestamp_policy(),
            cell.timestamps(),
            self.current_timestamps,
            "cell",
            cell.name(),
        )?;
        if !self.options.collects_report()
            && self.options.rounding_policy() == GdsRoundingPolicy::Nearest
        {
            let bytes =
                GdsFileWriter.write_cell_with_timestamps(cell, self.database_units, timestamps)?;
            self.output.write_all(&bytes)?;
            self.output.flush()?;
            return Ok(());
        }
        let (bytes, report) =
            write_cell_with_options(cell, self.database_units, self.options, timestamps)?;
        self.output.write_all(&bytes)?;
        self.output.flush()?;
        self.conversion_report.extend(report);
        Ok(())
    }

    /// Returns the conversion report accumulated by cells written so far.
    ///
    /// Affected locations follow cell write order. [`Self::finish_with_report`]
    /// sorts its returned report by cell name and numeric element index.
    pub const fn conversion_report(&self) -> &GdsConversionReport {
        &self.conversion_report
    }

    /// Writes and flushes the library footer, returning the output stream.
    pub fn finish(mut self) -> Result<W, GdsError> {
        write_gds_tail_to_file(&mut self.output)?;
        self.output.flush()?;
        Ok(self.output)
    }

    /// Finishes the library and returns the output plus its conversion report.
    pub fn finish_with_report(mut self) -> Result<(W, GdsConversionReport), GdsError> {
        write_gds_tail_to_file(&mut self.output)?;
        self.output.flush()?;
        self.conversion_report.sort();
        Ok((self.output, self.conversion_report))
    }
}

pub fn write_u16_array(buffer: &mut impl Write, array: &[u16]) -> Result<(), GdsError> {
    Ok(write_u16_array_as_big_endian(buffer, array)?)
}

fn cell_head_record(timestamps: GdsTimestamps) -> [u16; 14] {
    let [size, head] = record_header(GDSRecord::BgnStr, GDSDataType::TwoByteSignedInteger, 12);
    let ts = timestamps.to_record();
    [
        size, head, ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7], ts[8], ts[9], ts[10],
        ts[11],
    ]
}

/// Writes a single i32 value as a `FourByteSignedInteger` GDS record.
fn write_i32_record(
    buffer: &mut impl Write,
    record: GDSRecord,
    value: i32,
) -> Result<(), GdsError> {
    write_u16_array(
        buffer,
        &record_header(record, GDSDataType::FourByteSignedInteger, 1),
    )?;
    buffer.write_all(&value.to_be_bytes())?;
    Ok(())
}

fn write_gds_head_to_file(
    library_name: &str,
    user_units: f64,
    db_units: f64,
    timestamps: GdsTimestamps,
    buffer: &mut impl Write,
) -> Result<(), GdsError> {
    fn encode_unit(value: f64, name: &str) -> Result<[u8; 8], GdsError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(GdsError::ValidationError {
                message: format!("{name} must be finite and positive, got {value}"),
            });
        }
        eight_byte_real(value).map_err(|error| GdsError::ValidationError {
            message: format!("{name} {value} cannot be encoded as GDS REAL8: {error}"),
        })
    }

    let encoded_user_units = encode_unit(user_units, "User units")?;
    let encoded_database_units = encode_unit(db_units, "Database units")?;
    let [s1, h1] = record_header(GDSRecord::Header, GDSDataType::TwoByteSignedInteger, 1);
    let [s2, h2] = record_header(GDSRecord::BgnLib, GDSDataType::TwoByteSignedInteger, 12);
    let ts = timestamps.to_record();
    let head_start = [
        s1, h1, 0x0258, s2, h2, ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7], ts[8],
        ts[9], ts[10], ts[11],
    ];

    let mut header = Vec::with_capacity(64 + library_name.len());
    write_u16_array(&mut header, &head_start)?;
    write_string_with_record_to_file(&mut header, GDSRecord::LibName, library_name)?;
    write_u16_array(
        &mut header,
        &record_header(GDSRecord::Units, GDSDataType::EightByteReal, 2),
    )?;
    header.extend_from_slice(&encoded_user_units);
    header.extend_from_slice(&encoded_database_units);
    buffer.write_all(&header)?;
    Ok(())
}

fn write_gds_tail_to_file(buffer: &mut impl Write) -> Result<(), GdsError> {
    write_u16_array(
        buffer,
        &record_header(GDSRecord::EndLib, GDSDataType::NoData, 1),
    )
}

fn write_points_to_file(
    buffer: &mut impl Write,
    points: &[Point],
    converter: &mut impl CoordinateConversion,
    field: &str,
    require_closed: bool,
) -> Result<(), GdsError> {
    struct PointField<'a> {
        field: &'a str,
        index: usize,
        axis: char,
    }

    impl fmt::Display for PointField<'_> {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}[{}].{}", self.field, self.index, self.axis)
        }
    }

    validate_point_limit(points, "XY record")?;

    let record_size = GDSDataType::FourByteSignedInteger.record_size(points.len() as u16 * 2);
    let xy_header_buffer = [
        record_size,
        record_head(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
    ];

    write_u16_array(buffer, &xy_header_buffer)?;

    if require_closed {
        let mut first = None;
        let mut last = None;
        for (index, point) in points.iter().enumerate() {
            let scaled_x = converter.convert_unit(
                point.x(),
                PointField {
                    field,
                    index,
                    axis: 'x',
                },
            )?;
            let scaled_y = converter.convert_unit(
                point.y(),
                PointField {
                    field,
                    index,
                    axis: 'y',
                },
            )?;
            first.get_or_insert([scaled_x, scaled_y]);
            last = Some([scaled_x, scaled_y]);
            buffer.write_all(&scaled_x.to_be_bytes())?;
            buffer.write_all(&scaled_y.to_be_bytes())?;
        }
        if let (Some(first), Some(last)) = (first, last) {
            converter.validate_closed(first, last, field)?;
        }
    } else {
        for (index, point) in points.iter().enumerate() {
            let scaled_x = converter.convert_unit(
                point.x(),
                PointField {
                    field,
                    index,
                    axis: 'x',
                },
            )?;
            let scaled_y = converter.convert_unit(
                point.y(),
                PointField {
                    field,
                    index,
                    axis: 'y',
                },
            )?;
            buffer.write_all(&scaled_x.to_be_bytes())?;
            buffer.write_all(&scaled_y.to_be_bytes())?;
        }
    }

    Ok(())
}

fn write_database_points_to_file(
    buffer: &mut impl Write,
    points: &[[i32; 2]],
) -> Result<(), GdsError> {
    let record_size = GDSDataType::FourByteSignedInteger.record_size(points.len() as u16 * 2);
    write_u16_array(
        buffer,
        &[
            record_size,
            record_head(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
        ],
    )?;
    for [x, y] in points {
        buffer.write_all(&x.to_be_bytes())?;
        buffer.write_all(&y.to_be_bytes())?;
    }
    Ok(())
}

fn write_element_tail_to_file(
    buffer: &mut impl Write,
    properties: &[Property],
) -> Result<(), GdsError> {
    for property in properties {
        let [size, head] = record_header(GDSRecord::PropAttr, GDSDataType::TwoByteSignedInteger, 1);
        write_u16_array(buffer, &[size, head, property.attribute()])?;
        write_string_with_record_to_file(buffer, GDSRecord::PropValue, property.value())?;
    }

    let tail = [
        GDSDataType::NoData.record_size(1),
        record_head(GDSRecord::EndEl, GDSDataType::NoData),
    ];
    write_u16_array(buffer, &tail)
}

fn write_string_with_record_to_file(
    buffer: &mut impl Write,
    record: GDSRecord,
    string: &str,
) -> Result<(), GdsError> {
    let byte_len = string.len();
    let Some(padded_len) = byte_len.checked_add(byte_len % 2) else {
        return Err(GdsError::ValidationError {
            message: "String record length overflow".to_string(),
        });
    };
    let Some(record_size) = 4usize
        .checked_add(padded_len)
        .and_then(|size| u16::try_from(size).ok())
    else {
        return Err(GdsError::ValidationError {
            message: format!("String record of {byte_len} bytes exceeds the GDS record limit"),
        });
    };

    let string_start = [record_size, record_head(record, GDSDataType::AsciiString)];

    write_u16_array(buffer, &string_start)?;

    buffer.write_all(string.as_bytes())?;
    if !byte_len.is_multiple_of(2) {
        buffer.write_all(&[0])?;
    }

    Ok(())
}

fn write_transformation_to_file(
    buffer: &mut impl Write,
    angle: f64,
    magnification: f64,
    x_reflection: bool,
    converter: &impl CoordinateConversion,
) -> Result<(), GdsError> {
    let encoded_magnification = if magnification == 1.0 {
        None
    } else {
        if magnification <= 0.0 {
            converter.validate_positive(magnification, "magnification")?;
        }
        Some(converter.encode_gds_real8(magnification, "magnification")?)
    };
    let encoded_angle = if angle == 0.0 {
        None
    } else {
        Some(converter.encode_gds_real8(angle, "angle")?)
    };
    let transform_applied = angle != 0.0 || magnification != 1.0 || x_reflection;
    if transform_applied {
        let buffer_flags = [
            GDSDataType::BitArray.record_size(1),
            record_head(GDSRecord::STrans, GDSDataType::BitArray),
            if x_reflection { STRANS_X_REFLECTION } else { 0 },
        ];

        write_u16_array(buffer, &buffer_flags)?;

        if let Some(encoded_magnification) = encoded_magnification {
            let buffer_mag = [
                GDSDataType::EightByteReal.record_size(1),
                record_head(GDSRecord::Mag, GDSDataType::EightByteReal),
            ];
            write_u16_array(buffer, &buffer_mag)?;
            buffer.write_all(&encoded_magnification)?;
        }

        if let Some(encoded_angle) = encoded_angle {
            let buffer_rot = [
                GDSDataType::EightByteReal.record_size(1),
                record_head(GDSRecord::Angle, GDSDataType::EightByteReal),
            ];
            write_u16_array(buffer, &buffer_rot)?;
            buffer.write_all(&encoded_angle)?;
        }
    }

    Ok(())
}

/// Builds the common element header: record type, layer, and sub-type (DataType/BoxType/NodeType).
fn element_head(
    record: GDSRecord,
    layer: u16,
    type_record: GDSRecord,
    type_value: u16,
) -> [u16; 8] {
    let [s1, h1] = record_header(record, GDSDataType::NoData, 1);
    let [s2, h2] = record_header(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger, 1);
    let [s3, h3] = record_header(type_record, GDSDataType::TwoByteSignedInteger, 1);
    [s1, h1, s2, h2, layer, s3, h3, type_value]
}

/// Serializes a library with the built-in writer and returns its conversion report.
///
/// Custom [`GdsWriter`] overrides are intentionally not used by this API.
pub fn write_library_with_options(
    library: &Library,
    user_units: f64,
    db_units: f64,
    options: GdsWriteOptions,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    let current_timestamps = GdsTimestamps::current();
    let library_timestamps = resolve_timestamps(
        options.timestamp_policy(),
        library.timestamps(),
        current_timestamps,
        "library",
        library.name(),
    )?;
    let mut buffer = Vec::new();
    write_gds_head_to_file(
        library.name(),
        user_units,
        db_units,
        library_timestamps,
        &mut buffer,
    )?;

    let mut cells: Vec<&Cell> = library.cells().values().collect();
    cells.sort_unstable_by(|left, right| left.name().cmp(right.name()));
    let cells = cells
        .par_iter()
        .map(|cell| {
            let timestamps = resolve_timestamps(
                options.timestamp_policy(),
                cell.timestamps(),
                current_timestamps,
                "cell",
                cell.name(),
            )?;
            write_cell_with_options(cell, db_units, options, timestamps)
        })
        .collect::<Vec<_>>();

    let mut report = GdsConversionReport::default();
    for cell in cells {
        let (bytes, cell_report) = cell?;
        buffer.extend_from_slice(&bytes);
        if options.collects_report() {
            report.extend(cell_report);
        }
    }

    write_gds_tail_to_file(&mut buffer)?;
    report.sort();
    Ok((buffer, report))
}

/// Serializes an entire library to GDS bytes.
pub fn write_library(
    writer: &(impl GdsWriter + ?Sized),
    library: &Library,
    user_units: f64,
    db_units: f64,
) -> Result<Vec<u8>, GdsError> {
    write_library_with_timestamp_policy(
        writer,
        library,
        user_units,
        db_units,
        GdsTimestampPolicy::Current,
    )
}

/// Serializes an entire library using an explicit timestamp policy.
pub fn write_library_with_timestamp_policy(
    writer: &(impl GdsWriter + ?Sized),
    library: &Library,
    user_units: f64,
    db_units: f64,
    timestamp_policy: GdsTimestampPolicy,
) -> Result<Vec<u8>, GdsError> {
    let current_timestamps = GdsTimestamps::current();
    let library_timestamps = resolve_timestamps(
        timestamp_policy,
        library.timestamps(),
        current_timestamps,
        "library",
        library.name(),
    )?;
    let mut buffer = Vec::new();
    write_gds_head_to_file(
        library.name(),
        user_units,
        db_units,
        library_timestamps,
        &mut buffer,
    )?;

    let mut cells: Vec<&Cell> = library.cells().values().collect();
    cells.sort_unstable_by(|left, right| left.name().cmp(right.name()));

    let cells = cells
        .par_iter()
        .map(|cell| {
            let timestamps = resolve_timestamps(
                timestamp_policy,
                cell.timestamps(),
                current_timestamps,
                "cell",
                cell.name(),
            )?;
            writer.write_cell_with_timestamps(cell, db_units, timestamps)
        })
        .collect::<Vec<_>>();
    for cell in cells {
        let buf = cell?;
        buffer.extend_from_slice(&buf);
    }

    write_gds_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a single cell to GDS bytes.
pub fn write_cell(
    writer: &(impl GdsWriter + ?Sized),
    cell: &Cell,
    db_units: f64,
) -> Result<Vec<u8>, GdsError> {
    write_cell_with_timestamp_policy(writer, cell, db_units, GdsTimestampPolicy::Current)
}

/// Serializes a single cell using an explicit timestamp policy.
pub fn write_cell_with_timestamp_policy(
    writer: &(impl GdsWriter + ?Sized),
    cell: &Cell,
    db_units: f64,
    timestamp_policy: GdsTimestampPolicy,
) -> Result<Vec<u8>, GdsError> {
    let current_timestamps = GdsTimestamps::current();
    let timestamps = resolve_timestamps(
        timestamp_policy,
        cell.timestamps(),
        current_timestamps,
        "cell",
        cell.name(),
    )?;
    write_cell_with_timestamps(writer, cell, db_units, timestamps)
}

/// Serializes a single cell with an already resolved timestamp pair.
pub fn write_cell_with_timestamps(
    writer: &(impl GdsWriter + ?Sized),
    cell: &Cell,
    db_units: f64,
    timestamps: GdsTimestamps,
) -> Result<Vec<u8>, GdsError> {
    validate_database_units(db_units)?;
    validate_structure_name(cell.name())?;

    let mut buffer = Vec::new();
    write_u16_array(&mut buffer, &cell_head_record(timestamps))?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, cell.name())?;

    let elements = cell
        .elements()
        .par_iter()
        .map(|element| writer.write_element(element, db_units))
        .collect::<Vec<_>>();
    for element in elements {
        let buf = element?;
        buffer.extend_from_slice(&buf);
    }

    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::EndStr, GDSDataType::NoData, 1),
    )?;

    Ok(buffer)
}

fn write_default_cell_with_timestamps(
    cell: &Cell,
    db_units: f64,
    timestamps: GdsTimestamps,
) -> Result<Vec<u8>, GdsError> {
    validate_database_units(db_units)?;
    validate_structure_name(cell.name())?;

    let mut buffer = Vec::new();
    write_u16_array(&mut buffer, &cell_head_record(timestamps))?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, cell.name())?;

    let elements = cell
        .elements()
        .par_iter()
        .enumerate()
        .map(|(element_index, element)| {
            write_default_element(element, db_units, cell.name(), element_index)
        })
        .collect::<Vec<_>>();
    for element in elements {
        let bytes = element?;
        buffer.extend_from_slice(&bytes);
    }

    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::EndStr, GDSDataType::NoData, 1),
    )?;

    Ok(buffer)
}

fn write_default_element(
    element: &Element,
    db_units: f64,
    cell_name: &str,
    element_index: usize,
) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new_validated(
        db_units,
        cell_name,
        element_index,
        element_type(element),
    );
    match element {
        Element::Polygon(polygon) => write_polygon_with_converter(polygon, &mut converter),
        Element::Path(path) => write_path_with_converter(path, &mut converter),
        Element::Text(text) => write_text_with_converter(text, &mut converter),
        Element::Reference(reference) => {
            if let Instance::Cell(referenced_cell_name) = reference.instance() {
                write_reference_cell(reference, referenced_cell_name, &mut converter)
            } else {
                write_reference_with_options(
                    reference,
                    db_units,
                    GdsWriteOptions::default().without_report(),
                    cell_name,
                    element_index,
                    None,
                )
                .map(|(bytes, _)| bytes)
            }
        }
        Element::Box(gds_box) => write_box_with_converter(gds_box, &mut converter),
        Element::Node(node) => write_node_with_converter(node, &mut converter),
    }
}

fn write_cell_with_options(
    cell: &Cell,
    db_units: f64,
    options: GdsWriteOptions,
    timestamps: GdsTimestamps,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    validate_database_units(db_units)?;
    validate_structure_name(cell.name())?;

    let mut buffer = Vec::new();
    write_u16_array(&mut buffer, &cell_head_record(timestamps))?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, cell.name())?;

    let elements = cell
        .elements()
        .par_iter()
        .enumerate()
        .map(|(element_index, element)| {
            write_element_with_options(element, db_units, options, cell.name(), element_index)
        })
        .collect::<Vec<_>>();

    let mut report = GdsConversionReport::default();
    for element in elements {
        let (bytes, element_report) = element?;
        buffer.extend_from_slice(&bytes);
        if options.collects_report() {
            report.extend(element_report);
        }
    }

    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::EndStr, GDSDataType::NoData, 1),
    )?;

    report.sort();
    Ok((buffer, report))
}

fn resolve_timestamps(
    policy: GdsTimestampPolicy,
    preserved: Option<GdsTimestamps>,
    current: GdsTimestamps,
    target_kind: &str,
    target_name: &str,
) -> Result<GdsTimestamps, GdsError> {
    match policy {
        GdsTimestampPolicy::Current => Ok(current),
        GdsTimestampPolicy::Preserve => preserved.ok_or_else(|| GdsError::ValidationError {
            message: format!(
                "Cannot preserve timestamps for {target_kind} '{target_name}': no timestamps are stored"
            ),
        }),
        GdsTimestampPolicy::Zero => Ok(GdsTimestamps::ZERO),
    }
}

fn element_type(element: &Element) -> &'static str {
    match element {
        Element::Polygon(_) => "polygon",
        Element::Path(_) => "path",
        Element::Text(_) => "text",
        Element::Reference(_) => "reference",
        Element::Box(_) => "box",
        Element::Node(_) => "node",
    }
}

fn write_element_with_options(
    element: &Element,
    db_units: f64,
    options: GdsWriteOptions,
    cell_name: &str,
    element_index: usize,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    write_element_with_field_prefix(element, db_units, options, cell_name, element_index, None)
}

fn write_element_with_field_prefix(
    element: &Element,
    db_units: f64,
    options: GdsWriteOptions,
    cell_name: &str,
    element_index: usize,
    field_prefix: Option<String>,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    if let Element::Reference(reference) = element {
        return write_reference_with_options(
            reference,
            db_units,
            options,
            cell_name,
            element_index,
            field_prefix,
        );
    }

    let reported_element_type = if field_prefix.is_some() {
        "reference"
    } else {
        element_type(element)
    };
    let converter = CoordinateConverter::new(
        db_units,
        options.rounding_policy(),
        cell_name,
        element_index,
        reported_element_type,
    )?
    .with_field_prefix(field_prefix);
    let mut converter = if options.collects_report() {
        converter
    } else {
        converter.without_report()
    };
    let bytes = match element {
        Element::Polygon(polygon) => write_polygon_with_converter(polygon, &mut converter),
        Element::Path(path) => write_path_with_converter(path, &mut converter),
        Element::Text(text) => write_text_with_converter(text, &mut converter),
        Element::Reference(_) => {
            return Err(GdsError::ValidationError {
                message: "Reference dispatch failed".to_string(),
            });
        }
        Element::Box(gds_box) => write_box_with_converter(gds_box, &mut converter),
        Element::Node(node) => write_node_with_converter(node, &mut converter),
    }?;
    Ok((bytes, converter.finish()))
}

/// Serializes a polygon to GDS bytes.
pub fn write_polygon(polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new(db_units, "polygon")?;
    write_polygon_with_converter(polygon, &mut converter)
}

fn write_polygon_with_converter(
    polygon: &Polygon,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    validate_polygon_points(polygon.points())?;
    validate_layer(polygon.layer())?;
    validate_data_type(polygon.data_type())?;

    let mut buffer = Vec::new();

    let head = element_head(
        GDSRecord::Boundary,
        polygon.layer().value(),
        GDSRecord::DataType,
        polygon.data_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
    write_points_to_file(&mut buffer, polygon.points(), converter, "point", true)?;
    write_element_tail_to_file(&mut buffer, polygon.properties())?;

    Ok(buffer)
}

/// Serializes a path to GDS bytes.
pub fn write_path(path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new(db_units, "path")?;
    write_path_with_converter(path, &mut converter)
}

fn write_path_with_converter(
    path: &Path,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    validate_path_points(path.points())?;
    validate_layer(path.layer())?;
    validate_data_type(path.data_type())?;

    let mut buffer = Vec::new();

    let head = element_head(
        GDSRecord::Path,
        path.layer().value(),
        GDSRecord::DataType,
        path.data_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;

    if let Some(path_type) = path.path_type() {
        let [size, head] = record_header(GDSRecord::PathType, GDSDataType::TwoByteSignedInteger, 1);
        write_u16_array(&mut buffer, &[size, head, path_type.value()])?;
    }

    if let Some(width) = path.width() {
        let width_value = converter.convert_unit(width, "width")?;
        write_i32_record(&mut buffer, GDSRecord::Width, width_value)?;
    }

    if let Some(begin_ext) = path.begin_extension() {
        let value = converter.convert_unit(begin_ext, "begin_extension")?;
        write_i32_record(&mut buffer, GDSRecord::BgnExtn, value)?;
    }

    if let Some(end_ext) = path.end_extension() {
        let value = converter.convert_unit(end_ext, "end_extension")?;
        write_i32_record(&mut buffer, GDSRecord::EndExtn, value)?;
    }

    write_points_to_file(&mut buffer, path.points(), converter, "point", false)?;
    write_element_tail_to_file(&mut buffer, path.properties())?;

    Ok(buffer)
}

/// Serializes a text element to GDS bytes.
pub fn write_text(text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new(db_units, "text")?;
    write_text_with_converter(text, &mut converter)
}

fn write_text_with_converter(
    text: &Text,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    validate_layer(text.layer())?;
    validate_string_length(text.text())?;
    let angle = text.angle().to_degrees().value();

    let mut buffer = Vec::new();

    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::Text, GDSDataType::NoData, 1),
    )?;
    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger, 1),
    )?;
    write_u16_array(&mut buffer, &[text.layer().value()])?;
    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger, 1),
    )?;
    write_u16_array(&mut buffer, &[text.data_type().value()])?;
    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::Presentation, GDSDataType::BitArray, 1),
    )?;
    write_u16_array(
        &mut buffer,
        &[get_presentation_value(
            *text.vertical_presentation(),
            *text.horizontal_presentation(),
        )],
    )?;
    write_transformation_to_file(
        &mut buffer,
        angle,
        text.magnification(),
        text.x_reflection(),
        converter,
    )?;
    write_points_to_file(&mut buffer, &[*text.origin()], converter, "origin", false)?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::String, text.text())?;
    write_element_tail_to_file(&mut buffer, text.properties())?;

    Ok(buffer)
}

/// Serializes a reference to GDS bytes.
///
/// Properties on inline references are appended to each expanded element.
pub fn write_reference(reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
    if let Instance::Cell(referenced_cell_name) = reference.instance() {
        let mut converter = NearestCoordinateConverter::new(db_units, "reference")?;
        return write_reference_cell(reference, referenced_cell_name, &mut converter);
    }
    write_reference_with_options(
        reference,
        db_units,
        GdsWriteOptions::default().without_report(),
        "<standalone>",
        0,
        None,
    )
    .map(|(bytes, _)| bytes)
}

fn write_reference_with_options(
    reference: &Reference,
    db_units: f64,
    options: GdsWriteOptions,
    cell_name: &str,
    element_index: usize,
    field_prefix: Option<String>,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    match reference.instance() {
        Instance::Cell(referenced_cell_name) => {
            let converter = CoordinateConverter::new(
                db_units,
                options.rounding_policy(),
                cell_name,
                element_index,
                "reference",
            )?
            .with_field_prefix(field_prefix);
            let mut converter = if options.collects_report() {
                converter
            } else {
                converter.without_report()
            };
            let bytes = write_reference_cell(reference, referenced_cell_name, &mut converter)?;
            Ok((bytes, converter.finish()))
        }
        Instance::Element(element) => write_reference_element(
            reference,
            element.as_ref().as_ref(),
            db_units,
            options,
            cell_name,
            element_index,
            field_prefix.as_deref(),
        ),
    }
}

fn write_reference_element(
    reference: &Reference,
    element: &Element,
    db_units: f64,
    options: GdsWriteOptions,
    cell_name: &str,
    element_index: usize,
    field_prefix: Option<&str>,
) -> Result<(Vec<u8>, GdsConversionReport), GdsError> {
    let source_grid = reference.grid();
    let properties = reference.properties();

    let grid_converter = CoordinateConverter::new(
        db_units,
        options.rounding_policy(),
        cell_name,
        element_index,
        "reference",
    )?
    .with_field_prefix(field_prefix.map(str::to_owned));
    grid_converter.validate_finite(source_grid.angle().value(), "angle")?;
    grid_converter.validate_positive(source_grid.magnification(), "magnification")?;
    grid_converter.physical_value(source_grid.origin().x(), "origin.x")?;
    grid_converter.physical_value(source_grid.origin().y(), "origin.y")?;
    if let Some(spacing) = source_grid.spacing_x() {
        grid_converter.physical_value(spacing.x(), "spacing_x.x")?;
        grid_converter.physical_value(spacing.y(), "spacing_x.y")?;
    }
    if let Some(spacing) = source_grid.spacing_y() {
        grid_converter.physical_value(spacing.x(), "spacing_y.x")?;
        grid_converter.physical_value(spacing.y(), "spacing_y.y")?;
    }

    let grid = source_grid.clone().to_float_unit();
    let spacing_x = grid.spacing_x().unwrap_or_default();
    let spacing_y = grid.spacing_y().unwrap_or_default();
    let mut buf = Vec::new();
    let mut report = grid_converter.finish();
    for column_index in 0..grid.columns() {
        for row_index in 0..grid.rows() {
            let offset = (spacing_x * column_index) + (spacing_y * row_index);
            let rotated_offset = offset.rotate_around_point(grid.angle(), &Point::default());
            let final_position = grid.origin().to_float_unit() + rotated_offset.to_float_unit();

            let mut new_element = element.clone().to_float_unit();
            if grid.x_reflection() {
                new_element = new_element.reflect(crate::Radians::new(0.0), Point::default());
            }
            new_element = new_element.rotate(grid.angle(), Point::default());
            new_element = new_element.scale(grid.magnification(), Point::default());
            new_element = new_element.move_by(final_position);
            if !properties.is_empty() {
                new_element.properties_mut().extend_from_slice(properties);
            }

            let expansion_prefix = if let Some(prefix) = &field_prefix {
                format!("{prefix}.inline[{column_index},{row_index}]")
            } else {
                format!("inline[{column_index},{row_index}]")
            };
            let (bytes, element_report) = write_element_with_field_prefix(
                &new_element,
                db_units,
                options,
                cell_name,
                element_index,
                Some(expansion_prefix),
            )?;
            buf.extend_from_slice(&bytes);
            report.extend(element_report);
        }
    }
    Ok((buf, report))
}

fn write_reference_cell(
    reference: &Reference,
    cell_name: &str,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    let grid = reference.grid();
    validate_structure_name(cell_name)?;
    validate_col_row(grid.columns(), grid.rows())?;
    if grid.columns() > 1 && grid.spacing_x().is_none() {
        return Err(GdsError::ValidationError {
            message: "Array references with multiple columns require column spacing".to_string(),
        });
    }
    if grid.rows() > 1 && grid.spacing_y().is_none() {
        return Err(GdsError::ValidationError {
            message: "Array references with multiple rows require row spacing".to_string(),
        });
    }

    let is_single_instance = grid.columns() == 1 && grid.rows() == 1;

    let record = if is_single_instance {
        GDSRecord::SRef
    } else {
        GDSRecord::ARef
    };

    let mut buffer = Vec::new();

    write_u16_array(&mut buffer, &record_header(record, GDSDataType::NoData, 1))?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::SName, cell_name)?;
    write_transformation_to_file(
        &mut buffer,
        grid.angle().to_degrees().value(),
        grid.magnification(),
        grid.x_reflection(),
        converter,
    )?;

    if is_single_instance {
        write_points_to_file(&mut buffer, &[grid.origin()], converter, "origin", false)?;
    } else {
        let grid = grid.clone().to_float_unit();
        let [size, head] = record_header(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger, 2);
        write_u16_array(
            &mut buffer,
            &[size, head, grid.columns() as u16, grid.rows() as u16],
        )?;

        let origin = grid.origin();
        let spacing_x = grid.spacing_x().unwrap_or_default();
        let spacing_y = grid.spacing_y().unwrap_or_default();
        let origin_x = converter.physical_value(origin.x(), "origin.x")?;
        let origin_y = converter.physical_value(origin.y(), "origin.y")?;
        let origin_database_x = converter.convert_physical(origin_x, "origin.x")?;
        let origin_database_y = converter.convert_physical(origin_y, "origin.y")?;

        let spacing_x_x = converter.physical_value(spacing_x.x(), "spacing_x.x")?;
        let spacing_x_y = converter.physical_value(spacing_x.y(), "spacing_x.y")?;
        let spacing_y_x = converter.physical_value(spacing_y.x(), "spacing_y.x")?;
        let spacing_y_y = converter.physical_value(spacing_y.y(), "spacing_y.y")?;
        let (sin, cos) = grid.angle().value().sin_cos();
        let rotated_spacing_x_x = spacing_x_x.mul_add(cos, -(spacing_x_y * sin));
        let rotated_spacing_x_y = spacing_x_x.mul_add(sin, spacing_x_y * cos);
        let rotated_spacing_y_x = spacing_y_x.mul_add(cos, -(spacing_y_y * sin));
        let rotated_spacing_y_y = spacing_y_x.mul_add(sin, spacing_y_y * cos);
        let spacing_database_x_x =
            converter.convert_physical(rotated_spacing_x_x, "spacing_x.rotated_x")?;
        let spacing_database_x_y =
            converter.convert_physical(rotated_spacing_x_y, "spacing_x.rotated_y")?;
        let spacing_database_y_x =
            converter.convert_physical(rotated_spacing_y_x, "spacing_y.rotated_x")?;
        let spacing_database_y_y =
            converter.convert_physical(rotated_spacing_y_y, "spacing_y.rotated_y")?;

        let point2_physical_x = rotated_spacing_x_x.mul_add(f64::from(grid.columns()), origin_x);
        let point2_physical_y = rotated_spacing_x_y.mul_add(f64::from(grid.columns()), origin_y);
        let point3_physical_x = rotated_spacing_y_x.mul_add(f64::from(grid.rows()), origin_x);
        let point3_physical_y = rotated_spacing_y_y.mul_add(f64::from(grid.rows()), origin_y);
        let point2_database_x = converter.checked_array_endpoint(
            origin_database_x,
            spacing_database_x_x,
            grid.columns(),
            point2_physical_x,
            "aref_endpoint[1].x",
        )?;
        let point2_database_y = converter.checked_array_endpoint(
            origin_database_y,
            spacing_database_x_y,
            grid.columns(),
            point2_physical_y,
            "aref_endpoint[1].y",
        )?;
        let point3_database_x = converter.checked_array_endpoint(
            origin_database_x,
            spacing_database_y_x,
            grid.rows(),
            point3_physical_x,
            "aref_endpoint[2].x",
        )?;
        let point3_database_y = converter.checked_array_endpoint(
            origin_database_y,
            spacing_database_y_y,
            grid.rows(),
            point3_physical_y,
            "aref_endpoint[2].y",
        )?;

        write_database_points_to_file(
            &mut buffer,
            &[
                [origin_database_x, origin_database_y],
                [point2_database_x, point2_database_y],
                [point3_database_x, point3_database_y],
            ],
        )?;
    }

    write_element_tail_to_file(&mut buffer, reference.properties())?;

    Ok(buffer)
}

/// Serializes a box to GDS bytes.
pub fn write_box(gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new(db_units, "box")?;
    write_box_with_converter(gds_box, &mut converter)
}

fn write_box_with_converter(
    gds_box: &GdsBox,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    validate_layer(gds_box.layer())?;
    validate_data_type(gds_box.box_type())?;

    let mut buffer = Vec::new();

    let head = element_head(
        GDSRecord::Box,
        gds_box.layer().value(),
        GDSRecord::BoxType,
        gds_box.box_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
    let points = gds_box.points();
    write_points_to_file(&mut buffer, &points, converter, "point", true)?;
    write_element_tail_to_file(&mut buffer, gds_box.properties())?;

    Ok(buffer)
}

/// Serializes a node to GDS bytes.
pub fn write_node(node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
    let mut converter = NearestCoordinateConverter::new(db_units, "node")?;
    write_node_with_converter(node, &mut converter)
}

fn write_node_with_converter(
    node: &Node,
    converter: &mut impl CoordinateConversion,
) -> Result<Vec<u8>, GdsError> {
    validate_node_points(node.points())?;
    validate_layer(node.layer())?;
    validate_data_type(node.node_type())?;

    let mut buffer = Vec::new();

    let head = element_head(
        GDSRecord::Node,
        node.layer().value(),
        GDSRecord::NodeType,
        node.node_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
    write_points_to_file(&mut buffer, node.points(), converter, "point", false)?;
    write_element_tail_to_file(&mut buffer, node.properties())?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{self, BufReader, Write};

    use chrono::{NaiveDate, NaiveDateTime};
    use rayon::ThreadPoolBuilder;

    use crate::config::gds_file_types::GDSRecordData;
    use crate::elements::node::MAX_NODE_POINTS;
    use crate::io::read::RecordReader;
    use crate::io::write::gds_format::{MAX_GDS_REAL8_MAGNITUDE, MIN_GDS_REAL8_MAGNITUDE};
    use crate::{
        DEFAULT_INTEGER_UNITS, DataType, GdsTimestampPolicy, GdsTimestamps, Grid, Layer, Library,
        Property, Unit,
    };

    use super::validation::MAX_POINTS;
    use super::*;

    #[derive(Default)]
    struct FlushCountingWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl Write for FlushCountingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    fn polygon_with_property(value: impl Into<String>) -> Polygon {
        let mut polygon = Polygon::new(
            [
                Point::integer(0, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(0, 10, DEFAULT_INTEGER_UNITS),
            ],
            Layer::new(1),
            DataType::new(2),
        );
        polygon.properties_mut().push(Property::new(65_535, value));
        polygon
    }

    fn datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("test date should be valid")
            .and_hms_opt(hour, minute, second)
            .expect("test time should be valid")
    }

    fn timestamps(first_year: i32, second_year: i32) -> GdsTimestamps {
        GdsTimestamps::try_new(
            datetime(first_year, 2, 3, 4, 5, 6),
            datetime(second_year, 7, 8, 9, 10, 11),
        )
        .expect("test timestamps should be valid")
    }

    fn read_library_bytes(bytes: &[u8]) -> Result<Library, GdsError> {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("timestamps.gds");
        fs::write(&path, bytes).expect("test GDS should be writable");
        Library::read_file(path, Some(DEFAULT_INTEGER_UNITS))
    }

    fn assert_validation_error(result: &Result<Vec<u8>, GdsError>) {
        assert!(matches!(result, Err(GdsError::ValidationError { .. })));
    }

    fn library_with_cells(cells: &[(String, u16)]) -> Library {
        let mut library = Library::new("deterministic");
        for (name, layer) in cells {
            let mut cell = Cell::new(name);
            for layer in [*layer, layer + 64] {
                cell.add(Polygon::new(
                    [
                        Point::integer(0, 0, DEFAULT_INTEGER_UNITS),
                        Point::integer(10, 0, DEFAULT_INTEGER_UNITS),
                        Point::integer(0, 10, DEFAULT_INTEGER_UNITS),
                    ],
                    Layer::new(layer),
                    DataType::new(0),
                ));
            }
            library.add_cell(cell);
        }
        library
    }

    fn points_with_count(point_count: usize) -> Vec<Point> {
        (0..point_count)
            .map(|index| {
                let coordinate = i32::try_from(index).expect("test point count should fit i32");
                Point::integer(coordinate, -coordinate, DEFAULT_INTEGER_UNITS)
            })
            .collect()
    }

    fn path_with_point_count(point_count: usize) -> Path {
        Path::new(
            points_with_count(point_count),
            Layer::new(1),
            DataType::new(2),
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn maximum_length_node_roundtrips() {
        let points = points_with_count(MAX_NODE_POINTS);
        let node = Node::new(points.clone(), Layer::new(1), DataType::new(2));
        let element_bytes = write_node(&node, DEFAULT_INTEGER_UNITS)
            .expect("maximum-length node should be writable");
        let xy = RecordReader::new(BufReader::new(element_bytes.as_slice()))
            .find_map(
                |record| match record.expect("node record should be readable") {
                    (GDSRecord::XY, GDSRecordData::I32(coordinates)) => Some(coordinates),
                    _ => None,
                },
            )
            .expect("node should contain an XY record");

        assert_eq!(xy.len(), MAX_NODE_POINTS * 2);

        let mut cell = Cell::new("top");
        cell.add(node);
        let mut library = Library::new("node_limit");
        library.add_cell(cell);
        let bytes = library
            .to_bytes_with_timestamp_policy(
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
            .expect("maximum-length node library should be writable");
        let roundtripped = Library::from_bytes(&bytes, Some(DEFAULT_INTEGER_UNITS))
            .expect("maximum-length node library should be readable");
        let roundtripped_node = roundtripped
            .get_cell("top")
            .and_then(|cell| cell.nodes().next())
            .expect("roundtripped node should exist");

        assert_eq!(roundtripped_node.points(), points);
    }

    #[test]
    fn oversized_node_is_rejected_by_writer() {
        let node = Node {
            points: points_with_count(MAX_NODE_POINTS + 1),
            ..Node::default()
        };

        let error = write_node(&node, DEFAULT_INTEGER_UNITS)
            .expect_err("oversized node should be rejected");

        assert_eq!(
            error.to_string(),
            format!(
                "Validation error: Node must have between 1 and {MAX_NODE_POINTS} points, got {}",
                MAX_NODE_POINTS + 1
            )
        );
    }

    #[test]
    fn maximum_length_path_preserves_entire_xy_record_and_roundtrips() {
        let path = path_with_point_count(MAX_POINTS);
        let element_bytes = write_path(&path, DEFAULT_INTEGER_UNITS)
            .expect("maximum-length path should be writable");
        let xy = RecordReader::new(BufReader::new(element_bytes.as_slice()))
            .find_map(
                |record| match record.expect("path record should be readable") {
                    (GDSRecord::XY, GDSRecordData::I32(coordinates)) => Some(coordinates),
                    _ => None,
                },
            )
            .expect("path should contain an XY record");
        let last_coordinate =
            i32::try_from(MAX_POINTS - 1).expect("maximum point index should fit i32");

        assert_eq!(xy.len(), MAX_POINTS * 2);
        assert_eq!(&xy[xy.len() - 2..], &[last_coordinate, -last_coordinate]);

        let mut cell = Cell::new("top");
        cell.add(path.clone());
        let mut library = Library::new("point_limit");
        library.add_cell(cell);
        let bytes = library
            .to_bytes_with_timestamp_policy(
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
            .expect("maximum-length path library should be writable");
        let roundtripped = Library::from_bytes(&bytes, Some(DEFAULT_INTEGER_UNITS))
            .expect("maximum-length path library should be readable");
        let roundtripped_path = roundtripped
            .get_cell("top")
            .and_then(|cell| cell.paths().next())
            .expect("roundtripped path should exist");

        assert_eq!(roundtripped_path.points(), path.points());
    }

    #[test]
    fn oversized_path_is_rejected_before_xy_serialization() {
        let path = path_with_point_count(MAX_POINTS + 1);
        let error = write_path(&path, DEFAULT_INTEGER_UNITS)
            .expect_err("oversized path should be rejected");

        assert_eq!(
            error.to_string(),
            format!(
                "Validation error: Path has {} points, which exceeds the maximum of {MAX_POINTS}",
                MAX_POINTS + 1
            )
        );

        let mut bytes = Vec::new();
        let mut converter = CoordinateConverter::new(
            DEFAULT_INTEGER_UNITS,
            GdsRoundingPolicy::Nearest,
            "<standalone>",
            0,
            "path",
        )
        .expect("converter should be created");
        let error = write_points_to_file(&mut bytes, path.points(), &mut converter, "point", false)
            .expect_err("oversized XY record should be rejected");

        assert!(bytes.is_empty());
        assert_eq!(
            error.to_string(),
            format!(
                "Validation error: XY record has {} points, which exceeds the maximum of {MAX_POINTS}",
                MAX_POINTS + 1
            )
        );
    }

    fn library_with_node(point: Point) -> Library {
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(Node::new([point], Layer::new(1), DataType::new(0)));
        library.add_cell(cell);
        library
    }

    fn zero_timestamp_options(rounding_policy: GdsRoundingPolicy) -> GdsWriteOptions {
        GdsWriteOptions::new(rounding_policy).with_timestamp_policy(GdsTimestampPolicy::Zero)
    }

    fn record_i32(bytes: &[u8], wanted: GDSRecord) -> Vec<i32> {
        RecordReader::new(BufReader::new(bytes))
            .find_map(
                |record| match record.expect("written record should be readable") {
                    (record, GDSRecordData::I32(values)) if record == wanted => Some(values),
                    _ => None,
                },
            )
            .expect("requested record should be present")
    }

    fn records_i32(bytes: &[u8], wanted: GDSRecord) -> Vec<Vec<i32>> {
        RecordReader::new(BufReader::new(bytes))
            .filter_map(
                |record| match record.expect("written record should be readable") {
                    (record, GDSRecordData::I32(values)) if record == wanted => Some(values),
                    _ => None,
                },
            )
            .collect()
    }

    fn record_f64(bytes: &[u8], wanted: GDSRecord) -> Vec<f64> {
        RecordReader::new(BufReader::new(bytes))
            .find_map(
                |record| match record.expect("written record should be readable") {
                    (record, GDSRecordData::F64(values)) if record == wanted => Some(values),
                    _ => None,
                },
            )
            .expect("requested record should be present")
    }

    #[test]
    fn float_coordinates_are_scaled_before_rounding() {
        let library = library_with_node(Point::float(0.4, -0.4, 1e-9));
        let (bytes, report) = library
            .to_bytes_with_options(
                1e-6,
                1e-10,
                zero_timestamp_options(GdsRoundingPolicy::Nearest),
            )
            .expect("coordinate should be writable");

        assert_eq!(record_i32(&bytes, GDSRecord::XY), [4, -4]);
        assert!(report.affected_locations().is_empty());
    }

    #[test]
    fn rounding_policies_are_deterministic_for_negative_half_grid_values() {
        let library = library_with_node(Point::float(0.5, -0.5, 1.0));
        for (policy, expected) in [
            (GdsRoundingPolicy::Nearest, [1, -1]),
            (GdsRoundingPolicy::Floor, [0, -1]),
            (GdsRoundingPolicy::Ceil, [1, 0]),
        ] {
            let (bytes, report) = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(policy))
                .expect("half-grid coordinate should be writable");

            assert_eq!(record_i32(&bytes, GDSRecord::XY), expected);
            assert_eq!(report.max_physical_quantization_error(), 0.5);
            assert_eq!(report.affected_locations().len(), 1);
            assert_eq!(report.affected_locations()[0].field(), "point[0].x");
            assert_eq!(report.affected_locations()[0].affected_field_count(), 2);
        }
    }

    #[test]
    fn tiny_off_grid_values_follow_every_rounding_policy() {
        let library = library_with_node(Point::float(1e-16, -1e-16, 1.0));
        for (policy, expected) in [
            (GdsRoundingPolicy::Nearest, [0, 0]),
            (GdsRoundingPolicy::Floor, [0, -1]),
            (GdsRoundingPolicy::Ceil, [1, 0]),
        ] {
            let (bytes, report) = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(policy))
                .expect("tiny off-grid values should use the selected policy");

            assert_eq!(record_i32(&bytes, GDSRecord::XY), expected);
            assert_eq!(report.affected_locations()[0].affected_field_count(), 2);
            assert!(report.max_physical_quantization_error() > 0.0);
        }

        let error = library
            .to_bytes_with_options(
                1.0,
                1.0,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect_err("tiny off-grid value should not be treated as zero");
        assert!(error.to_string().contains("field 'point[0].x'"));
        assert!(error.to_string().contains("off the 1 database-unit grid"));
    }

    #[test]
    fn meaningful_i32_boundary_offset_follows_every_rounding_policy() {
        let maximum = f64::from(i32::MAX);
        let ulp = f64::from_bits(maximum.to_bits() + 1) - maximum;
        let off_grid = maximum + 4.0 * ulp;
        let library = library_with_node(Point::float(off_grid, 0.0, 1.0));

        for policy in [GdsRoundingPolicy::Nearest, GdsRoundingPolicy::Floor] {
            let (bytes, report) = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(policy))
                .expect("in-range rounded boundary value should be writable");
            assert_eq!(record_i32(&bytes, GDSRecord::XY), [i32::MAX, 0]);
            assert_eq!(report.affected_locations().len(), 1);
        }

        let ceil_error = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Ceil))
            .expect_err("ceil should cross the i32 boundary");
        assert!(ceil_error.to_string().contains("outside the GDS i32 range"));

        let exact_error = library
            .to_bytes_with_options(
                1.0,
                1.0,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect_err("meaningful boundary offset should remain off-grid");
        assert!(
            exact_error
                .to_string()
                .contains("off the 1 database-unit grid")
        );
    }

    #[test]
    fn every_policy_snaps_near_integer_scaled_values() {
        let library = library_with_node(Point::float(
            2.999_999_999_999_999_6,
            -2.999_999_999_999_999_6,
            1.0,
        ));
        for policy in [
            GdsRoundingPolicy::Nearest,
            GdsRoundingPolicy::Floor,
            GdsRoundingPolicy::Ceil,
            GdsRoundingPolicy::ErrorIfOffGrid,
        ] {
            let (bytes, report) = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(policy))
                .expect("near-grid value should be writable");
            assert_eq!(record_i32(&bytes, GDSRecord::XY), [3, -3]);
            assert!(report.affected_locations().is_empty());
        }
    }

    #[test]
    fn error_if_off_grid_accepts_ulp_residuals_and_rejects_real_offsets() {
        let exact = library_with_node(Point::float(0.3, -0.3, 1.0));
        let (bytes, _) = exact
            .to_bytes_with_options(
                1.0,
                0.1,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect("decimal grid value should survive binary representation");
        assert_eq!(record_i32(&bytes, GDSRecord::XY), [3, -3]);

        let off_grid = library_with_node(Point::float(0.31, 0.0, 1.0));
        let error = off_grid
            .to_bytes_with_options(
                1.0,
                0.1,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect_err("off-grid value should be rejected");
        insta::assert_snapshot!(
            error.to_string(),
            @"Validation error: Cell 'top', element 0 (node), field 'point[0].x': physical value 0.31 is off the 0.1 database-unit grid"
        );
    }

    #[test]
    fn invalid_units_and_coordinate_values_are_rejected() {
        let library = library_with_node(Point::integer(0, 0, 1.0));
        for database_units in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let error = library
                .to_bytes_with_options(
                    1.0,
                    database_units,
                    zero_timestamp_options(GdsRoundingPolicy::Nearest),
                )
                .expect_err("invalid database units should be rejected");
            assert!(error.to_string().contains("finite and positive"));
        }
        for user_units in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let error = library
                .to_bytes_with_options(
                    user_units,
                    1.0,
                    zero_timestamp_options(GdsRoundingPolicy::Nearest),
                )
                .expect_err("invalid user units should be rejected");
            assert!(error.to_string().contains("finite and positive"));
        }

        let nonfinite = library_with_node(Point::float(f64::NAN, 0.0, 1.0));
        let error = nonfinite
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect_err("nonfinite coordinate should be rejected");
        assert!(error.to_string().contains("field 'point[0].x'"));
        assert!(error.to_string().contains("not finite"));

        for source_units in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let invalid_source = library_with_node(Point::integer(1, 0, source_units));
            let error = invalid_source
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect_err("invalid source units should be rejected");
            assert!(
                error
                    .to_string()
                    .contains("source units must be finite and positive")
            );
        }
    }

    #[test]
    fn real8_header_boundaries_are_checked_and_round_trip() {
        let maximum_inside = f64::from_bits(MAX_GDS_REAL8_MAGNITUDE.to_bits().saturating_sub(1));
        let library = Library::new("real8");
        let (bytes, _) = library
            .to_bytes_with_options(
                MIN_GDS_REAL8_MAGNITUDE,
                maximum_inside,
                zero_timestamp_options(GdsRoundingPolicy::Nearest),
            )
            .expect("representable REAL8 boundaries should be writable");
        assert_eq!(
            record_f64(&bytes, GDSRecord::Units),
            [MIN_GDS_REAL8_MAGNITUDE, maximum_inside]
        );

        let below_minimum = f64::from_bits(MIN_GDS_REAL8_MAGNITUDE.to_bits().saturating_sub(1));
        for (user_units, database_units, field) in [
            (below_minimum, 1.0, "User units"),
            (1.0, MAX_GDS_REAL8_MAGNITUDE, "Database units"),
        ] {
            let error = library
                .to_bytes_with_options(
                    user_units,
                    database_units,
                    zero_timestamp_options(GdsRoundingPolicy::Nearest),
                )
                .expect_err("unrepresentable header unit should be rejected");
            assert!(error.to_string().contains(field));
            assert!(error.to_string().contains("GDS REAL8"));
        }
    }

    #[test]
    fn real8_transform_boundaries_are_checked_with_element_context() {
        let maximum_inside = f64::from_bits(MAX_GDS_REAL8_MAGNITUDE.to_bits().saturating_sub(1));
        for magnification in [MIN_GDS_REAL8_MAGNITUDE, maximum_inside] {
            let reference =
                Reference::new("leaf").with_grid(Grid::default().with_magnification(magnification));
            let mut library = Library::new("real8");
            library.add_cell(Cell::new("leaf"));
            let mut top = Cell::new("top");
            top.add(reference);
            library.add_cell(top);

            let (bytes, _) = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect("representable transform should be writable");
            assert_eq!(record_f64(&bytes, GDSRecord::Mag), [magnification]);
        }

        for (grid, field) in [
            (
                Grid::default().with_magnification(MAX_GDS_REAL8_MAGNITUDE),
                "magnification",
            ),
            (
                Grid::default().with_angle(crate::Radians::new(MAX_GDS_REAL8_MAGNITUDE)),
                "angle",
            ),
            (
                Grid::default().with_angle(crate::Radians::new(f64::MAX)),
                "angle",
            ),
        ] {
            let reference = Reference::new("leaf").with_grid(grid);
            let mut library = Library::new("real8");
            library.add_cell(Cell::new("leaf"));
            let mut top = Cell::new("top");
            top.add(reference);
            library.add_cell(top);

            let error = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect_err("unrepresentable transform should be rejected");
            assert!(
                error
                    .to_string()
                    .contains(&format!("element 0 (reference), field '{field}'"))
            );
        }
    }

    #[test]
    fn coordinate_overflow_is_rejected_before_casting() {
        for value in [f64::from(i32::MAX) + 1.0, f64::from(i32::MIN) - 1.0] {
            let library = library_with_node(Point::float(value, 0.0, 1.0));
            let error = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect_err("out-of-range coordinate should be rejected");
            assert!(error.to_string().contains("outside the GDS i32 range"));
            assert!(error.to_string().contains("Cell 'top', element 0 (node)"));
        }
    }

    #[test]
    fn exact_i32_coordinate_boundaries_are_accepted() {
        let library = library_with_node(Point::integer(i32::MIN, i32::MAX, 1.0));
        let (bytes, report) = library
            .to_bytes_with_options(
                1.0,
                1.0,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect("exact i32 boundaries should be writable");
        assert_eq!(record_i32(&bytes, GDSRecord::XY), [i32::MIN, i32::MAX]);
        assert!(report.affected_locations().is_empty());
    }

    #[test]
    fn path_scalar_overflow_and_nonfinite_values_are_rejected() {
        for (width, begin_extension, expected_field) in [
            (
                Some(Unit::float(f64::from(i32::MAX) + 1.0, 1.0)),
                None,
                "width",
            ),
            (
                None,
                Some(Unit::float(f64::INFINITY, 1.0)),
                "begin_extension",
            ),
        ] {
            let path = Path::new(
                [Point::float(0.0, 0.0, 1.0), Point::float(1.0, 0.0, 1.0)],
                Layer::new(1),
                DataType::new(0),
                None,
                width,
                begin_extension,
                None,
            );
            let mut library = Library::new("conversion");
            let mut cell = Cell::new("top");
            cell.add(path);
            library.add_cell(cell);

            let error = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect_err("invalid path scalar should be rejected");
            assert!(
                error
                    .to_string()
                    .contains(&format!("field '{expected_field}'"))
            );
        }
    }

    #[test]
    fn polygon_must_remain_closed_after_conversion() {
        let polygon = Polygon::new(
            [
                Point::float(0.0, 0.0, 1.0),
                Point::float(1e-16, 0.0, 1.0),
                Point::float(0.0, 1e-16, 1.0),
                Point::float(5e-16, 0.0, 1.0),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(polygon);
        library.add_cell(cell);

        let error = library
            .to_bytes_with_options(
                1.0,
                1e-16,
                zero_timestamp_options(GdsRoundingPolicy::Nearest),
            )
            .expect_err("quantized open polygon should be rejected");
        assert!(
            error
                .to_string()
                .contains("not closed after database-unit conversion")
        );
    }

    #[test]
    fn transform_values_must_be_finite_and_magnification_positive() {
        for grid in [
            Grid::default().with_magnification(0.0),
            Grid::default().with_magnification(f64::NAN),
            Grid::default().with_angle(crate::Radians::new(f64::INFINITY)),
        ] {
            let reference = Reference::new("leaf").with_grid(grid);
            let mut library = Library::new("conversion");
            library.add_cell(Cell::new("leaf"));
            let mut top = Cell::new("top");
            top.add(reference);
            library.add_cell(top);

            let error = library
                .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
                .expect_err("invalid transform should be rejected");
            assert!(
                error.to_string().contains("magnification") || error.to_string().contains("angle")
            );
        }
    }

    #[test]
    fn parallel_conversion_returns_the_first_element_error() {
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(Node::new(
            [Point::float(f64::INFINITY, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Node::new(
            [Point::float(f64::NEG_INFINITY, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(cell);

        for thread_count in [1, 4] {
            let pool = ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .expect("test thread pool should build");
            let error = pool
                .install(|| {
                    library.to_bytes_with_options(
                        1.0,
                        1.0,
                        zero_timestamp_options(GdsRoundingPolicy::Nearest),
                    )
                })
                .expect_err("invalid coordinate should be rejected");
            assert!(error.to_string().contains("element 0 (node)"));
        }
    }

    #[test]
    fn default_parallel_writer_always_returns_the_lowest_element_error() {
        let mut slow_points = points_with_count(MAX_POINTS);
        *slow_points
            .last_mut()
            .expect("maximum-length path should contain points") =
            Point::float(f64::INFINITY, 0.0, DEFAULT_INTEGER_UNITS);
        let slow_invalid = Path::new(
            slow_points,
            Layer::new(1),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
        let fast_invalid = Node::new(
            [Point::float(f64::NEG_INFINITY, 0.0, DEFAULT_INTEGER_UNITS)],
            Layer::new(1),
            DataType::new(0),
        );
        let mut cell = Cell::new("top");
        cell.add(slow_invalid);
        cell.add(fast_invalid);
        let mut library = Library::new("conversion");
        library.add_cell(cell);
        let pool = ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("test thread pool should build");

        for _ in 0..100 {
            let error = pool
                .install(|| {
                    library.write_with_timestamp_policy(
                        &GdsFileWriter,
                        DEFAULT_INTEGER_UNITS,
                        DEFAULT_INTEGER_UNITS,
                        GdsTimestampPolicy::Zero,
                    )
                })
                .expect_err("invalid coordinate should be rejected");
            assert!(error.to_string().contains("element 0 (path)"));
            assert!(
                error
                    .to_string()
                    .contains(&format!("point[{}].x", MAX_POINTS - 1))
            );
        }
    }

    #[test]
    fn report_order_is_cell_then_numeric_element_index() {
        let mut library = Library::new("conversion");
        for name in ["z", "a"] {
            let mut cell = Cell::new(name);
            cell.add(Node::new(
                [Point::float(0.5, 0.0, 1.0)],
                Layer::new(1),
                DataType::new(0),
            ));
            cell.add(Node::new(
                [Point::float(1.5, 0.0, 1.0)],
                Layer::new(1),
                DataType::new(0),
            ));
            library.add_cell(cell);
        }

        let (_, report) = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect("library should be writable");
        assert_eq!(
            report
                .affected_locations()
                .iter()
                .map(|location| (location.cell_name(), location.element_index()))
                .collect::<Vec<_>>(),
            [("a", 0), ("a", 1), ("z", 0), ("z", 1)]
        );
    }

    #[test]
    fn width_and_extensions_use_the_selected_rounding_policy() {
        let path = Path::new(
            [Point::float(0.0, 0.0, 1.0), Point::float(1.0, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
            None,
            Some(Unit::float(0.5, 1.0)),
            Some(Unit::float(-0.5, 1.0)),
            Some(Unit::float(1.5, 1.0)),
        );
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(path);
        library.add_cell(cell);

        let (bytes, report) = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect("path fields should be writable");

        assert_eq!(record_i32(&bytes, GDSRecord::Width), [1]);
        assert_eq!(record_i32(&bytes, GDSRecord::BgnExtn), [-1]);
        assert_eq!(record_i32(&bytes, GDSRecord::EndExtn), [2]);
        assert_eq!(report.affected_locations()[0].affected_field_count(), 3);
        assert_eq!(report.affected_locations()[0].field(), "width");
    }

    #[test]
    fn aref_spacing_and_derived_endpoints_are_checked() {
        let reference = Reference::new("leaf").with_grid(
            Grid::default()
                .with_columns(2)
                .with_spacing_x(Some(Point::float(0.5, 0.0, 1.0))),
        );
        let mut library = Library::new("conversion");
        library.add_cell(Cell::new("leaf"));
        let mut top = Cell::new("top");
        top.add(reference);
        library.add_cell(top);

        let (bytes, report) = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect("array reference should be writable");
        assert_eq!(record_i32(&bytes, GDSRecord::XY), [0, 0, 2, 0, 0, 0]);
        assert_eq!(report.max_physical_quantization_error(), 1.0);
        assert_eq!(report.affected_locations()[0].field(), "aref_endpoint[1].x");
        assert_eq!(report.affected_locations()[0].affected_field_count(), 2);

        let overflow =
            Reference::new("leaf").with_grid(Grid::default().with_columns(2).with_spacing_x(Some(
                Point::float(f64::from(i32::MAX) / 2.0 + 1.0, 0.0, 1.0),
            )));
        let mut top = Cell::new("top");
        top.add(overflow);
        let mut library = Library::new("overflow");
        library.add_cell(Cell::new("leaf"));
        library.add_cell(top);
        let error = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect_err("derived AREF endpoint overflow should be rejected");
        assert!(error.to_string().contains("aref_endpoint[1].x"));
        assert!(error.to_string().contains("outside the GDS i32 range"));
    }

    #[test]
    fn array_endpoint_rejects_nonfinite_accumulated_physical_value() {
        let finite_spacing = f64::MAX / 2.0;
        let count = u32::MAX;
        assert!(finite_spacing.is_finite());
        let accumulated = finite_spacing * f64::from(count);
        assert!(!accumulated.is_finite());

        let mut converter =
            CoordinateConverter::new(1.0, GdsRoundingPolicy::Nearest, "top", 0, "reference")
                .expect("converter should be created");
        let error = converter
            .checked_array_endpoint(0, 0, count, accumulated, "aref_endpoint[1].x")
            .expect_err("nonfinite accumulated physical endpoint should be rejected");
        assert!(error.to_string().contains("aref_endpoint[1].x"));
        assert!(
            error
                .to_string()
                .contains("physical value inf is not finite")
        );
        assert!(converter.finish().affected_locations().is_empty());
    }

    #[test]
    fn rotated_aref_snaps_per_instance_spacing_before_full_count_endpoint() {
        let reference = Reference::new("leaf").with_grid(
            Grid::default()
                .with_origin(Point::integer(10, 20, 1.0))
                .with_columns(2)
                .with_spacing_x(Some(Point::float(0.5, 0.0, 1.0)))
                .with_angle(crate::Radians::new(std::f64::consts::FRAC_PI_2)),
        );
        let mut library = Library::new("conversion");
        library.add_cell(Cell::new("leaf"));
        let mut top = Cell::new("top");
        top.add(reference);
        library.add_cell(top);

        let (bytes, report) = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect("rotated array should be writable");
        assert_eq!(record_i32(&bytes, GDSRecord::XY), [10, 20, 10, 22, 10, 20]);
        assert_eq!(report.max_physical_quantization_error(), 1.0);
        assert_eq!(report.affected_locations()[0].field(), "aref_endpoint[1].y");
    }

    #[test]
    fn inline_reference_conversion_cannot_saturate_before_validation() {
        let node = Node::new(
            [Point::integer(i32::MAX, 0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        let reference =
            Reference::new(Element::from(node)).with_grid(Grid::default().with_magnification(2.0));
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(reference);
        library.add_cell(cell);

        let error = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect_err("expanded coordinate overflow should be rejected");
        assert!(error.to_string().contains("element 0 (reference)"));
        assert!(error.to_string().contains("inline[0,0].point[0].x"));
    }

    #[test]
    fn inline_reference_applies_policy_only_to_cancelled_emitted_coordinates() {
        let node = Node::new(
            [Point::float(0.75, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        let reference = Reference::new(Element::from(node))
            .with_grid(Grid::default().with_origin(Point::float(0.25, 0.0, 1.0)));
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(reference);
        library.add_cell(cell);

        let (bytes, report) = library
            .to_bytes_with_options(
                1.0,
                1.0,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect("off-grid inputs that cancel to an exact emitted point should be writable");
        assert_eq!(record_i32(&bytes, GDSRecord::XY), [1, 0]);
        assert!(report.affected_locations().is_empty());
    }

    #[test]
    fn inline_reference_rounds_only_rotated_emitted_spacing() {
        let node = Node::new(
            [Point::float(0.0, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        let reference = Reference::new(Element::from(node)).with_grid(
            Grid::default()
                .with_columns(2)
                .with_spacing_x(Some(Point::float(0.0, std::f64::consts::SQRT_2, 1.0)))
                .with_angle(crate::Radians::new(std::f64::consts::FRAC_PI_4)),
        );
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(reference);
        library.add_cell(cell);

        let (bytes, report) = library
            .to_bytes_with_options(
                1.0,
                1.0,
                zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid),
            )
            .expect("rotated spacing that emits exact points should be writable");
        assert_eq!(
            records_i32(&bytes, GDSRecord::XY),
            [vec![0, 0], vec![-1, 1]]
        );
        assert!(report.affected_locations().is_empty());
    }

    #[test]
    fn inline_reference_validates_unemitted_source_units() {
        let node = Node::new(
            [Point::float(0.0, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        let reference = Reference::new(Element::from(node))
            .with_grid(Grid::default().with_origin(Point::integer(1, 0, 0.0)));
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(reference);
        library.add_cell(cell);

        let error = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect_err("invalid raw inline grid units should be rejected");
        assert!(error.to_string().contains("field 'origin.x'"));
        assert!(
            error
                .to_string()
                .contains("source units must be finite and positive")
        );
    }

    #[test]
    fn nested_inline_reference_errors_keep_every_expansion_index() {
        let node = Node::new(
            [Point::integer(i32::MAX, 0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        let inner =
            Reference::new(Element::from(node)).with_grid(Grid::default().with_magnification(2.0));
        let outer = Reference::new(Element::from(inner));
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(outer);
        library.add_cell(cell);

        let error = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect_err("nested expanded overflow should be rejected");
        assert!(
            error
                .to_string()
                .contains("inline[0,0].inline[0,0].point[0].x")
        );
    }

    #[test]
    fn streaming_and_in_memory_options_share_conversion_results() {
        let library = library_with_node(Point::float(0.5, -0.5, 1.0));
        for policy in [GdsRoundingPolicy::Floor, GdsRoundingPolicy::Ceil] {
            let options = zero_timestamp_options(policy);
            let (expected_bytes, expected_report) = library
                .to_bytes_with_options(1.0, 1.0, options)
                .expect("in-memory write should succeed");
            let mut streamed_bytes = Vec::new();
            let streamed_report = library
                .write_to_with_options(&mut streamed_bytes, 1.0, 1.0, options)
                .expect("stream write should succeed");

            assert_eq!(streamed_bytes, expected_bytes);
            assert_eq!(streamed_report, expected_report);
        }

        let options = zero_timestamp_options(GdsRoundingPolicy::ErrorIfOffGrid);
        let expected_error = library
            .to_bytes_with_options(1.0, 1.0, options)
            .expect_err("in-memory write should reject off-grid coordinates");
        let streamed_error = library
            .write_to_with_options(Vec::new(), 1.0, 1.0, options)
            .expect_err("stream write should reject off-grid coordinates");
        assert_eq!(streamed_error.to_string(), expected_error.to_string());
    }

    #[test]
    fn failed_stream_cell_does_not_change_accumulated_report() {
        let options = zero_timestamp_options(GdsRoundingPolicy::Nearest);
        let mut writer = GdsStreamWriter::new_with_options(Vec::new(), "stream", 1.0, 1.0, options)
            .expect("stream should start");
        let mut valid = Cell::new("valid");
        valid.add(Node::new(
            [Point::float(0.5, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        writer.write_cell(&valid).expect("valid cell should write");
        let report_before_error = writer.conversion_report().clone();

        let mut invalid = Cell::new("invalid");
        invalid.add(Node::new(
            [Point::float(f64::INFINITY, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        writer
            .write_cell(&invalid)
            .expect_err("invalid cell should fail");

        assert_eq!(writer.conversion_report(), &report_before_error);
    }

    #[test]
    fn live_stream_report_uses_write_order_and_finished_report_is_sorted() {
        let options = zero_timestamp_options(GdsRoundingPolicy::Nearest);
        let mut writer = GdsStreamWriter::new_with_options(Vec::new(), "stream", 1.0, 1.0, options)
            .expect("stream should start");
        for name in ["z", "a"] {
            let mut cell = Cell::new(name);
            cell.add(Node::new(
                [Point::float(0.5, 0.0, 1.0)],
                Layer::new(1),
                DataType::new(0),
            ));
            writer.write_cell(&cell).expect("cell should write");
        }
        assert_eq!(
            writer
                .conversion_report()
                .affected_locations()
                .iter()
                .map(GdsQuantization::cell_name)
                .collect::<Vec<_>>(),
            ["z", "a"]
        );

        let (_, report) = writer.finish_with_report().expect("stream should finish");
        assert_eq!(
            report
                .affected_locations()
                .iter()
                .map(GdsQuantization::cell_name)
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
    }

    #[test]
    fn properties_survive_quantized_writes_at_attribute_boundary() {
        let mut node = Node::new(
            [Point::float(0.5, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        );
        node.properties_mut()
            .push(Property::new(u16::MAX, "boundary"));
        let mut library = Library::new("conversion");
        let mut cell = Cell::new("top");
        cell.add(node);
        library.add_cell(cell);

        let (bytes, _) = library
            .to_bytes_with_options(1.0, 1.0, zero_timestamp_options(GdsRoundingPolicy::Nearest))
            .expect("property boundary should remain writable");
        let records = RecordReader::new(BufReader::new(bytes.as_slice()))
            .collect::<Result<Vec<_>, _>>()
            .expect("records should parse");
        assert!(records.iter().any(|(record, data)| matches!(
            (record, data),
            (GDSRecord::PropAttr, GDSRecordData::I16(values)) if values == &[-1]
        )));
    }

    #[test]
    fn arbitrary_float_bits_never_panic_during_conversion() {
        let mut bits = 0_u64;
        for _ in 0..1024 {
            bits = bits
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            for policy in [
                GdsRoundingPolicy::Nearest,
                GdsRoundingPolicy::Floor,
                GdsRoundingPolicy::Ceil,
                GdsRoundingPolicy::ErrorIfOffGrid,
            ] {
                let mut converter = CoordinateConverter::new(1e-9, policy, "bits", 0, "node")
                    .expect("database units should be valid");
                let _result = converter.convert_physical(f64::from_bits(bits), "point[0].x");
            }
        }
    }

    #[test]
    fn default_writer_preserves_context_for_later_inline_reference_errors() {
        let mut cell = Cell::new("top");
        cell.add(Node::new(
            [Point::integer(0, 0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Reference::new(Element::from(Node::new(
            [Point::float(f64::INFINITY, 0.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
        ))));
        let mut library = Library::new("conversion");
        library.add_cell(cell);

        let error = library
            .write_with_timestamp_policy(&GdsFileWriter, 1.0, 1.0, GdsTimestampPolicy::Zero)
            .expect_err("invalid inline coordinate should be rejected");
        assert!(
            error
                .to_string()
                .contains("Cell 'top', element 1 (reference), field 'inline[0,0].point[0].x'")
        );
    }

    #[test]
    fn properties_are_written_before_end_element() {
        let bytes = write_polygon(&polygon_with_property("net-a"), DEFAULT_INTEGER_UNITS)
            .expect("polygon should be writable");
        let records: Vec<_> = RecordReader::new(BufReader::new(bytes.as_slice()))
            .collect::<Result<_, _>>()
            .expect("element records should be readable");
        let record_types: Vec<_> = records.iter().map(|(record, _)| *record).collect();

        assert_eq!(
            record_types,
            [
                GDSRecord::Boundary,
                GDSRecord::Layer,
                GDSRecord::DataType,
                GDSRecord::XY,
                GDSRecord::PropAttr,
                GDSRecord::PropValue,
                GDSRecord::EndEl,
            ]
        );
        assert!(matches!(
            &records[4].1,
            GDSRecordData::I16(attributes) if attributes == &[-1]
        ));
        assert!(matches!(
            &records[5].1,
            GDSRecordData::Str(value) if value == "net-a"
        ));
    }

    #[test]
    fn oversized_property_value_is_rejected() {
        let property_value = "x".repeat(usize::from(u16::MAX));

        let error = write_polygon(
            &polygon_with_property(property_value),
            DEFAULT_INTEGER_UNITS,
        )
        .expect_err("oversized property value should be rejected");

        assert!(matches!(error, GdsError::ValidationError { .. }));
    }

    #[test]
    fn library_cells_are_written_in_name_order() {
        let cells: Vec<_> = (0..32)
            .rev()
            .map(|index| (format!("cell_{index:02}"), index))
            .collect();
        let library = library_with_cells(&cells);

        let bytes = library
            .write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
            .expect("library should be writable");
        let names: Vec<_> = RecordReader::new(BufReader::new(bytes.as_slice()))
            .filter_map(
                |record| match record.expect("library records should be readable") {
                    (GDSRecord::StrName, GDSRecordData::Str(name)) => Some(name),
                    _ => None,
                },
            )
            .collect();
        let expected: Vec<_> = (0..32).map(|index| format!("cell_{index:02}")).collect();

        assert_eq!(names, expected);
    }

    #[test]
    fn serialization_ignores_insertion_order_and_rayon_thread_count() {
        let cells: Vec<_> = (0..32)
            .map(|index| (format!("cell_{index:02}"), index))
            .collect();
        let shuffled_cells: Vec<_> = (0..32)
            .map(|index| cells[(index * 17) % cells.len()].clone())
            .collect();
        let forward_library = library_with_cells(&cells);
        let shuffled_library = library_with_cells(&shuffled_cells);
        let single_threaded = ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .expect("single-threaded pool should be buildable")
            .install(|| {
                forward_library.write_with_timestamp_policy(
                    &GdsFileWriter,
                    DEFAULT_INTEGER_UNITS,
                    DEFAULT_INTEGER_UNITS,
                    GdsTimestampPolicy::Zero,
                )
            })
            .expect("forward library should be writable");
        let multi_threaded = ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("multi-threaded pool should be buildable")
            .install(|| {
                shuffled_library.write_with_timestamp_policy(
                    &GdsFileWriter,
                    DEFAULT_INTEGER_UNITS,
                    DEFAULT_INTEGER_UNITS,
                    GdsTimestampPolicy::Zero,
                )
            })
            .expect("shuffled library should be writable");

        assert_eq!(single_threaded, multi_threaded);

        let mut streamed = Vec::new();
        shuffled_library
            .write_to_with_timestamp_policy(
                &mut streamed,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
            .expect("shuffled library should be streamable");
        assert_eq!(single_threaded, streamed);

        let layers: Vec<_> = RecordReader::new(BufReader::new(single_threaded.as_slice()))
            .filter_map(
                |record| match record.expect("library records should be readable") {
                    (GDSRecord::Layer, GDSRecordData::I16(layer)) => layer.first().copied(),
                    _ => None,
                },
            )
            .collect();
        let expected: Vec<_> = (0..32).flat_map(|layer| [layer, layer + 64]).collect();

        assert_eq!(layers, expected);
    }

    #[test]
    fn inline_reference_properties_are_written_on_expanded_elements() {
        let mut reference = Reference::new(polygon_with_property("inner"));
        reference.properties_mut().push(Property::new(7, "outer"));

        let bytes = write_reference(&reference, DEFAULT_INTEGER_UNITS)
            .expect("inline reference should be writable");
        let property_values: Vec<_> = RecordReader::new(BufReader::new(bytes.as_slice()))
            .filter_map(
                |record| match record.expect("element records should be readable") {
                    (GDSRecord::PropValue, GDSRecordData::Str(value)) => Some(value),
                    _ => None,
                },
            )
            .collect();

        assert_eq!(property_values, ["inner", "outer"]);
    }

    #[test]
    fn inline_reference_grid_without_spacing_remains_writable() {
        let reference = Reference::new(polygon_with_property("inner"))
            .with_grid(Grid::default().with_columns(2).with_rows(2));

        let bytes = write_reference(&reference, DEFAULT_INTEGER_UNITS)
            .expect("inline grid should use zero spacing");
        let boundaries = RecordReader::new(BufReader::new(bytes.as_slice()))
            .filter(|record| matches!(record, Ok((GDSRecord::Boundary, GDSRecordData::None))))
            .count();

        assert_eq!(boundaries, 4);
    }

    #[test]
    fn invalid_public_writer_states_return_validation_errors() {
        assert_validation_error(&GdsFileWriter.write_cell(&Cell::new(""), DEFAULT_INTEGER_UNITS));
        assert_validation_error(&write_reference(&Reference::new(""), DEFAULT_INTEGER_UNITS));
        assert_validation_error(&write_node(&Node::default(), DEFAULT_INTEGER_UNITS));

        let spacing_x = Point::integer(10, 0, DEFAULT_INTEGER_UNITS);
        let spacing_y = Point::integer(0, 10, DEFAULT_INTEGER_UNITS);
        for grid in [
            Grid::default().with_columns(0),
            Grid::default().with_rows(0),
            Grid::default()
                .with_columns(validation::MAX_COL_ROW + 1)
                .with_spacing_x(Some(spacing_x)),
            Grid::default()
                .with_rows(validation::MAX_COL_ROW + 1)
                .with_spacing_y(Some(spacing_y)),
            Grid::default().with_columns(2),
            Grid::default().with_rows(2),
        ] {
            assert_validation_error(&write_reference(
                &Reference::new("target").with_grid(grid),
                DEFAULT_INTEGER_UNITS,
            ));
        }
    }

    #[test]
    fn valid_array_references_always_write_three_points_and_reread() {
        let spacing_x = Point::integer(10, 0, DEFAULT_INTEGER_UNITS);
        let spacing_y = Point::integer(0, 10, DEFAULT_INTEGER_UNITS);
        for grid in [
            Grid::default()
                .with_columns(2)
                .with_spacing_x(Some(spacing_x)),
            Grid::default().with_rows(2).with_spacing_y(Some(spacing_y)),
            Grid::default()
                .with_columns(2)
                .with_rows(2)
                .with_spacing_x(Some(spacing_x))
                .with_spacing_y(Some(spacing_y)),
        ] {
            let columns = grid.columns();
            let rows = grid.rows();
            let reference = Reference::new("target").with_grid(grid);
            let element_bytes = write_reference(&reference, DEFAULT_INTEGER_UNITS)
                .expect("valid AREF should be writable");
            let records = RecordReader::new(BufReader::new(element_bytes.as_slice()))
                .collect::<Result<Vec<_>, _>>()
                .expect("written AREF records should decode");
            assert!(records.iter().any(|(record, data)| matches!(
                (record, data),
                (GDSRecord::XY, GDSRecordData::I32(values)) if values.len() == 6
            )));

            let mut top = Cell::new("top");
            top.add(reference);
            let mut library = Library::new("arrays");
            library.add_cell(Cell::new("target"));
            library.add_cell(top);
            let bytes = library
                .write_with_timestamp_policy(
                    &GdsFileWriter,
                    DEFAULT_INTEGER_UNITS,
                    DEFAULT_INTEGER_UNITS,
                    GdsTimestampPolicy::Zero,
                )
                .expect("library with valid AREF should be writable");
            let parsed =
                Library::from_bytes(&bytes, None).expect("written AREF should be readable");
            let parsed_reference = parsed
                .get_cell("top")
                .and_then(|cell| cell.elements().first())
                .and_then(|element| match element {
                    Element::Reference(reference) => Some(reference),
                    _ => None,
                })
                .expect("top element should reread as a reference");
            assert_eq!(parsed_reference.grid().columns(), columns);
            assert_eq!(parsed_reference.grid().rows(), rows);
        }
    }

    #[test]
    fn stream_writer_flushes_each_cell_and_produces_a_readable_library() {
        let output = FlushCountingWriter::default();
        let mut writer = GdsStreamWriter::new(
            output,
            "streamed",
            DEFAULT_INTEGER_UNITS,
            DEFAULT_INTEGER_UNITS,
        )
        .expect("library header should be writable");

        writer
            .write_cell(&Cell::new("first"))
            .expect("first cell should be writable");
        writer
            .write_cell(&Cell::new("second"))
            .expect("second cell should be writable");
        let output = writer.finish().expect("library footer should be writable");

        assert_eq!(output.flushes, 4);

        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let path = directory.path().join("streamed.gds");
        fs::write(&path, output.bytes).expect("streamed bytes should be writable");
        let library = Library::read_file(path, Some(DEFAULT_INTEGER_UNITS))
            .expect("streamed library should be readable");

        assert_eq!(library.name(), "streamed");
        assert!(library.get_cell("first").is_some());
        assert!(library.get_cell("second").is_some());
    }

    #[test]
    fn invalid_library_name_does_not_write_a_partial_stream_header() {
        let mut output = Vec::new();
        let library_name = "x".repeat(usize::from(u16::MAX));
        let error = GdsStreamWriter::new(
            &mut output,
            &library_name,
            DEFAULT_INTEGER_UNITS,
            DEFAULT_INTEGER_UNITS,
        )
        .err()
        .expect("oversized library name should be rejected");

        assert!(matches!(error, GdsError::ValidationError { .. }));
        assert!(output.is_empty());
    }

    #[test]
    fn preserve_round_trips_library_and_structure_timestamps() {
        let library_timestamps = timestamps(2020, 2021);
        let cell_timestamps = timestamps(2022, 2023);
        let mut library = Library::new("preserved");
        library.set_timestamps(library_timestamps);
        let mut cell = Cell::new("top");
        cell.set_timestamps(cell_timestamps);
        library.add_cell(cell);

        let bytes = library
            .write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Preserve,
            )
            .expect("preserved timestamps should be writable");
        let parsed = read_library_bytes(&bytes).expect("written library should be readable");

        assert_eq!(parsed.timestamps(), Some(library_timestamps));
        assert_eq!(
            parsed
                .get_cell("top")
                .expect("parsed cell should exist")
                .timestamps(),
            Some(cell_timestamps)
        );
    }

    #[test]
    fn zero_policy_is_stable_and_readable() {
        let mut library = Library::new("stable");
        library.add_cell(Cell::new("top"));

        let write = || {
            library.write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
        };
        let first = write().expect("zero timestamps should be writable");
        let second = write().expect("zero timestamps should be writable again");
        let parsed = read_library_bytes(&first).expect("zero timestamps should be readable");

        assert_eq!(first, second);
        assert_eq!(parsed.timestamps(), Some(GdsTimestamps::ZERO));
        assert_eq!(
            parsed
                .get_cell("top")
                .expect("parsed cell should exist")
                .timestamps(),
            Some(GdsTimestamps::ZERO)
        );
    }

    #[test]
    fn current_policy_uses_one_time_for_the_entire_library() {
        let mut library = Library::new("current");
        library.add_cell(Cell::new("first"));
        library.add_cell(Cell::new("second"));
        let bytes = library
            .write(&GdsFileWriter, DEFAULT_INTEGER_UNITS, DEFAULT_INTEGER_UNITS)
            .expect("current timestamps should be writable");

        let timestamps: Vec<_> = RecordReader::new(BufReader::new(bytes.as_slice()))
            .filter_map(|record| {
                let (record, data) = record.expect("written records should be readable");
                match (record, data) {
                    (GDSRecord::BgnLib | GDSRecord::BgnStr, GDSRecordData::I16(values)) => {
                        Some(values)
                    }
                    _ => None,
                }
            })
            .collect();

        assert_eq!(timestamps.len(), 3);
        assert!(timestamps.windows(2).all(|pair| pair[0] == pair[1]));
        assert!(timestamps.iter().all(|values| values[..6] == values[6..]));
    }

    #[test]
    fn preserve_requires_library_and_structure_metadata() {
        let mut library = Library::new("missing");
        library.add_cell(Cell::new("top"));

        let error = library
            .write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Preserve,
            )
            .expect_err("missing library timestamps should be rejected");
        assert!(error.to_string().contains("library 'missing'"));

        library.set_timestamps(timestamps(2020, 2021));
        let error = library
            .write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Preserve,
            )
            .expect_err("missing cell timestamps should be rejected");
        assert!(error.to_string().contains("cell 'top'"));
    }

    #[test]
    fn invalid_preserved_date_reports_the_raw_values() {
        let library = Library::new("invalid");
        let mut bytes = library
            .write_with_timestamp_policy(
                &GdsFileWriter,
                DEFAULT_INTEGER_UNITS,
                DEFAULT_INTEGER_UNITS,
                GdsTimestampPolicy::Zero,
            )
            .expect("zero timestamps should be writable");

        // HEADER is six bytes; the first BGNLIB month occupies bytes 12..14.
        bytes
            .get_mut(12..14)
            .expect("BGNLIB timestamp should exist")
            .copy_from_slice(&13_i16.to_be_bytes());
        let error = read_library_bytes(&bytes).expect_err("month 13 should be rejected");

        assert!(error.to_string().contains("BGNLIB"));
        assert!(error.to_string().contains("13"));
    }

    #[test]
    fn stream_writer_preserves_library_and_structure_timestamps() {
        let library_timestamps = timestamps(2020, 2021);
        let cell_timestamps = timestamps(2022, 2023);
        let mut library = Library::new("streamed-preserve");
        library.set_timestamps(library_timestamps);
        let mut cell = Cell::new("top");
        cell.set_timestamps(cell_timestamps);
        library.add_cell(cell);

        let mut writer = GdsStreamWriter::from_library_with_timestamp_policy(
            Vec::new(),
            &library,
            DEFAULT_INTEGER_UNITS,
            DEFAULT_INTEGER_UNITS,
            GdsTimestampPolicy::Preserve,
        )
        .expect("stream header should be writable");
        writer
            .write_cell(library.get_cell("top").expect("source cell should exist"))
            .expect("streamed cell should be writable");
        let bytes = writer.finish().expect("stream should finish");
        let parsed = read_library_bytes(&bytes).expect("streamed library should be readable");

        assert_eq!(parsed.timestamps(), Some(library_timestamps));
        assert_eq!(
            parsed
                .get_cell("top")
                .expect("parsed cell should exist")
                .timestamps(),
            Some(cell_timestamps)
        );
    }
}
