use std::fmt;

use linc::contract::LinkAnalysisFingerprint;
use parc::contract::{SourceFingerprint, TargetFingerprint};

use crate::{GeneratedFileSet, GenerationDiagnostic, RustLinkPlan, ValidatedRustProjection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenerationFingerprint(pub(crate) [u8; 32]);

impl GenerationFingerprint {
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for GenerationFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("gprojection1_")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Exact upstream identity ledger for one generated bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationManifest {
    source_fingerprint: SourceFingerprint,
    target_fingerprint: TargetFingerprint,
    evidence_fingerprint: LinkAnalysisFingerprint,
    generation_fingerprint: GenerationFingerprint,
}

impl GenerationManifest {
    pub(crate) const fn new(
        source_fingerprint: SourceFingerprint,
        target_fingerprint: TargetFingerprint,
        evidence_fingerprint: LinkAnalysisFingerprint,
        generation_fingerprint: GenerationFingerprint,
    ) -> Self {
        Self {
            source_fingerprint,
            target_fingerprint,
            evidence_fingerprint,
            generation_fingerprint,
        }
    }

    pub const fn source_fingerprint(&self) -> SourceFingerprint {
        self.source_fingerprint
    }

    pub const fn target_fingerprint(&self) -> TargetFingerprint {
        self.target_fingerprint
    }

    pub const fn evidence_fingerprint(&self) -> LinkAnalysisFingerprint {
        self.evidence_fingerprint
    }

    pub const fn generation_fingerprint(&self) -> GenerationFingerprint {
        self.generation_fingerprint
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationBundle {
    projection: ValidatedRustProjection,
    files: GeneratedFileSet,
    link_plan: RustLinkPlan,
    manifest: GenerationManifest,
    diagnostics: Vec<GenerationDiagnostic>,
}

impl GenerationBundle {
    pub(crate) fn new(
        projection: ValidatedRustProjection,
        files: GeneratedFileSet,
        link_plan: RustLinkPlan,
        manifest: GenerationManifest,
        diagnostics: Vec<GenerationDiagnostic>,
    ) -> Self {
        Self {
            projection,
            files,
            link_plan,
            manifest,
            diagnostics,
        }
    }

    pub fn projection(&self) -> &ValidatedRustProjection {
        &self.projection
    }

    pub fn files(&self) -> &GeneratedFileSet {
        &self.files
    }

    pub fn link_plan(&self) -> &RustLinkPlan {
        &self.link_plan
    }

    pub const fn manifest(&self) -> &GenerationManifest {
        &self.manifest
    }

    pub fn diagnostics(&self) -> &[GenerationDiagnostic] {
        &self.diagnostics
    }
}
