use crate::ir::RustProjection;

/// Result of a `gec` generation run.
///
/// Contains the projected Rust IR and any diagnostics produced during
/// the projection process.
#[derive(Debug, Clone, Default)]
pub struct GecOutput {
    /// The projected Rust items.
    pub projection: RustProjection,
    /// Diagnostics produced during projection (warnings, skips, etc.).
    pub diagnostics: Vec<GecDiagnostic>,
}

/// Severity level for generation diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GecSeverity {
    Warning,
    Info,
}

/// One diagnostic produced during projection.
#[derive(Debug, Clone)]
pub struct GecDiagnostic {
    pub severity: GecSeverity,
    pub message: String,
    pub item_name: Option<String>,
}

impl GecOutput {
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

    pub fn warnings(&self) -> impl Iterator<Item = &GecDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == GecSeverity::Warning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_default_is_empty() {
        let out = GecOutput::new();
        assert!(out.is_empty());
        assert_eq!(out.item_count(), 0);
        assert!(!out.has_diagnostics());
    }

    #[test]
    fn warnings_filter() {
        let out = GecOutput {
            projection: RustProjection::default(),
            diagnostics: vec![
                GecDiagnostic {
                    severity: GecSeverity::Info,
                    message: "info".into(),
                    item_name: None,
                },
                GecDiagnostic {
                    severity: GecSeverity::Warning,
                    message: "warn".into(),
                    item_name: Some("foo".into()),
                },
            ],
        };
        assert_eq!(out.warnings().count(), 1);
    }
}
