# Intake Contract

## Primary input

`gerc` is source-first, but the important ownership rule is:

- `gerc` library code owns its own input types
- `gerc` library code must not depend on `parc` or `linc`
- translation from PARC or LINC artifacts belongs outside `gerc/src/**`

`GercInput` is therefore the crate-owned intake boundary.

## Optional enrichment

Additional evidence forms can optionally be attached.

### Link analysis artifact

The preferred binary/link evidence contract derived from `linc`.

When present, `gerc` reads:

- resolved link-plan data
- declared link surface
- attached validation evidence

without forcing source declarations to flow through `linc` as the only path.

### Validation report artifact

Declaration-vs-artifact validation evidence. When present, `gerc` uses
validation findings to drive safety gating decisions.

Attached validation evidence is not advisory. `gerc` rejects functions and
variables that are missing validation matches or that only have unusable
matches such as ABI mismatches, duplicate providers, hidden providers,
decoration mismatches, or wrong-kind matches.

Representation evidence is also treated conservatively. If a record or enum
already carries a representation block but upstream left required fields unset
such as record size, record alignment, or enum underlying size, `gerc` rejects
that declaration instead of guessing.

### Resolved link-plan artifact

Resolved native link requirements. When present, `gerc` uses the resolved
plan (with concrete artifact paths and search directories) instead of source
or analysis-declared raw link surfaces.

## Building a `GercInput`

```rust
use gerc::intake::{GercInput, SourcePackage};

// Preferred source-package intake
let input = GercInput::from_source_package(SourcePackage::default());

// Optional explicit enrichment (builder pattern)
let input = GercInput::from_source_package(SourcePackage::default())
    .with_validation(report)
    .with_link_plan(plan);

let input = GercInput::from_source_json(source_json).unwrap();
```

## Normalization

`GercInput::normalize()` is called automatically during `generate()`. It:

- Deduplicates function declarations by name
- Aligns provenance markers
- Is idempotent (safe to call multiple times)

## What gerc does NOT accept

`gerc` does not accept raw C source code or header files directly. Source
extraction belongs in `parc`, and link/binary evidence belongs in `linc`.

The library also should not grow direct PARC/LINC dependency paths in `src/`.
That would violate the intended boundary.
