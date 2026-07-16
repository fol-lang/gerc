//! Strict Rust FFI projection over checked PARC source and LINC ABI evidence.
//!
//! GERC has one production intake path: [`GenerationRequest`]. It borrows a
//! complete PARC closure and a checked LINC analysis. Output domain values are
//! immutable and have no serde implementation; GERC H1 deliberately defines
//! no output transport family and therefore has no unchecked JSON decode path.

mod bundle;
mod emit;
mod error;
mod files;
mod fingerprint;
mod generate;
mod link;
mod projection;
mod request;

pub use bundle::{GenerationBundle, GenerationFingerprint, GenerationManifest};
pub use error::{
    GenerationContext, GenerationDiagnostic, GenerationDiagnosticCode, GenerationError,
    GenerationErrorCode, GenerationResult,
};
pub use files::{GeneratedFile, GeneratedFileSet, GeneratedPath};
pub use generate::generate;
pub use link::{RustLinkArtifact, RustLinkArtifactKind, RustLinkAtom, RustLinkPlan};
pub use projection::{
    NativeSymbolBinding, RustAbi, RustEnum, RustEnumVariant, RustField, RustFunction, RustItem,
    RustMacro, RustName, RustParameter, RustRecord, RustRecordKind, RustScalar, RustType,
    RustTypeAlias, RustTypeKind, RustVariable, SourceDeclarationMetadata, ValidatedRustProjection,
};
pub use request::{GenerationRequest, ItemSelection};

/// Frozen typed generation-domain identity. This is an in-memory contract,
/// not a JSON schema identifier.
pub const GENERATION_SCHEMA_ID: &str = "follang.gerc.generation";
pub const GENERATION_SCHEMA_VERSION: u16 = 1;
pub const GENERATION_ALGORITHM_ID: &str = "gerc-rust-ffi-projection-v1";
pub const GENERATOR_IDENTITY: &str = concat!("follang-gerc/", env!("CARGO_PKG_VERSION"));

pub(crate) use emit::render_projection;
