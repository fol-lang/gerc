# Intake Contract

## Primary input

`gec` is source-first. The required input is always a `linc::SourcePackage`.

Compatibility and enrichment paths sit around that required input:

- `linc::SourcePackage` via `GecInput::from_source_package(...)`
- legacy `linc::BindingPackage` via `GecInput::from_package(...)`
  or `input_from_binding_package(...)`

The source-package path is the preferred starting point when the caller already
has `parc` or another frontend contract. It keeps `gec` on the split-pipeline
path and lets the crate consume source meaning directly.

## Optional enrichment

Three additional `linc` evidence forms can optionally be attached:

### `LinkAnalysisPackage`

The preferred evidence contract from `linc`.

When present, `gec` reads:

- resolved link-plan data
- declared link surface
- attached validation evidence

without forcing source declarations to flow through `linc` as the only path.

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
plan (with concrete artifact paths and search directories) instead of source
or analysis-declared raw link surfaces.

## Building a `GecInput`

```rust
use gec::intake::{input_from_binding_package, GecInput};

use linc::{LinkAnalysisPackage, SourcePackage};

// Preferred source-package intake
let input = GecInput::from_source_package(SourcePackage::default());

// Preferred evidence attachment
let input = GecInput::from_source_package(SourcePackage::default())
    .with_analysis(LinkAnalysisPackage::default());

// Transitional legacy-binding intake
let input = input_from_binding_package(pkg);

// Optional explicit enrichment (builder pattern)
let input = GecInput::from_package(pkg)
    .with_validation(report)
    .with_link_plan(plan);

// Explicit JSON entrypoints
let input = GecInput::from_binding_json(binding_json).unwrap();
let input = GecInput::from_source_json(source_json).unwrap();
```

## Normalization

`GecInput::normalize()` is called automatically during `generate()`. It:

- Deduplicates function declarations by name
- Aligns provenance markers
- Is idempotent (safe to call multiple times)

## What gec does NOT accept

`gec` does not accept raw C source code or header files directly. Source
extraction belongs in `parc`, and transitional raw-header scanning belongs in
`linc::HeaderConfig`.
