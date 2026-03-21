//! `gec` — Rust projection layer in the `PARC -> LINC -> GERC` pipeline.
//!
//! This crate consumes `linc` analysis results and generates Rust FFI-facing
//! code from C declarations.  It is designed as a library-first crate: the
//! primary entry point is the public Rust API, not a CLI binary.
//!
//! # What `gec` does
//!
//! - Consumes `linc` `BindingPackage` (and optional validation/link-plan data).
//! - Maps C types to Rust FFI-safe types.
//! - Lowers `linc` declarations into an internal Rust projection IR.
//! - Emits deterministic Rust source files and Cargo-compatible crate structures.
//! - Emits `build.rs` / native link metadata for Cargo/rustc.
//!
//! # What `gec` does **not** do
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
//!     -> gec (Rust projection)
//!     -> generated Rust bindings crate
//!     -> fol-interloop-rust (optional downstream)
//! ```
//!
//! # Public API overview
//!
//! The primary workflow is:
//!
//! 1. Build a [`GecConfig`] describing what to generate.
//! 2. Feed it a [`GecInput`] (wrapping a `linc::BindingPackage` plus optional extras).
//! 3. Receive a [`GecOutput`] containing the projected Rust IR and generation results.
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

pub use contract::{generate, generate_from_source, GecOutputMeta, SCHEMA_VERSION};

pub use config::GecConfig;
pub use crategen::{
    emit_build_rs, emit_crate, normalize_crate_name, CrateManifest, EmittedCrate, OutputMode,
    OverwritePolicy,
};
pub use emit::{emit_source, emit_type};
pub use error::{GecError, GecResult};
pub use intake::GecInput;
pub use output::GecOutput;
