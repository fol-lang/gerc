# Intake Contract

## Primary input

`gec` requires a `bic::BindingPackage` as its primary input. This is the
canonical output of `bic`'s C analysis pipeline and contains:

- **Items** — function declarations, record (struct/union) definitions, enum
  definitions, type aliases, variable declarations, and unsupported markers
- **Diagnostics** — warnings and errors from `bic` analysis
- **Macros** — C preprocessor macro definitions (integer literals become Rust
  constants)
- **Layouts** — ABI layout information for records
- **Link surface** — native library requirements, search paths, artifacts

## Optional enrichment

Two additional `bic` outputs can optionally be attached:

### `ValidationReport`

Declaration-vs-artifact validation evidence. When present, `gec` uses
validation findings to influence safety gating decisions.

### `ResolvedLinkPlan`

Resolved native link requirements. When present, `gec` uses the resolved
plan (with concrete artifact paths and search directories) instead of the
raw link surface from the binding package.

## Building a `GecInput`

```rust
use gec::intake::GecInput;

// Minimal: just a BindingPackage
let input = GecInput::from_package(pkg);

// With optional enrichment (builder pattern)
let input = GecInput::from_package(pkg)
    .with_validation(report)
    .with_link_plan(plan);

// From JSON
let input = GecInput::from_json(json_str).unwrap();
```

## Normalization

`GecInput::normalize()` is called automatically during `generate()`. It:

- Deduplicates function declarations by name (last-wins)
- Aligns provenance markers
- Is idempotent (safe to call multiple times)

## What gec does NOT accept

`gec` does not accept raw C source code or header files. All C parsing is
`bic`'s responsibility. If you have C headers, run them through `bic` first
to obtain a `BindingPackage`.
