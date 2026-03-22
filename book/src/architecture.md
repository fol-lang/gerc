# Architecture

## What GERC Owns

`gerc` owns Rust lowering, projection, emission, and generated build output
for this toolchain layer.

It does not own:

- source parsing
- binary inspection
- upstream artifact translation inside `src/**`

## Module Layout

| Module | Purpose |
|---|---|
| `intake` | Crate-owned source and evidence input models |
| `c` | Crate-owned C-side binding model and native evidence types |
| `typemap` | Maps C-like types to Rust FFI types |
| `gate` | Safety gating for each declaration |
| `lower` | Lowers accepted items into `RustProjection` |
| `ir` | Rust projection IR and supporting types |
| `emit` | Renders the IR into deterministic Rust source |
| `linkgen` | Lowers native link surfaces into Cargo and `rustc` directives |
| `crategen` | Writes crate directories and source bundles to disk |
| `contract` | Top-level generation entry point and JSON output contract |
| `consumer` | Generic downstream-consumer contract and metadata sidecar |
| `config` | Generation configuration (`GercConfig`) |
| `output` | Generation output container (`GercOutput`) |
| `error` | Crate error types (`GercError`, `GercResult`) |

`gerc` is the only Rust emitter in this pipeline layer. If older Rust-emission
logic still exists elsewhere, the end state is to move the useful behavior here
and delete the duplicate path.

At the crate root, `gerc` exposes four top-level API families:

- generation and crate emission
- staged intake and evidence attachment
- JSON metadata and projection contracts
- consumer inspection helpers and metadata sidecars

## Data Flow

```text
GercInput
    │
    ├── gate::gate_package()  -> Vec<GateDecision> + diagnostics
    │       │
    │       └── filter: only accepted items pass through
    │
    ├── lower::lower_package() -> RustProjection + diagnostics
    │
    ├── linkgen lowering       -> native link requirements
    │
    └── GercOutput { projection, diagnostics }
            │
            ├── emit_source()   -> Rust source string
            ├── emit_crate()    -> Cargo crate or source bundle on disk
            └── build_sidecar() -> JSON metadata for consumers
```

This is an internal `gerc` data flow. It is not permission for `gerc/src/**`
to import upstream crate types. Upstream artifacts must be translated outside
the library boundary and then handed to `gerc` in `gerc`'s own input model.

## Artifact Boundary

`gerc` consumes its own source/evidence model and emits its own generation
artifacts.

The boundary rule is:

- `gerc/src/**` must not depend on `parc` or `linc`
- tests/examples/external harnesses may translate upstream artifacts into `gerc` input
- generated Rust/build outputs are the downstream-facing product

## Key Types

`gerc` should not import `parc` or `linc` in library code.
If another package's artifact needs to be consumed, the translation belongs in
tests/examples or an external harness.

### `GercInput`

The primary input container. It wraps:

- a required `SourcePackage`
- optional `EvidenceInputs`

`EvidenceInputs` can carry `LinkAnalysisPackage`, `ValidationReport`, and
`ResolvedLinkPlan` data.

### `RustProjection`

The central IR type. It contains:

- `items: Vec<RustItem>` - functions, records, enums, type aliases, constants,
  statics, unsupported markers
- `modules: Vec<RustModule>` - optional module organization
- `link_requirements: Vec<RustLinkRequirement>` - native link metadata
- `notes: Vec<ProjectionNote>` - provenance and diagnostic notes

### `GateDecision`

Either `Accept` or `Reject(reason)`. Rejected items produce diagnostics but no
Rust code.
