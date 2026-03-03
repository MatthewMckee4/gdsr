use std::fmt;
use std::io;

/// Error type for GDSII operations.
#[derive(Debug)]
pub enum GdsError {
    /// An I/O error occurred during reading or writing.
    Io(io::Error),
    /// The data is invalid or malformed.
    InvalidData { message: String },
    /// A validation check failed.
    ValidationError { message: String },
}

impl fmt::Display for GdsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::InvalidData { message } => write!(f, "Invalid data: {message}"),
            Self::ValidationError { message } => write!(f, "Validation error: {message}"),
        }
    }
}

impl std::error::Error for GdsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::InvalidData { .. } | Self::ValidationError { .. } => None,
        }
    }
}

impl From<io::Error> for GdsError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gds_error_display_io() {
        let err = GdsError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        insta::assert_snapshot!(err.to_string(), @"I/O error: file not found");
    }

    #[test]
    fn test_gds_error_display_invalid_data() {
        let err = GdsError::InvalidData {
            message: "bad record".to_string(),
        };
        insta::assert_snapshot!(err.to_string(), @"Invalid data: bad record");
    }

    #[test]
    fn test_gds_error_display_validation() {
        let err = GdsError::ValidationError {
            message: "too few points".to_string(),
        };
        insta::assert_snapshot!(err.to_string(), @"Validation error: too few points");
    }

    #[test]
    fn test_gds_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let gds_err: GdsError = io_err.into();
        assert!(matches!(gds_err, GdsError::Io(_)));
    }

    #[test]
    fn test_gds_error_source() {
        use std::error::Error;

        let io_err = GdsError::Io(io::Error::other("inner"));
        assert!(io_err.source().is_some());

        let invalid = GdsError::InvalidData {
            message: "msg".to_string(),
        };
        assert!(invalid.source().is_none());

        let validation = GdsError::ValidationError {
            message: "msg".to_string(),
        };
        assert!(validation.source().is_none());
    }
}
