# Intake Contract

## Primary input

`gec` accepts two explicit intake forms:

- `linc::BindingPackage` via `GecInput::from_package(...)`
- `linc::SourcePackage` via `GecInput::from_source_package(...)`

The source-package path is the preferred starting point when the caller already
has `parc`/`linc` source contracts. It keeps `gec` on the split-pipeline path
and lets the crate adapt source declarations into bindings internally.

The binding-package path is the richer machine contract and contains:

- **Items** — function declarations, record (struct/union) definitions, enum
  definitions, type aliases, variable declarations, and unsupported markers
- **Diagnostics** — warnings and errors from upstream analysis
- **Macros** — C preprocessor macro definitions (integer literals become Rust
  constants)
- **Layouts** — ABI layout information for records
- **Link surface** — native library requirements, search paths, artifacts

## Optional enrichment

Two additional `linc` outputs can optionally be attached:

### `ValidationReport`

Declaration-vs-artifact validation evidence. When present, `gec` uses
validation findings to drive safety gating decisions.

Attached validation evidence is not advisory. `gec` rejects functions and
variables that are missing validation matches or that only have unusable
matches such as ABI mismatches, duplicate providers, hidden providers,
decoration mismatches, or wrong-kind matches.

Representation evidence is also treated conservatively. If a record or enum
already carries a representation block but upstream left required fields unset
such as record size, record alignment, or enum underlying size, `gec` rejects
that declaration instead of guessing.

### `ResolvedLinkPlan`

Resolved native link requirements. When present, `gec` uses the resolved
plan (with concrete artifact paths and search directories) instead of the
raw link surface from the binding package.

## Building a `GecInput`

```rust
use gec::intake::GecInput;

use linc::SourcePackage;

// Source-package intake
let input = GecInput::from_source_package(SourcePackage::default());

// Binding-package intake
let input = GecInput::from_package(pkg);

// Optional enrichment (builder pattern)
let input = GecInput::from_package(pkg)
    .with_validation(report)
    .with_link_plan(plan);

// Explicit JSON entrypoints
let input = GecInput::from_binding_json(binding_json).unwrap();
let input = GecInput::from_source_json(source_json).unwrap();
```

## Normalization

`GecInput::normalize()` is called automatically during `generate()`. It:

- Deduplicates function declarations by name (last-wins)
- Aligns provenance markers
- Is idempotent (safe to call multiple times)

## What gec does NOT accept

`gec` does not accept raw C source code or header files directly. Source
extraction belongs in `parc`, and transitional raw-header scanning belongs in
`linc::HeaderConfig`.
