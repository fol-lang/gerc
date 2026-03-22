# Architecture

## What GERC Owns

`gec` owns Rust lowering, projection, emission, and generated build output for
this toolchain layer.

It does not own:

- source parsing
- binary inspection
- upstream artifact translation inside `src/**`

## Module layout

| Module | Purpose |
|---|---|
| `intake` | Consumes GERC-owned source and evidence input models |
| `typemap` | Maps GERC-owned C-like types to Rust FFI types (`RustType`) |
| `gate` | Safety gating — accepts or rejects each declaration |
| `lower` | Lowers accepted input items into Rust projection IR |
| `ir` | Internal Rust projection IR: `RustProjection`, `RustItem`, `RustType` |
| `emit` | Renders the IR into deterministic Rust source code |
| `crategen` | Writes a full Cargo-compatible crate directory to disk |
| `linkgen` | Lowers native link surfaces into `build.rs` and `rustc` directives |
| `contract` | Top-level `generate()` entry point and JSON output contract |
| `consumer` | Generic downstream-consumer contract and metadata sidecar |
| `config` | Generation configuration (`GecConfig`) |
| `output` | Generation output container (`GecOutput`, diagnostics) |
| `error` | Crate error types (`GecError`, `GecResult`) |

`gec` is the only Rust emitter in this pipeline layer. If older Rust-emission
logic still exists elsewhere, the intended end state is to move the useful
behavior here and delete the duplicate path.

At the crate root, `gec` now exposes four top-level API families without
module-qualified imports for routine use:

- generation and crate emission
- staged intake and evidence attachment
- JSON metadata and projection contracts
- consumer inspection helpers and metadata sidecars

## Data flow

```text
GecInput (GERC-owned source + optional evidence)
    │
    ├── gate::gate_package()  →  Vec<GateDecision> + diagnostics
    │       │
    │       └── filter: only Accept items pass through
    │
    ├── lower::lower_package()  →  RustProjection + diagnostics
    │
    ├── lower_link_surface()  →  Vec<RustLinkRequirement>
    │
    └── GecOutput { projection, diagnostics }
            │
            ├── emit_source()  →  Rust source string
            ├── emit_crate()   →  Cargo crate on disk
            └── build_sidecar() → JSON metadata for consumers
```

This is an internal `gec` data flow. It is not permission for `gec/src/**` to
import upstream crate types. Upstream artifacts must be translated outside the
library boundary and then handed to `gec` in `gec`'s own input model.

## Artifact boundary

`gec` consumes its own source/evidence model and emits its own generation
artifacts.

The boundary rule is:

- `gec/src/**` must not depend on `parc` or `linc`
- tests/examples/external harnesses may translate upstream artifacts into `gec`
  input
- generated Rust/build outputs are the downstream-facing product

## Key types

GERC should not import `parc` or `linc` in library code.
If another package's artifact needs to be consumed, the translation belongs in
tests/examples or an external harness.

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
