# API Reference

## API tiers

`gec` organizes its public API into two tiers:

- **Tier 1 (stable)**: `generate()`, `generate_from_source()`, `GecConfig`, `GecInput`, `GecOutput`,
  `GecOutputMeta`, `SCHEMA_VERSION`
- **Tier 2 (public but less stable)**: individual modules (`lower`, `gate`,
  `emit`, `typemap`, `linkgen`, `crategen`, `consumer`)

The crate root also re-exports the common emission entrypoints so downstream
code does not need to import `emit` or `crategen` directly for routine use:
`emit_source`, `emit_type`, `emit_crate`, `emit_build_rs`, `emit_rustc_args`,
`emit_rustc_link_args`, `OutputMode`, `OverwritePolicy`, `CrateManifest`, and
`EmittedCrate`.

For explicit staged workflows, the crate root also re-exports:

- `EvidenceInputs` for optional analysis/validation/link-plan attachment
- `gate_package` and `GateDecision` for explicit gating inspection
- `lower_package` for explicit projection lowering
- `output_meta`, `meta_to_json`, `meta_from_json`, `projection_to_json`, and
  `projection_from_json` for JSON contracts
- `GecConsumer`, `ConsumerReport`, `ConsumerFinding`, `FindingKind`,
  `PassthroughConsumer`, `FolConsumer`, `build_sidecar`, `sidecar_to_json`,
  `sidecar_from_json`, `extern_function_names`, `record_names`, and
  `type_alias_names` for downstream inspection

## Primary workflow

`generate_from_source()` is the preferred entrypoint when the caller already
has a `linc::SourcePackage`. Use `GecInput` directly when attaching optional
`linc` evidence in parallel with source.

```rust
use gec::{
    emit_crate, emit_rustc_args, emit_source, generate, generate_from_source, GecConfig,
    GecInput, OutputMode, OverwritePolicy,
};

// 1a. Preferred: build input from a SourcePackage
let input = GecInput::from_source_package(source.clone()).with_analysis(analysis);

// 1b. Or generate directly from a SourcePackage
let output = generate_from_source(source, &GecConfig::new("mylib_sys")).unwrap();

// 3. Configure generation for the explicit-input path
let config = GecConfig::new("mylib_sys");

// 4. Run the pipeline
let output = generate(&input, &config).unwrap();

// 5. Use the output
let source = emit_source(&output.projection);
let rustc_args = emit_rustc_args(&output.projection);
let emitted = emit_crate(
    &output.projection,
    &config,
    std::path::Path::new("/tmp/mylib_sys"),
    OutputMode::Crate,
    OverwritePolicy::Overwrite,
).unwrap();
```

`GecInput` exposes an explicit source-contract JSON constructor:

- `GecInput::from_source_json(...)`

When validation evidence is attached, `generate()` filters out declarations
that fail validation gating instead of emitting speculative Rust bindings.

The generated Rust source also includes comment-level projection notes for
preserved provenance and other non-routine lowering outcomes. This keeps
upstream context visible without changing the Rust API surface.

The emitted crate path now supports both:

- Cargo build-script rendering via `build.rs`
- direct `rustc` argument rendering via `rustc-link-args.txt` and
  `emit_rustc_args(...)`

## Integration coverage

The current suite exercises realistic split-pipeline paths, not only
hand-constructed packages:

- vendored `parc -> gec` source-only generation for zlib and libpng fixtures
- vendored `parc -> linc -> gec` generation with declared link surfaces
- vendored `parc -> linc -> gec` generation with resolved link-plan evidence

## Configuration

`GecConfig` controls what gets generated:

| Field | Default | Description |
|---|---|---|
| `crate_name` | `"gec_output"` | Name for the generated crate |
| `crate_version` | `"0.1.0"` | Version string |
| `emit_functions` | `true` | Emit `extern "C"` function declarations |
| `emit_records` | `true` | Emit `#[repr(C)]` struct/union types |
| `emit_enums` | `true` | Emit enum types |
| `emit_type_aliases` | `true` | Emit type aliases |
| `emit_variables` | `true` | Emit static variable declarations |
| `emit_constants` | `true` | Emit constant definitions |
| `emit_build_script` | `true` | Emit `build.rs` with link metadata |

## Error handling

All fallible operations return `GecResult<T>`, which is
`Result<T, GecError>`.

```rust
pub enum GecError {
    EmptyInput,
    InvalidConfig { reason: String },
    Io(std::io::Error),
    Serialization(String),
}
```

## JSON contract

Output metadata can be serialized for downstream tooling:

```rust
use gec::contract::{output_meta, meta_to_json};

let meta = output_meta(&config, &output);
let json = meta_to_json(&meta).unwrap();
```

The JSON includes `schema_version` for forward compatibility.
`meta_from_json()` rejects metadata with a schema version newer than the
current build supports.

## Consumer contract

Downstream tools implement the `GecConsumer` trait:

```rust
use gec::consumer::{GecConsumer, ConsumerReport};
use gec::ir::RustProjection;

struct MyConsumer;

impl GecConsumer for MyConsumer {
    fn inspect(&self, proj: &RustProjection) -> ConsumerReport {
        // inspect the projection...
        ConsumerReport::default()
    }
}
```

A `MetadataSidecar` (JSON) can be generated alongside the crate for tools
that don't need to parse Rust source:

```rust
use gec::consumer::{build_sidecar, sidecar_to_json};

let sidecar = build_sidecar("mylib_sys", &output.projection);
let json = sidecar_to_json(&sidecar).unwrap();
```

Generated crate metadata and crate-level `src/lib.rs` markers identify the
emitter as `GERC`.
