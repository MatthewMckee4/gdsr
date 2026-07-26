use crate::error::GdsError;
use crate::{DataType, Layer, Point};

pub const MAX_POINTS: usize = 8191;
pub const MAX_LAYER: u16 = 255;
pub const MAX_DATA_TYPE: u16 = 255;
pub const MAX_STRING_LENGTH: usize = 512;
pub const MAX_STRUCTURE_NAME_LENGTH: usize = 32;
pub const MAX_COL_ROW: u32 = 32767;
pub const MIN_POLYGON_POINTS: usize = 4;

/// Returns an `InvalidInput` error if the layer is out of the GDS2 spec range (0-255).
pub fn validate_layer(layer: Layer) -> Result<(), GdsError> {
    if layer.value() > MAX_LAYER {
        return Err(GdsError::ValidationError {
            message: format!(
                "Layer {} exceeds maximum value of {MAX_LAYER}",
                layer.value()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the data type is out of the GDS2 spec range (0-255).
pub fn validate_data_type(data_type: DataType) -> Result<(), GdsError> {
    if data_type.value() > MAX_DATA_TYPE {
        return Err(GdsError::ValidationError {
            message: format!(
                "Data type {} exceeds maximum value of {MAX_DATA_TYPE}",
                data_type.value()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the string exceeds 512 characters.
pub fn validate_string_length(s: &str) -> Result<(), GdsError> {
    if s.len() > MAX_STRING_LENGTH {
        return Err(GdsError::ValidationError {
            message: format!(
                "String length {} exceeds maximum of {MAX_STRING_LENGTH} characters",
                s.len()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the structure name exceeds 32 characters or contains
/// invalid characters (only alphanumeric, `_`, `?`, `$` are allowed).
pub fn validate_structure_name(name: &str) -> Result<(), GdsError> {
    if name.is_empty() {
        return Err(GdsError::ValidationError {
            message: "Structure name cannot be empty".to_string(),
        });
    }
    if name.len() > MAX_STRUCTURE_NAME_LENGTH {
        return Err(GdsError::ValidationError {
            message: format!(
                "Structure name length {} exceeds maximum of {MAX_STRUCTURE_NAME_LENGTH} characters",
                name.len()
            ),
        });
    }
    if let Some(c) = name
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '?' && *c != '$')
    {
        return Err(GdsError::ValidationError {
            message: format!("Structure name contains invalid character: '{c}'"),
        });
    }
    Ok(())
}

/// Returns a validation error if columns or rows are outside 1..=32767.
pub fn validate_col_row(columns: u32, rows: u32) -> Result<(), GdsError> {
    if !(1..=MAX_COL_ROW).contains(&columns) {
        return Err(GdsError::ValidationError {
            message: format!("Column count {columns} must be between 1 and {MAX_COL_ROW}"),
        });
    }
    if !(1..=MAX_COL_ROW).contains(&rows) {
        return Err(GdsError::ValidationError {
            message: format!("Row count {rows} must be between 1 and {MAX_COL_ROW}"),
        });
    }
    Ok(())
}

/// Returns a validation error if a node has no points.
pub fn validate_node_points(points: &[Point]) -> Result<(), GdsError> {
    if points.is_empty() {
        return Err(GdsError::ValidationError {
            message: "Node must have at least one point".to_string(),
        });
    }
    Ok(())
}

pub const MIN_PATH_POINTS: usize = 2;

/// Returns a validation error if a path has fewer than 2 points.
pub fn validate_path_points(points: &[Point]) -> Result<(), GdsError> {
    if points.len() < MIN_PATH_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Path must have at least {MIN_PATH_POINTS} points, got {}",
                points.len()
            ),
        });
    }
    Ok(())
}

/// Validates that a polygon has the correct number of points for GDS serialization.
pub fn validate_polygon_points(points: &[Point]) -> Result<(), GdsError> {
    if points.len() > MAX_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon has {} points, which exceeds the maximum of {}",
                points.len(),
                MAX_POINTS
            ),
        });
    }

    if points.len() < MIN_POLYGON_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon must have at least {MIN_POLYGON_POINTS} points (3 vertices + closing point), got {}",
                points.len()
            ),
        });
    }

    Ok(())
}
