use std::fmt;

use super::gds_format::eight_byte_real;
use crate::{GdsError, GdsTimestampPolicy, Unit};

/// Rounding applied when physical values do not fall on the output database grid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GdsRoundingPolicy {
    /// Round to the nearest grid point. Half-grid values round away from zero.
    #[default]
    Nearest,
    /// Round toward negative infinity.
    ///
    /// Scaled values within one ULP of an integer are treated as exact.
    Floor,
    /// Round toward positive infinity.
    ///
    /// Scaled values within one ULP of an integer are treated as exact.
    Ceil,
    /// Reject values that do not fall on the database grid.
    ///
    /// A one-ULP tolerance avoids rejecting exact physical values solely because of
    /// binary floating-point representation.
    ErrorIfOffGrid,
}

/// Options for built-in GDS serialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GdsWriteOptions {
    rounding_policy: GdsRoundingPolicy,
    timestamp_policy: GdsTimestampPolicy,
    collect_report: bool,
}

impl Default for GdsWriteOptions {
    fn default() -> Self {
        Self::new(GdsRoundingPolicy::Nearest)
    }
}

impl GdsWriteOptions {
    /// Creates options using `rounding_policy` and current timestamps.
    pub const fn new(rounding_policy: GdsRoundingPolicy) -> Self {
        Self {
            rounding_policy,
            timestamp_policy: GdsTimestampPolicy::Current,
            collect_report: true,
        }
    }

    /// Returns the coordinate rounding policy.
    pub const fn rounding_policy(self) -> GdsRoundingPolicy {
        self.rounding_policy
    }

    /// Returns the timestamp policy.
    pub const fn timestamp_policy(self) -> GdsTimestampPolicy {
        self.timestamp_policy
    }

    /// Sets the timestamp policy.
    #[must_use]
    pub const fn with_timestamp_policy(mut self, timestamp_policy: GdsTimestampPolicy) -> Self {
        self.timestamp_policy = timestamp_policy;
        self
    }

    pub(super) const fn without_report(mut self) -> Self {
        self.collect_report = false;
        self
    }

    pub(super) const fn collects_report(self) -> bool {
        self.collect_report
    }
}

/// One element affected by database-grid quantization.
#[derive(Clone, Debug, PartialEq)]
pub struct GdsQuantization {
    cell_name: String,
    element_index: usize,
    element_type: String,
    representative_field: String,
    affected_field_count: usize,
    physical_value: f64,
    database_value: i32,
    physical_error: f64,
}

impl GdsQuantization {
    /// Returns the containing cell name.
    pub fn cell_name(&self) -> &str {
        &self.cell_name
    }

    /// Returns the element index within the cell.
    pub const fn element_index(&self) -> usize {
        self.element_index
    }

    /// Returns the element kind.
    pub fn element_type(&self) -> &str {
        &self.element_type
    }

    /// Returns a field with the element's largest quantization error.
    pub fn field(&self) -> &str {
        &self.representative_field
    }

    /// Returns how many fields in this element changed.
    pub const fn affected_field_count(&self) -> usize {
        self.affected_field_count
    }

    /// Returns the source physical value in metres.
    pub const fn physical_value(&self) -> f64 {
        self.physical_value
    }

    /// Returns the written database-unit value.
    pub const fn database_value(&self) -> i32 {
        self.database_value
    }

    /// Returns the absolute physical quantization error in metres.
    pub const fn physical_error(&self) -> f64 {
        self.physical_error
    }
}

/// Summary of physical values changed while writing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GdsConversionReport {
    max_physical_quantization_error: f64,
    affected_locations: Vec<GdsQuantization>,
}

impl GdsConversionReport {
    /// Returns the largest absolute physical quantization error in metres.
    pub const fn max_physical_quantization_error(&self) -> f64 {
        self.max_physical_quantization_error
    }

    /// Returns affected elements in this report's established order.
    ///
    /// Completed library writes and finished stream reports are sorted by cell name
    /// and numeric element index. A report borrowed from a live stream writer uses
    /// cell write order until that writer is finished.
    pub fn affected_locations(&self) -> &[GdsQuantization] {
        &self.affected_locations
    }

    pub(super) fn extend(&mut self, mut other: Self) {
        self.max_physical_quantization_error = self
            .max_physical_quantization_error
            .max(other.max_physical_quantization_error);
        for location in other.affected_locations.drain(..) {
            if let Some(existing) = self.affected_locations.last_mut()
                && existing.cell_name == location.cell_name
                && existing.element_index == location.element_index
                && existing.element_type == location.element_type
            {
                existing.affected_field_count += location.affected_field_count;
                if location.physical_error > existing.physical_error {
                    existing.representative_field = location.representative_field;
                    existing.physical_value = location.physical_value;
                    existing.database_value = location.database_value;
                    existing.physical_error = location.physical_error;
                }
            } else {
                self.affected_locations.push(location);
            }
        }
    }

    pub(super) fn sort(&mut self) {
        self.affected_locations.sort_by(|left, right| {
            (
                &left.cell_name,
                left.element_index,
                &left.element_type,
                &left.representative_field,
            )
                .cmp(&(
                    &right.cell_name,
                    right.element_index,
                    &right.element_type,
                    &right.representative_field,
                ))
        });
    }
}

pub(super) struct CoordinateConverter<'a> {
    database_units: f64,
    policy: GdsRoundingPolicy,
    cell_name: &'a str,
    element_index: usize,
    element_type: &'a str,
    field_prefix: Option<String>,
    collect_report: bool,
    report: GdsConversionReport,
}

pub(super) trait CoordinateConversion {
    fn convert_unit(&mut self, value: Unit, field: impl fmt::Display) -> Result<i32, GdsError>;
    fn physical_value(
        &self,
        value: Unit,
        field: &(impl fmt::Display + ?Sized),
    ) -> Result<f64, GdsError>;
    fn convert_physical(
        &mut self,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError>;
    fn checked_array_endpoint(
        &mut self,
        origin: i32,
        spacing: i32,
        count: u32,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError>;
    fn validate_finite(&self, value: f64, field: &str) -> Result<(), GdsError>;
    fn validate_positive(&self, value: f64, field: &str) -> Result<(), GdsError>;
    fn encode_gds_real8(&self, value: f64, field: &str) -> Result<[u8; 8], GdsError>;
    fn validate_closed(&self, first: [i32; 2], last: [i32; 2], field: &str)
    -> Result<(), GdsError>;
}

pub(super) struct NearestCoordinateConverter<'a> {
    database_units: f64,
    cell_name: &'a str,
    element_index: usize,
    element_type: &'a str,
}

impl<'a> NearestCoordinateConverter<'a> {
    #[inline]
    pub(super) fn new(database_units: f64, element_type: &'a str) -> Result<Self, GdsError> {
        validate_database_units(database_units)?;
        Ok(Self::new_validated(
            database_units,
            "<standalone>",
            0,
            element_type,
        ))
    }

    #[inline]
    pub(super) const fn new_validated(
        database_units: f64,
        cell_name: &'a str,
        element_index: usize,
        element_type: &'a str,
    ) -> Self {
        Self {
            database_units,
            cell_name,
            element_index,
            element_type,
        }
    }

    #[cold]
    fn error(&self, field: &(impl fmt::Display + ?Sized), message: &str) -> GdsError {
        GdsError::ValidationError {
            message: format!(
                "Cell '{}', element {} ({}), field '{field}': {message}",
                self.cell_name, self.element_index, self.element_type
            ),
        }
    }
}

impl CoordinateConversion for NearestCoordinateConverter<'_> {
    #[inline]
    fn convert_unit(&mut self, value: Unit, field: impl fmt::Display) -> Result<i32, GdsError> {
        if let Unit::Integer(integer) = value
            && integer.units == self.database_units
        {
            return Ok(integer.value);
        }
        let physical_value = self.physical_value(value, &field)?;
        self.convert_physical(physical_value, field)
    }

    #[inline]
    fn physical_value(
        &self,
        value: Unit,
        field: &(impl fmt::Display + ?Sized),
    ) -> Result<f64, GdsError> {
        if !value.units().is_finite() || value.units() <= 0.0 {
            return Err(self.error(
                field,
                &format!(
                    "source units must be finite and positive, got {}",
                    value.units()
                ),
            ));
        }
        let physical_value = value.absolute_value();
        if !physical_value.is_finite() {
            return Err(self.error(
                field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        Ok(physical_value)
    }

    #[inline]
    fn convert_physical(
        &mut self,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        if !physical_value.is_finite() {
            return Err(self.error(
                &field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        let scaled = physical_value / self.database_units;
        if !scaled.is_finite() {
            return Err(self.error(
                &field,
                &format!(
                    "physical value {physical_value} cannot be represented using database unit {}",
                    self.database_units
                ),
            ));
        }
        let rounded = scaled.round();
        if rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
            return Err(self.error(
                &field,
                &format!(
                    "physical value {physical_value} rounds to {rounded}, outside the GDS i32 range"
                ),
            ));
        }
        Ok(rounded as i32)
    }

    #[inline]
    fn checked_array_endpoint(
        &mut self,
        origin: i32,
        spacing: i32,
        count: u32,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        if !physical_value.is_finite() {
            return Err(self.error(
                &field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        let database_value = i64::from(origin) + i64::from(spacing) * i64::from(count);
        i32::try_from(database_value).map_err(|_| {
            self.error(
                &field,
                &format!("derived database value {database_value} is outside the GDS i32 range"),
            )
        })
    }

    #[inline]
    fn validate_finite(&self, value: f64, field: &str) -> Result<(), GdsError> {
        if !value.is_finite() {
            return Err(self.error(field, &format!("value {value} is not finite")));
        }
        Ok(())
    }

    #[inline]
    fn validate_positive(&self, value: f64, field: &str) -> Result<(), GdsError> {
        self.validate_finite(value, field)?;
        if value <= 0.0 {
            return Err(self.error(field, &format!("value {value} must be positive")));
        }
        Ok(())
    }

    #[inline]
    fn encode_gds_real8(&self, value: f64, field: &str) -> Result<[u8; 8], GdsError> {
        eight_byte_real(value).map_err(|error| {
            self.error(
                field,
                &format!("value {value} cannot be encoded as GDS REAL8: {error}"),
            )
        })
    }

    #[inline]
    fn validate_closed(
        &self,
        first: [i32; 2],
        last: [i32; 2],
        field: &str,
    ) -> Result<(), GdsError> {
        if first != last {
            return Err(self.error(
                field,
                &format!(
                    "shape is not closed after database-unit conversion: first point {first:?}, last point {last:?}"
                ),
            ));
        }
        Ok(())
    }
}

impl<'a> CoordinateConverter<'a> {
    pub(super) fn new(
        database_units: f64,
        policy: GdsRoundingPolicy,
        cell_name: &'a str,
        element_index: usize,
        element_type: &'a str,
    ) -> Result<Self, GdsError> {
        validate_database_units(database_units)?;
        Ok(Self {
            database_units,
            policy,
            cell_name,
            element_index,
            element_type,
            field_prefix: None,
            collect_report: true,
            report: GdsConversionReport::default(),
        })
    }

    pub(super) fn with_field_prefix(mut self, field_prefix: Option<String>) -> Self {
        self.field_prefix = field_prefix;
        self
    }

    pub(super) const fn without_report(mut self) -> Self {
        self.collect_report = false;
        self
    }

    pub(super) fn convert_unit(
        &mut self,
        value: Unit,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        if let Unit::Integer(integer) = value
            && integer.units == self.database_units
        {
            return Ok(integer.value);
        }
        if !value.units().is_finite() || value.units() <= 0.0 {
            return Err(self.error(
                &field,
                &format!(
                    "source units must be finite and positive, got {}",
                    value.units()
                ),
            ));
        }
        let physical_value = value.absolute_value();
        if !physical_value.is_finite() {
            return Err(self.error(
                &field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        self.convert_physical(physical_value, field)
    }

    pub(super) fn physical_value(
        &self,
        value: Unit,
        field: &(impl fmt::Display + ?Sized),
    ) -> Result<f64, GdsError> {
        if !value.units().is_finite() || value.units() <= 0.0 {
            return Err(self.error(
                field,
                &format!(
                    "source units must be finite and positive, got {}",
                    value.units()
                ),
            ));
        }
        let physical_value = value.absolute_value();
        if !physical_value.is_finite() {
            return Err(self.error(
                field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        Ok(physical_value)
    }

    pub(super) fn convert_physical(
        &mut self,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        if !physical_value.is_finite() {
            return Err(self.error(
                &field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }

        let scaled = physical_value / self.database_units;
        if !scaled.is_finite() {
            return Err(self.error(
                &field,
                &format!(
                    "physical value {physical_value} cannot be represented using database unit {}",
                    self.database_units
                ),
            ));
        }

        let nearest = scaled.round();
        let rounded = match self.policy {
            GdsRoundingPolicy::Nearest => nearest,
            policy => {
                let snapped = if within_one_ulp(scaled, nearest) {
                    nearest
                } else {
                    scaled
                };
                match policy {
                    GdsRoundingPolicy::Floor => snapped.floor(),
                    GdsRoundingPolicy::Ceil => snapped.ceil(),
                    GdsRoundingPolicy::ErrorIfOffGrid => {
                        if snapped != nearest {
                            return Err(self.error(
                                &field,
                                &format!(
                                    "physical value {physical_value} is off the {} database-unit grid",
                                    self.database_units
                                ),
                            ));
                        }
                        nearest
                    }
                    GdsRoundingPolicy::Nearest => nearest,
                }
            }
        };

        if rounded < f64::from(i32::MIN) || rounded > f64::from(i32::MAX) {
            return Err(self.error(
                &field,
                &format!(
                    "physical value {physical_value} rounds to {rounded}, outside the GDS i32 range"
                ),
            ));
        }

        let database_value = rounded as i32;
        self.record_quantization(physical_value, database_value, field);
        Ok(database_value)
    }

    pub(super) fn checked_array_endpoint(
        &mut self,
        origin: i32,
        spacing: i32,
        count: u32,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        if !physical_value.is_finite() {
            return Err(self.error(
                &field,
                &format!("physical value {physical_value} is not finite"),
            ));
        }
        let database_value = i64::from(origin) + i64::from(spacing) * i64::from(count);
        let Ok(database_value) = i32::try_from(database_value) else {
            return Err(self.error(
                &field,
                &format!("derived database value {database_value} is outside the GDS i32 range"),
            ));
        };
        self.record_quantization(physical_value, database_value, field);
        Ok(database_value)
    }

    fn record_quantization(
        &mut self,
        physical_value: f64,
        database_value: i32,
        field: impl fmt::Display,
    ) {
        if !self.collect_report {
            return;
        }
        let physical_error =
            (physical_value - f64::from(database_value) * self.database_units).abs();
        let scaled = physical_value / self.database_units;
        if physical_error > 0.0 && !within_one_ulp(scaled, f64::from(database_value)) {
            self.report.max_physical_quantization_error = self
                .report
                .max_physical_quantization_error
                .max(physical_error);
            let representative_field = if let Some(prefix) = &self.field_prefix {
                format!("{prefix}.{field}")
            } else {
                field.to_string()
            };
            if let Some(location) = self.report.affected_locations.first_mut() {
                location.affected_field_count += 1;
                if physical_error > location.physical_error {
                    location.representative_field = representative_field;
                    location.physical_value = physical_value;
                    location.database_value = database_value;
                    location.physical_error = physical_error;
                }
            } else {
                self.report.affected_locations.push(GdsQuantization {
                    cell_name: self.cell_name.to_string(),
                    element_index: self.element_index,
                    element_type: self.element_type.to_string(),
                    representative_field,
                    affected_field_count: 1,
                    physical_value,
                    database_value,
                    physical_error,
                });
            }
        }
    }

    pub(super) fn finish(self) -> GdsConversionReport {
        self.report
    }

    pub(super) fn validate_finite(&self, value: f64, field: &str) -> Result<(), GdsError> {
        if !value.is_finite() {
            return Err(self.error(field, &format!("value {value} is not finite")));
        }
        Ok(())
    }

    pub(super) fn validate_positive(&self, value: f64, field: &str) -> Result<(), GdsError> {
        self.validate_finite(value, field)?;
        if value <= 0.0 {
            return Err(self.error(field, &format!("value {value} must be positive")));
        }
        Ok(())
    }

    pub(super) fn encode_gds_real8(&self, value: f64, field: &str) -> Result<[u8; 8], GdsError> {
        eight_byte_real(value).map_err(|error| {
            self.error(
                field,
                &format!("value {value} cannot be encoded as GDS REAL8: {error}"),
            )
        })
    }

    pub(super) fn validate_closed(
        &self,
        first: [i32; 2],
        last: [i32; 2],
        field: &str,
    ) -> Result<(), GdsError> {
        if first != last {
            return Err(self.error(
                field,
                &format!(
                    "shape is not closed after database-unit conversion: first point {first:?}, last point {last:?}"
                ),
            ));
        }
        Ok(())
    }

    fn error(&self, field: &(impl fmt::Display + ?Sized), message: &str) -> GdsError {
        let field = if let Some(prefix) = &self.field_prefix {
            format!("{prefix}.{field}")
        } else {
            field.to_string()
        };
        GdsError::ValidationError {
            message: format!(
                "Cell '{}', element {} ({}), field '{field}': {message}",
                self.cell_name, self.element_index, self.element_type
            ),
        }
    }
}

impl CoordinateConversion for CoordinateConverter<'_> {
    fn convert_unit(&mut self, value: Unit, field: impl fmt::Display) -> Result<i32, GdsError> {
        CoordinateConverter::convert_unit(self, value, field)
    }

    fn physical_value(
        &self,
        value: Unit,
        field: &(impl fmt::Display + ?Sized),
    ) -> Result<f64, GdsError> {
        CoordinateConverter::physical_value(self, value, field)
    }

    fn convert_physical(
        &mut self,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        CoordinateConverter::convert_physical(self, physical_value, field)
    }

    fn checked_array_endpoint(
        &mut self,
        origin: i32,
        spacing: i32,
        count: u32,
        physical_value: f64,
        field: impl fmt::Display,
    ) -> Result<i32, GdsError> {
        CoordinateConverter::checked_array_endpoint(
            self,
            origin,
            spacing,
            count,
            physical_value,
            field,
        )
    }

    fn validate_finite(&self, value: f64, field: &str) -> Result<(), GdsError> {
        CoordinateConverter::validate_finite(self, value, field)
    }

    fn validate_positive(&self, value: f64, field: &str) -> Result<(), GdsError> {
        CoordinateConverter::validate_positive(self, value, field)
    }

    fn encode_gds_real8(&self, value: f64, field: &str) -> Result<[u8; 8], GdsError> {
        CoordinateConverter::encode_gds_real8(self, value, field)
    }

    fn validate_closed(
        &self,
        first: [i32; 2],
        last: [i32; 2],
        field: &str,
    ) -> Result<(), GdsError> {
        CoordinateConverter::validate_closed(self, first, last, field)
    }
}

fn within_one_ulp(left: f64, right: f64) -> bool {
    fn ordered_bits(value: f64) -> u64 {
        let bits = value.to_bits();
        if value.is_sign_negative() {
            !bits
        } else {
            bits | (1_u64 << 63)
        }
    }

    ordered_bits(left).abs_diff(ordered_bits(right)) <= 1
}

pub(super) fn validate_database_units(database_units: f64) -> Result<(), GdsError> {
    if !database_units.is_finite() || database_units <= 0.0 {
        return Err(GdsError::ValidationError {
            message: format!("Database units must be finite and positive, got {database_units}"),
        });
    }
    Ok(())
}
