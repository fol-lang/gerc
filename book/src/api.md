# API Reference

## First principle

`gerc` is the Rust lowering and emission layer of the toolchain.

The safest downstream posture is:

1. prefer crate-root APIs first
2. provide `gerc`'s own source/evidence inputs directly
3. keep upstream artifact translation outside `gerc/src/**`
4. treat emitted Rust/build artifacts as the product boundary

## API tiers

`gerc` organizes its public API into two tiers:

- **Tier 1 (stable)**: `generate()`, `generate_from_source()`, `GercConfig`, `GercInput`, `GercOutput`,
  `GercOutputMeta`, `SCHEMA_VERSION`
- **Tier 2 (public but less stable)**: individual modules (`lower`, `gate`,
  `emit`, `typemap`, `linkgen`, `crategen`, `consumer`)

The crate root also re-exports the common emission entrypoints so downstream
code does not need to import `emit` or `crategen` directly for routine use:
`emit_source`, `emit_type`, `emit_crate`, `emit_build_rs`, `emit_rustc_args`,
`emit_rustc_link_args`, `OutputMode`, `OverwritePolicy`, `CrateManifest`, and
`EmittedCrate`.

For explicit staged workflows, the crate root also re-exports:

- `EvidenceInputs` for optional analysis/validation/link-plan attachment
- `GateDecision` for explicit gating inspection results
- `output_meta`, `meta_to_json`, `meta_from_json`, `projection_to_json`, and
  `projection_from_json` for JSON contracts
- `GercConsumer`, `ConsumerReport`, `ConsumerFinding`, `FindingKind`,
  `PassthroughConsumer`, `FolConsumer`, `build_sidecar`, `sidecar_to_json`,
  `sidecar_from_json`, `extern_function_names`, `record_names`, and
  `type_alias_names` for downstream inspection

For staged inspection, import the modules explicitly:

- `gerc::gate::gate_package(...)`
- `gerc::lower::lower_package(...)`

## Preferred public surface

These are the main consumer-facing entrypoints:

- `generate()` and `generate_from_source()`
- `GercInput`, `GercConfig`, and `GercOutput`
- `emit_source()`, `emit_crate()`, `emit_build_rs()`, and `emit_rustc_args()`
- JSON metadata/projection helpers
- consumer-sidecar helpers

## Primary workflow

`generate_from_source()` is the preferred entrypoint when the caller already
has a `gerc::SourcePackage`. Use `GercInput` directly when attaching optional
translated evidence in parallel with source.

```rust
use gerc::{
    emit_crate, emit_rustc_args, emit_source, generate, generate_from_source, GercConfig,
    GercInput, OutputMode, OverwritePolicy,
};

// 1a. Preferred: build input from a SourcePackage
let input = GercInput::from_source_package(source.clone()).with_analysis(analysis);

// 1b. Or generate directly from a SourcePackage
let output = generate_from_source(source, &GercConfig::new("mylib_sys")).unwrap();

// 3. Configure generation for the explicit-input path
let config = GercConfig::new("mylib_sys");

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

`GercInput` exposes an explicit source-contract JSON constructor:

- `GercInput::from_source_json(...)`

When validation evidence is attached, `generate()` filters out declarations
that fail validation gating instead of emitting speculative Rust bindings.

The generated Rust source also includes comment-level projection notes for
preserved provenance and other non-routine lowering outcomes. This keeps
upstream context visible without changing the Rust API surface.

The emitted crate path now supports both:

- Cargo build-script rendering via `build.rs`
- direct `rustc` argument rendering via `rustc-link-args.txt` and
  `emit_rustc_args(...)`

## Downstream posture

If you are integrating `gerc` into another tool, prefer:

1. root-level generation APIs
2. explicit `GercInput` construction when evidence is available
3. emitted Rust/build outputs rather than internal lowering modules
4. tests/examples/harnesses for any `parc` or `linc` artifact translation

## Integration coverage

The current suite exercises realistic split-pipeline paths, not only
hand-constructed packages:

- vendored `parc -> gerc` source-only generation for zlib and libpng fixtures
- vendored `parc -> linc -> gerc` generation with declared link surfaces
- vendored `parc -> linc -> gerc` generation with resolved link-plan evidence

Those paths are proved in tests and examples. They are not library-level crate
dependencies.

## Configuration

`GercConfig` controls what gets generated:

| Field | Default | Description |
|---|---|---|
| `crate_name` | `"gerc_output"` | Name for the generated crate |
| `crate_version` | `"0.1.0"` | Version string |
| `emit_functions` | `true` | Emit `extern "C"` function declarations |
| `emit_records` | `true` | Emit `#[repr(C)]` struct/union types |
| `emit_enums` | `true` | Emit enum types |
| `emit_type_aliases` | `true` | Emit type aliases |
| `emit_variables` | `true` | Emit static variable declarations |
| `emit_constants` | `true` | Emit constant definitions |
| `emit_build_script` | `true` | Emit `build.rs` with link metadata |

## Explicit non-goals

The current contract does not promise:

- parsing or preprocessing C
- binary inspection inside `gerc`
- automatic invention of missing ABI facts
- library-level dependencies on upstream pipeline crates

## Error handling

All fallible operations return `GercResult<T>`, which is
`Result<T, GercError>`.

```rust
pub enum GercError {
    EmptyInput,
    InvalidConfig { reason: String },
    Io(std::io::Error),
    Serialization(String),
}
```

## JSON contract

Output metadata can be serialized for downstream tooling:

```rust
use gerc::contract::{output_meta, meta_to_json};

let meta = output_meta(&config, &output);
let json = meta_to_json(&meta).unwrap();
```

The JSON includes `schema_version` as an artifact-shape gate.
`meta_from_json()` rejects metadata with a schema version newer than the
current build supports rather than guessing.

## Consumer contract

Downstream tools implement the `GercConsumer` trait:

```rust
use gerc::consumer::{GercConsumer, ConsumerReport};
use gerc::ir::RustProjection;

struct MyConsumer;

impl GercConsumer for MyConsumer {
    fn inspect(&self, proj: &RustProjection) -> ConsumerReport {
        // inspect the projection...
        ConsumerReport::default()
    }
}
```

A `MetadataSidecar` (JSON) can be generated alongside the crate for tools
that don't need to parse Rust source:

```rust
use gerc::consumer::{build_sidecar, sidecar_to_json};

let sidecar = build_sidecar("mylib_sys", &output.projection);
let json = sidecar_to_json(&sidecar).unwrap();
```

Generated crate metadata and crate-level `src/lib.rs` markers identify the
emitter as `GERC`.
