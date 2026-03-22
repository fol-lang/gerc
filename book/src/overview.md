# Overview

`GERC` currently ships as the `gerc` crate and sits between `parc`, `linc`, and downstream Rust tooling in the
following pipeline:

```text
PARC         (source parsing and extraction)
    → LINC   (link, validation, and ABI evidence)
    → GERC   (Rust projection, code generation)
    → generated Rust bindings crate
    → fol-interloop-rust  (optional downstream consumer)
    → fol-visible surface
```

Read this chapter as the workflow summary:

1. `gerc` receives its own input model
2. any upstream artifact translation happens outside `gerc/src/**`
3. `gerc` gates, lowers, and emits deterministic Rust/build artifacts

## What happens inside GERC

1. **Intake** — `gerc` receives its own source/evidence input model via
   `GercInput`. Tests/examples or external harnesses may translate PARC and LINC
   artifacts into that model.

2. **Safety gating** — Each declaration is checked against generation rules.
   Items that cannot be safely represented in Rust (bitfields, anonymous
   records, incomplete types) are rejected with diagnostics.

3. **Lowering** — Accepted declarations are lowered from GERC-owned source
   types into `gerc`'s internal Rust projection IR (`RustProjection`).

4. **Type mapping** — C types (`int`, `void*`, pointers, arrays, function
   pointers, etc.) are mapped to Rust FFI-safe equivalents.

5. **Emission** — The IR is rendered into deterministic Rust source code.
   Items are emitted in a stable order: constants, type aliases, enums,
   records, then an `extern "C"` block for functions and statics.

6. **Crate generation** — Optionally, `gerc` writes a full Cargo-compatible
   crate directory: `Cargo.toml`, `src/lib.rs`, and a `build.rs` with native
   link metadata.

## Design principles

- **Deterministic** — The same input always produces the same output.
- **Conservative** — Prefer refusing generation over emitting unsound code.
- **Library-first** — `gerc` is a Rust library, not a CLI tool.
- **Generic** — No `fol`-specific assumptions in the core crate.
- **Artifact-boundary integration** — cross-package composition belongs outside
  `gerc/src/**`.
- **No crate-level back-compat burden** — dead pipeline shapes should be
  deleted, not carried indefinitely.
