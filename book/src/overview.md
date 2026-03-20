# Overview

`gec` sits between `bic` (C analysis) and downstream Rust tooling in the
following pipeline:

```text
C headers / native artifacts
    → bic          (C parsing, ABI extraction, link discovery)
    → gec          (Rust projection, code generation)
    → generated Rust bindings crate
    → fol-interloop-rust  (optional downstream consumer)
    → fol-visible surface
```

## What happens inside gec

1. **Intake** — `gec` receives a `bic::BindingPackage` (plus optional
   validation and link-plan data) via `GecInput`.

2. **Safety gating** — Each declaration is checked against generation rules.
   Items that cannot be safely represented in Rust (bitfields, anonymous
   records, incomplete types) are rejected with diagnostics.

3. **Lowering** — Accepted declarations are lowered from `bic` types into
   `gec`'s internal Rust projection IR (`RustProjection`).

4. **Type mapping** — C types (`int`, `void*`, pointers, arrays, function
   pointers, etc.) are mapped to Rust FFI-safe equivalents.

5. **Emission** — The IR is rendered into deterministic Rust source code.
   Items are emitted in a stable order: constants, type aliases, enums,
   records, then an `extern "C"` block for functions and statics.

6. **Crate generation** — Optionally, `gec` writes a full Cargo-compatible
   crate directory: `Cargo.toml`, `src/lib.rs`, and a `build.rs` with native
   link metadata.

## Design principles

- **Deterministic** — The same input always produces the same output.
- **Conservative** — Prefer refusing generation over emitting unsound code.
- **Library-first** — `gec` is a Rust library, not a CLI tool.
- **Generic** — No `fol`-specific assumptions in the core crate.
