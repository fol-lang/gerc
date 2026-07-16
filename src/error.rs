use linc::contract::LinkAnalysisFingerprint;
use parc::contract::{
    CallingConvention, DeclarationId, MacroId, OperatingSystem, SourceFingerprint,
    TargetFingerprint,
};
use thiserror::Error;

/// Stable classification for generation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationErrorCode {
    EmptySelection,
    DuplicateSelection,
    SelectionMismatch,
    SourceFingerprintMismatch,
    TargetFingerprintMismatch,
    EvidenceCoverageMismatch,
    MissingDeclaration,
    MissingDeclarationEvidence,
    MissingLayoutEvidence,
    UnsupportedCallingConvention,
    UnsupportedType,
    UnsupportedDeclaration,
    UnsupportedRecordRepresentation,
    LayoutMismatch,
    InvalidEnumRepresentation,
    InvalidIdentifier,
    UnsupportedLinkProjection,
    ProjectionInvariant,
    GeneratedFileInvariant,
    GeneratedSourceParse,
}

impl GenerationErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptySelection => "GERC-E1000",
            Self::DuplicateSelection => "GERC-E1001",
            Self::SelectionMismatch => "GERC-E1002",
            Self::SourceFingerprintMismatch => "GERC-E1100",
            Self::TargetFingerprintMismatch => "GERC-E1101",
            Self::EvidenceCoverageMismatch => "GERC-E1102",
            Self::MissingDeclaration => "GERC-E1103",
            Self::MissingDeclarationEvidence => "GERC-E1104",
            Self::MissingLayoutEvidence => "GERC-E1105",
            Self::UnsupportedCallingConvention => "GERC-E2000",
            Self::UnsupportedType => "GERC-E2001",
            Self::UnsupportedDeclaration => "GERC-E2002",
            Self::UnsupportedRecordRepresentation => "GERC-E2003",
            Self::LayoutMismatch => "GERC-E2100",
            Self::InvalidEnumRepresentation => "GERC-E2101",
            Self::InvalidIdentifier => "GERC-E2200",
            Self::UnsupportedLinkProjection => "GERC-E2300",
            Self::ProjectionInvariant => "GERC-E9000",
            Self::GeneratedFileInvariant => "GERC-E9001",
            Self::GeneratedSourceParse => "GERC-E9002",
        }
    }
}

/// A closed, typed refusal surface. GERC never substitutes an unknown Rust
/// type or guesses missing ABI facts.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GenerationError {
    #[error("{source} [{context}]")]
    Contextual {
        context: GenerationContext,
        #[source]
        source: Box<GenerationError>,
    },
    #[error("generation selection must contain at least one DeclarationId")]
    EmptySelection,
    #[error("generation selection repeats declaration {declaration}")]
    DuplicateSelection { declaration: DeclarationId },
    #[error("generation selection does not match the complete PARC selection")]
    SelectionMismatch,
    #[error("LINC evidence was produced for a different PARC source fingerprint")]
    SourceFingerprintMismatch,
    #[error("LINC evidence was produced for a different target fingerprint")]
    TargetFingerprintMismatch,
    #[error("LINC declaration evidence does not cover the selected transitive PARC closure")]
    EvidenceCoverageMismatch,
    #[error("selected declaration {declaration} is absent from the complete PARC closure")]
    MissingDeclaration { declaration: DeclarationId },
    #[error("declaration {declaration} has no checked LINC declaration evidence")]
    MissingDeclarationEvidence { declaration: DeclarationId },
    #[error("declaration {declaration} has no checked LINC layout evidence")]
    MissingLayoutEvidence { declaration: DeclarationId },
    #[error(
        "calling convention {convention:?} for declaration {declaration} is not supported on {operating_system:?}"
    )]
    UnsupportedCallingConvention {
        declaration: DeclarationId,
        convention: CallingConvention,
        operating_system: OperatingSystem,
    },
    #[error("declaration {declaration} contains an unsupported type at {path}: {reason}")]
    UnsupportedType {
        declaration: DeclarationId,
        path: String,
        reason: &'static str,
    },
    #[error("declaration {declaration} cannot be projected: {reason}")]
    UnsupportedDeclaration {
        declaration: DeclarationId,
        reason: &'static str,
    },
    #[error("record {declaration} has an unsupported representation: {reason}")]
    UnsupportedRecordRepresentation {
        declaration: DeclarationId,
        reason: &'static str,
    },
    #[error("measured layout for declaration {declaration} cannot be represented: {reason}")]
    LayoutMismatch {
        declaration: DeclarationId,
        reason: &'static str,
    },
    #[error("enum {declaration} has an invalid measured representation: {reason}")]
    InvalidEnumRepresentation {
        declaration: DeclarationId,
        reason: &'static str,
    },
    #[error("declaration {declaration} has no usable Rust identifier")]
    InvalidIdentifier { declaration: DeclarationId },
    #[error("ordered link plan has no certified target argument projection: {reason}")]
    UnsupportedLinkProjection { reason: &'static str },
    #[error("validated Rust projection invariant failed: {reason}")]
    ProjectionInvariant { reason: &'static str },
    #[error("generated file-set invariant failed: {reason}")]
    GeneratedFileInvariant { reason: &'static str },
    #[error("generated Rust file {path} failed the production parse postcondition: {message}")]
    GeneratedSourceParse { path: &'static str, message: String },
}

impl GenerationError {
    pub const fn code(&self) -> GenerationErrorCode {
        match self {
            Self::Contextual { source, .. } => source.code(),
            Self::EmptySelection => GenerationErrorCode::EmptySelection,
            Self::DuplicateSelection { .. } => GenerationErrorCode::DuplicateSelection,
            Self::SelectionMismatch => GenerationErrorCode::SelectionMismatch,
            Self::SourceFingerprintMismatch => GenerationErrorCode::SourceFingerprintMismatch,
            Self::TargetFingerprintMismatch => GenerationErrorCode::TargetFingerprintMismatch,
            Self::EvidenceCoverageMismatch => GenerationErrorCode::EvidenceCoverageMismatch,
            Self::MissingDeclaration { .. } => GenerationErrorCode::MissingDeclaration,
            Self::MissingDeclarationEvidence { .. } => {
                GenerationErrorCode::MissingDeclarationEvidence
            }
            Self::MissingLayoutEvidence { .. } => GenerationErrorCode::MissingLayoutEvidence,
            Self::UnsupportedCallingConvention { .. } => {
                GenerationErrorCode::UnsupportedCallingConvention
            }
            Self::UnsupportedType { .. } => GenerationErrorCode::UnsupportedType,
            Self::UnsupportedDeclaration { .. } => GenerationErrorCode::UnsupportedDeclaration,
            Self::UnsupportedRecordRepresentation { .. } => {
                GenerationErrorCode::UnsupportedRecordRepresentation
            }
            Self::LayoutMismatch { .. } => GenerationErrorCode::LayoutMismatch,
            Self::InvalidEnumRepresentation { .. } => {
                GenerationErrorCode::InvalidEnumRepresentation
            }
            Self::InvalidIdentifier { .. } => GenerationErrorCode::InvalidIdentifier,
            Self::UnsupportedLinkProjection { .. } => {
                GenerationErrorCode::UnsupportedLinkProjection
            }
            Self::ProjectionInvariant { .. } => GenerationErrorCode::ProjectionInvariant,
            Self::GeneratedFileInvariant { .. } => GenerationErrorCode::GeneratedFileInvariant,
            Self::GeneratedSourceParse { .. } => GenerationErrorCode::GeneratedSourceParse,
        }
    }

    pub const fn stable_code(&self) -> &'static str {
        self.code().as_str()
    }

    pub const fn context(&self) -> Option<&GenerationContext> {
        match self {
            Self::Contextual { context, .. } => Some(context),
            _ => None,
        }
    }

    pub(crate) fn with_context(self, context: GenerationContext) -> Self {
        match self {
            Self::Contextual { .. } => self,
            source => Self::Contextual {
                context,
                source: Box::new(source),
            },
        }
    }
}

pub type GenerationResult<T> = Result<T, GenerationError>;

/// Non-fatal generation note retained in [`crate::GenerationBundle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationDiagnostic {
    code: GenerationDiagnosticCode,
    context: GenerationContext,
    declaration: Option<DeclarationId>,
    macro_id: Option<MacroId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationDiagnosticCode {
    PreservedMacroNotEmitted,
}

impl GenerationDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreservedMacroNotEmitted => "GERC-N3000",
        }
    }
}

impl GenerationDiagnostic {
    pub(crate) const fn preserved_macro_not_emitted(
        context: GenerationContext,
        macro_id: MacroId,
    ) -> Self {
        Self {
            code: GenerationDiagnosticCode::PreservedMacroNotEmitted,
            context,
            declaration: None,
            macro_id: Some(macro_id),
        }
    }

    pub const fn code(&self) -> GenerationDiagnosticCode {
        self.code
    }

    pub const fn stable_code(&self) -> &'static str {
        self.code.as_str()
    }

    pub const fn context(&self) -> &GenerationContext {
        &self.context
    }

    pub const fn declaration(&self) -> Option<DeclarationId> {
        self.declaration
    }

    pub const fn macro_id(&self) -> Option<MacroId> {
        self.macro_id
    }
}

/// Fingerprint context attached to every generation-phase error and note.
/// Selection-shape errors created before a request exists are the only errors
/// without this context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationContext {
    source_fingerprint: SourceFingerprint,
    target_fingerprint: TargetFingerprint,
    evidence_fingerprint: LinkAnalysisFingerprint,
}

impl GenerationContext {
    pub(crate) const fn new(
        source_fingerprint: SourceFingerprint,
        target_fingerprint: TargetFingerprint,
        evidence_fingerprint: LinkAnalysisFingerprint,
    ) -> Self {
        Self {
            source_fingerprint,
            target_fingerprint,
            evidence_fingerprint,
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
}

impl std::fmt::Display for GenerationContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "source={}, target={}, evidence={}",
            self.source_fingerprint, self.target_fingerprint, self.evidence_fingerprint
        )
    }
}
