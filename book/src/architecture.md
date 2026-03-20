# Architecture

## Module layout

| Module | Purpose |
|---|---|
| `intake` | Consumes `bic::BindingPackage` and optional enrichment data |
| `typemap` | Maps C types (`bic::BindingType`) to Rust FFI types (`RustType`) |
| `gate` | Safety gating — accepts or rejects each declaration |
| `lower` | Lowers accepted `bic` items into Rust projection IR |
| `ir` | Internal Rust projection IR: `RustProjection`, `RustItem`, `RustType` |
| `emit` | Renders the IR into deterministic Rust source code |
| `crategen` | Writes a full Cargo-compatible crate directory to disk |
| `linkgen` | Lowers `bic` link surfaces into `build.rs` link directives |
| `contract` | Top-level `generate()` entry point and JSON output contract |
| `consumer` | Generic downstream-consumer contract and metadata sidecar |
| `config` | Generation configuration (`GecConfig`) |
| `output` | Generation output container (`GecOutput`, diagnostics) |
| `error` | Crate error types (`GecError`, `GecResult`) |

## Data flow

```text
GecInput (bic::BindingPackage + optional extras)
    │
    ├── gate_package()  →  Vec<GateDecision> + diagnostics
    │       │
    │       └── filter: only Accept items pass through
    │
    ├── lower_package()  →  RustProjection + diagnostics
    │
    ├── lower_link_surface()  →  Vec<RustLinkRequirement>
    │
    └── GecOutput { projection, diagnostics }
            │
            ├── emit_source()  →  Rust source string
            ├── emit_crate()   →  Cargo crate on disk
            └── build_sidecar() → JSON metadata for consumers
```

## Key types

### `RustProjection`

The central IR type. Contains:
- `items: Vec<RustItem>` — functions, records, enums, type aliases, constants,
  statics, unsupported markers
- `modules: Vec<RustModule>` — optional module organization
- `link_requirements: Vec<RustLinkRequirement>` — native link metadata
- `notes: Vec<ProjectionNote>` — provenance and diagnostic notes

### `RustType`

Enum covering all Rust FFI-safe types:
- Primitives: `Void`, `Bool`, `CChar`, `CInt`, `CUInt`, `CLong`, etc.
- Compound: `Pointer`, `Array`, `FnPointer`
- References: `Named(String)` for typedefs/records/enums
- Special: `OpaquePtr` for `void*`, `Unknown` for unmappable types

### `GateDecision`

Either `Accept` or `Reject(reason)`. Items that are rejected produce
diagnostics but no Rust code.
