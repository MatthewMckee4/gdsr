use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike};

use crate::error::GdsError;

const MAX_GDS_YEAR: i32 = 9_999;

/// The two date/time values stored in a GDS `BGNLIB` or `BGNSTR` record.
///
/// For `BGNLIB`, [`Self::first`] is the last modification time and
/// [`Self::second`] is the last access time. For `BGNSTR`, they are the
/// creation time and last modification time, respectively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GdsTimestamps {
    first: Option<NaiveDateTime>,
    second: Option<NaiveDateTime>,
}

impl GdsTimestamps {
    /// The reproducible all-zero timestamp sentinel supported by GDS writers.
    pub const ZERO: Self = Self {
        first: None,
        second: None,
    };

    /// Creates a validated pair of GDS timestamps.
    pub fn try_new(first: NaiveDateTime, second: NaiveDateTime) -> Result<Self, GdsError> {
        validate_datetime(first)?;
        validate_datetime(second)?;
        Ok(Self {
            first: Some(first),
            second: Some(second),
        })
    }

    /// Returns the first date/time value.
    ///
    /// This is the last modification time for a library and the creation time
    /// for a structure.
    pub const fn first(&self) -> Option<&NaiveDateTime> {
        self.first.as_ref()
    }

    /// Returns the second date/time value.
    ///
    /// This is the last access time for a library and the last modification
    /// time for a structure.
    pub const fn second(&self) -> Option<&NaiveDateTime> {
        self.second.as_ref()
    }

    pub(crate) fn current() -> Self {
        let now = Local::now().naive_utc();
        Self {
            first: Some(now),
            second: Some(now),
        }
    }

    pub(crate) fn from_record(values: &[i16], record: &str) -> Result<Self, GdsError> {
        if values.len() != 12 {
            return Err(invalid_record_timestamps(record, values));
        }
        let (first, second) = values.split_at(6);

        let first = parse_datetime(first);
        let second = parse_datetime(second);
        match (first, second) {
            (Some(first), Some(second)) => {
                Self::try_new(first, second).map_err(|_| invalid_record_timestamps(record, values))
            }
            (None, None) if values.iter().all(|value| *value == 0) => Ok(Self::ZERO),
            _ => Err(invalid_record_timestamps(record, values)),
        }
    }

    pub(crate) fn to_record(self) -> [u16; 12] {
        let mut values = [0; 12];
        if let (Some(first), Some(second)) = (self.first, self.second) {
            values[..6].copy_from_slice(&datetime_fields(first));
            values[6..].copy_from_slice(&datetime_fields(second));
        }
        values
    }
}

/// Controls which timestamps are written to GDS `BGNLIB` and `BGNSTR` records.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GdsTimestampPolicy {
    /// Write one current time consistently across the whole output.
    #[default]
    Current,
    /// Write timestamps stored on the library and each structure.
    Preserve,
    /// Write all date/time fields as zero for reproducible output.
    Zero,
}

fn validate_datetime(value: NaiveDateTime) -> Result<(), GdsError> {
    if !(1..=MAX_GDS_YEAR).contains(&value.year()) {
        return Err(GdsError::ValidationError {
            message: format!(
                "GDS timestamp year {} is outside the supported range 1..={MAX_GDS_YEAR}",
                value.year()
            ),
        });
    }
    if value.nanosecond() != 0 {
        return Err(GdsError::ValidationError {
            message: "GDS timestamps do not support subsecond precision".to_string(),
        });
    }

    Ok(())
}

fn parse_datetime(values: &[i16]) -> Option<NaiveDateTime> {
    let [year, month, day, hour, minute, second] = *values else {
        return None;
    };
    if !(1..=MAX_GDS_YEAR).contains(&i32::from(year)) {
        return None;
    }

    NaiveDate::from_ymd_opt(
        i32::from(year),
        u32::try_from(month).ok()?,
        u32::try_from(day).ok()?,
    )?
    .and_hms_opt(
        u32::try_from(hour).ok()?,
        u32::try_from(minute).ok()?,
        u32::try_from(second).ok()?,
    )
}

fn datetime_fields(value: NaiveDateTime) -> [u16; 6] {
    [
        value.year() as u16,
        value.month() as u16,
        value.day() as u16,
        value.hour() as u16,
        value.minute() as u16,
        value.second() as u16,
    ]
}

fn invalid_record_timestamps(record: &str, values: &[i16]) -> GdsError {
    GdsError::InvalidData {
        message: format!("Invalid {record} timestamps: {values:?}"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn rejects_dates_that_cannot_be_stored_in_gds() {
        let date = NaiveDate::from_ymd_opt(10_000, 1, 1)
            .expect("test date should be valid in chrono")
            .and_hms_opt(0, 0, 0)
            .expect("test time should be valid");

        let error =
            GdsTimestamps::try_new(date, date).expect_err("five-digit years should be rejected");

        assert!(matches!(error, GdsError::ValidationError { .. }));
        assert!(error.to_string().contains("10000"));
    }

    #[test]
    fn rejects_subsecond_precision() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1)
            .expect("test date should be valid in chrono")
            .and_hms_nano_opt(0, 0, 0, 1)
            .expect("test time should be valid");

        let error =
            GdsTimestamps::try_new(date, date).expect_err("subsecond precision should be rejected");

        assert!(matches!(error, GdsError::ValidationError { .. }));
        assert!(error.to_string().contains("subsecond"));
    }

    #[test]
    fn zero_is_a_distinct_timestamp_sentinel() {
        assert_eq!(GdsTimestamps::ZERO.first(), None);
        assert_eq!(GdsTimestamps::ZERO.second(), None);
        assert_eq!(GdsTimestamps::ZERO.to_record(), [0; 12]);
    }
}
