# Overview

`gerc` sits between `parc`, `linc`, and downstream Rust tooling in the
toolchain:

```text
PARC  (source contracts)
  -> LINC  (link and evidence contracts)
  -> GERC  (Rust lowering and emission)
  -> generated Rust bindings crate or source bundle
```

Read this chapter as the workflow summary:

1. `gerc` receives its own input model
2. any upstream artifact translation happens outside `gerc/src/**`
3. `gerc` gates, lowers, and emits deterministic Rust and build artifacts

## What Happens Inside GERC

1. **Intake** - `gerc` receives a `GercInput` wrapping source plus optional
   evidence. Tests/examples or external harnesses may translate PARC and LINC
   artifacts into that model.

2. **Safety gating** - Each declaration is checked against generation rules.
   Items that cannot be safely represented in Rust are rejected with
   diagnostics rather than guessed into existence.

3. **Lowering** - Accepted declarations are lowered from the crate-owned C-side
   model into `RustProjection`.

4. **Type mapping** - C types, pointers, arrays, function pointers, and
   qualifiers are mapped to Rust FFI-safe equivalents.

5. **Emission** - The projection is rendered into deterministic Rust source.
   Items are emitted in a stable order: constants, type aliases, enums,
   records, then `extern "C"` declarations.

6. **Crate generation** - Optionally, `gerc` writes a Cargo-compatible crate
   directory or a source bundle plus `rustc` link arguments.

## Design Principles

- **Deterministic** - the same input always produces the same output
- **Conservative** - refuse generation rather than emit unsound code
- **Library-first** - `gerc` is a Rust library, not a CLI tool
- **Generic** - no `fol`-specific assumptions in the core crate
- **Artifact-boundary integration** - cross-package composition belongs outside
  `gerc/src/**`
- **No back-compat burden** - discarded shapes are deleted, not preserved
