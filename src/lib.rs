//! `gec` — Rust projection layer on top of `bic`.
//!
//! This crate consumes `bic` analysis results and generates Rust FFI-facing
//! code from C declarations.  It is designed as a library-first crate: the
//! primary entry point is the public Rust API, not a CLI binary.
//!
//! # What `gec` does
//!
//! - Consumes `bic` `BindingPackage` (and optional validation/link-plan data).
//! - Maps C types to Rust FFI-safe types.
//! - Lowers `bic` declarations into an internal Rust projection IR.
//! - Emits deterministic Rust source files and Cargo-compatible crate structures.
//! - Emits `build.rs` / native link metadata for Cargo/rustc.
//!
//! # What `gec` does **not** do
//!
//! - Parse C source — that is `bic`'s job.
//! - Duplicate ABI/layout/link discovery logic already in `bic`.
//! - Own `fol`-specific surface generation.
//! - Own final deployment/runtime loader policy.
//!
//! # Intended pipeline
//!
//! ```text
//! C headers / native artifacts
//!     -> bic (analysis)
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
//! 2. Feed it a [`GecInput`] (wrapping a `bic::BindingPackage` plus optional extras).
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

pub use contract::{generate, GecOutputMeta, SCHEMA_VERSION};

pub use config::GecConfig;
pub use error::{GecError, GecResult};
pub use intake::GecInput;
pub use output::GecOutput;
