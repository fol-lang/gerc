use crate::ir::RustProjection;

/// Result of a `gerc` generation run.
///
/// Contains the projected Rust IR and any diagnostics produced during
/// the projection process.
#[derive(Debug, Clone, Default)]
pub struct GercOutput {
    /// The projected Rust items.
    pub projection: RustProjection,
    /// Diagnostics produced during projection (warnings, skips, etc.).
    pub diagnostics: Vec<GercDiagnostic>,
}

/// Severity level for generation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GercSeverity {
    Warning,
    Info,
}

/// One diagnostic produced during projection.
#[derive(Debug, Clone)]
pub struct GercDiagnostic {
    pub severity: GercSeverity,
    pub message: String,
    pub item_name: Option<String>,
}

impl GercOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.projection.is_empty()
    }

    pub fn item_count(&self) -> usize {
        self.projection.len()
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn warnings(&self) -> impl Iterator<Item = &GercDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == GercSeverity::Warning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_default_is_empty() {
        let out = GercOutput::new();
        assert!(out.is_empty());
        assert_eq!(out.item_count(), 0);
        assert!(!out.has_diagnostics());
    }

    #[test]
    fn warnings_filter() {
        let out = GercOutput {
            projection: RustProjection::default(),
            diagnostics: vec![
                GercDiagnostic {
                    severity: GercSeverity::Info,
                    message: "info".into(),
                    item_name: None,
                },
                GercDiagnostic {
                    severity: GercSeverity::Warning,
                    message: "warn".into(),
                    item_name: Some("foo".into()),
                },
            ],
        };
        assert_eq!(out.warnings().count(), 1);
    }
}
