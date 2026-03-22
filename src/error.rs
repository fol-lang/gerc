use std::fmt;

/// Crate-wide error type for `gerc`.
#[derive(Debug)]
pub enum GercError {
    /// The input was empty or contained no usable declarations.
    EmptyInput,
    /// A configuration value was invalid or contradictory.
    InvalidConfig { reason: String },
    /// An I/O failure occurred during output emission.
    Io(std::io::Error),
    /// Serialization or deserialization failed.
    Serialization(String),
}

/// Convenience alias used throughout the crate.
pub type GercResult<T> = Result<T, GercError>;

impl fmt::Display for GercError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GercError::EmptyInput => write!(f, "input contains no usable declarations"),
            GercError::InvalidConfig { reason } => {
                write!(f, "invalid configuration: {reason}")
            }
            GercError::Io(e) => write!(f, "I/O error: {e}"),
            GercError::Serialization(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for GercError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GercError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GercError {
    fn from(e: std::io::Error) -> Self {
        GercError::Io(e)
    }
}

impl From<serde_json::Error> for GercError {
    fn from(e: serde_json::Error) -> Self {
        GercError::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_input() {
        let e = GercError::EmptyInput;
        assert_eq!(e.to_string(), "input contains no usable declarations");
    }

    #[test]
    fn display_invalid_config() {
        let e = GercError::InvalidConfig {
            reason: "bad value".into(),
        };
        assert!(e.to_string().contains("bad value"));
    }

    #[test]
    fn display_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let e = GercError::Io(io);
        assert!(e.to_string().contains("gone"));
    }

    #[test]
    fn display_serialization() {
        let e = GercError::Serialization("bad json".into());
        assert!(e.to_string().contains("bad json"));
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: GercError = io.into();
        assert!(matches!(e, GercError::Io(_)));
    }

    #[test]
    fn error_source_io() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = GercError::Io(io);
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn error_source_none_for_others() {
        let e = GercError::EmptyInput;
        assert!(std::error::Error::source(&e).is_none());
    }
}
