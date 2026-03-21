# API Reference

## API tiers

`gec` organizes its public API into two tiers:

- **Tier 1 (stable)**: `generate()`, `generate_from_source()`, `GecConfig`, `GecInput`, `GecOutput`,
  `GecOutputMeta`, `SCHEMA_VERSION`
- **Tier 2 (public but less stable)**: individual modules (`lower`, `gate`,
  `emit`, `typemap`, `linkgen`, `crategen`, `consumer`)

## Primary workflow

```rust
use gec::{generate_from_source, GecConfig, GecInput, generate};

// 1a. Build input from a linc BindingPackage
let input = GecInput::from_package(pkg);

// 1b. Or generate directly from a linc SourcePackage
let output = generate_from_source(source, &GecConfig::new("mylib_sys")).unwrap();

// 2. Configure generation for the explicit-input path
let config = GecConfig::new("mylib_sys");

// 3. Run the pipeline
let output = generate(&input, &config).unwrap();

// 4. Use the output
let source = gec::emit::emit_source(&output.projection);
```

`GecInput` also exposes explicit JSON constructors:

- `GecInput::from_binding_json(...)`
- `GecInput::from_source_json(...)`

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
