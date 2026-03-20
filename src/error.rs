use std::fmt;

/// Crate-wide error type for `gec`.
#[derive(Debug)]
pub enum GecError {
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
pub type GecResult<T> = Result<T, GecError>;

impl fmt::Display for GecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GecError::EmptyInput => write!(f, "input contains no usable declarations"),
            GecError::InvalidConfig { reason } => {
                write!(f, "invalid configuration: {reason}")
            }
            GecError::Io(e) => write!(f, "I/O error: {e}"),
            GecError::Serialization(msg) => write!(f, "serialization error: {msg}"),
        }
    }
}

impl std::error::Error for GecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GecError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GecError {
    fn from(e: std::io::Error) -> Self {
        GecError::Io(e)
    }
}

impl From<serde_json::Error> for GecError {
    fn from(e: serde_json::Error) -> Self {
        GecError::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_empty_input() {
        let e = GecError::EmptyInput;
        assert_eq!(e.to_string(), "input contains no usable declarations");
    }

    #[test]
    fn display_invalid_config() {
        let e = GecError::InvalidConfig {
            reason: "bad value".into(),
        };
        assert!(e.to_string().contains("bad value"));
    }

    #[test]
    fn display_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let e = GecError::Io(io);
        assert!(e.to_string().contains("gone"));
    }

    #[test]
    fn display_serialization() {
        let e = GecError::Serialization("bad json".into());
        assert!(e.to_string().contains("bad json"));
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e: GecError = io.into();
        assert!(matches!(e, GecError::Io(_)));
    }

    #[test]
    fn error_source_io() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = GecError::Io(io);
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn error_source_none_for_others() {
        let e = GecError::EmptyInput;
        assert!(std::error::Error::source(&e).is_none());
    }
}
