//! `GERC` implementation crate, currently published as `gec`.
//!
//! This crate consumes `linc` analysis results and generates Rust FFI-facing
//! code from C declarations.  It is designed as a library-first crate: the
//! primary entry point is the public Rust API, not a CLI binary.
//!
//! # What `GERC` does
//!
//! - Consumes source contracts directly and can adapt legacy `linc`
//!   `BindingPackage` input when migration requires it.
//! - Maps C types to Rust FFI-safe types.
//! - Lowers `linc` declarations into an internal Rust projection IR.
//! - Emits deterministic Rust source files and Cargo-compatible crate structures.
//! - Emits `build.rs` / native link metadata for Cargo/rustc.
//!
//! # What `GERC` does **not** do
//!
//! - Parse C source — that is `parc`'s job.
//! - Duplicate ABI/layout/link discovery logic already in `linc`.
//! - Own `fol`-specific surface generation.
//! - Own final deployment/runtime loader policy.
//!
//! # Intended pipeline
//!
//! ```text
//! PARC (source contracts)
//!     -> LINC (link and evidence contracts)
//!     -> GERC (`gec` crate today)
//!     -> generated Rust bindings crate
//!     -> fol-interloop-rust (optional downstream)
//! ```
//!
//! # Public API overview
//!
//! The primary workflow is:
//!
//! 1. Build a [`GecConfig`] describing what to generate.
//! 2. Feed it a [`GecInput`] (wrapping a source contract plus optional evidence).
//! 3. Receive a [`GecOutput`] containing the projected Rust IR and generation results.
//!
//! The crate root exposes four routine API families:
//!
//! - generation and crate emission
//! - staged intake, gating, and lowering
//! - JSON metadata and projection contracts
//! - consumer inspection helpers and metadata sidecars
//!
//! All public types live in the crate root or in clearly named submodules.

pub mod config;
pub mod consumer;
pub mod contract;
#[cfg(test)]
mod corpus;
pub mod crategen;
pub mod emit;
pub mod error;
pub mod gate;
pub mod intake;
pub mod ir;
pub mod linkgen;
pub mod lower;
pub mod output;
pub mod typemap;

pub use contract::{
    generate, generate_from_source, meta_from_json, meta_to_json, output_meta,
    projection_from_json, projection_to_json, GecOutputMeta, SCHEMA_VERSION,
};

pub use config::GecConfig;
pub use consumer::{
    build_sidecar, extern_function_names, record_names, sidecar_from_json, sidecar_to_json,
    type_alias_names, ConsumerFinding, ConsumerReport, FindingKind, FolConsumer, GecConsumer,
    MetadataSidecar, PassthroughConsumer, SidecarItem, SidecarItemKind,
};
pub use crategen::{
    emit_build_rs, emit_crate, normalize_crate_name, CrateManifest, EmittedCrate, OutputMode,
    OverwritePolicy,
};
pub use emit::{emit_source, emit_type};
pub use error::{GecError, GecResult};
pub use gate::{gate_package, GateDecision};
pub use intake::{
    input_from_binding_json, input_from_binding_package, source_from_binding_package,
    EvidenceInputs, GecInput,
};
pub use linkgen::emit_rustc_link_args;
pub use lower::lower_package;
pub use output::GecOutput;
