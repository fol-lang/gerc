# gec

`gec` is the Rust generation layer in the `PARC -> LINC -> GERC` pipeline.

It consumes `linc` source/link/evidence contracts and produces deterministic
Rust FFI output: a projected Rust IR, emitted Rust source, and optionally a
Cargo-compatible crate bundle with `build.rs`.

## Responsibilities

- source-first intake from `linc::SourcePackage`
- explicit `linc::BindingPackage` intake plus optional validation and link-plan evidence
- conservative gating of unsupported or under-evidenced declarations
- lowering into Rust projection IR
- deterministic Rust source emission
- emitted crate and build-script generation
- ownership of Rust FFI emission for this pipeline layer

## Non-responsibilities

- parsing C source or preprocessing headers
- inspecting native binaries directly
- inventing ABI facts that should come from upstream contracts
- downstream runtime policy or high-level wrapper generation

## Pipeline

```text
PARC (source contracts)
    -> LINC (link and evidence contracts)
    -> gec (Rust projection and crate emission)
    -> generated Rust bindings crate
```

## Status

This crate is in active migration toward `GERC`.

Current direction:

- old `bic`-centric framing is being deleted
- the crate is aligning to split `parc` + `linc` intake
- backward compatibility is intentionally not a goal
- legacy Rust emission behavior worth preserving is being rehomed here from `linc`

## Preferred usage

```rust
use gec::{emit_crate, emit_source, generate_from_source, GecConfig, OutputMode, OverwritePolicy};
use linc::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

let mut source = SourcePackage::default();
source.declarations.push(SourceDeclaration::Function(SourceFunction {
    name: "init".into(),
    parameters: vec![],
    return_type: SourceType::Int,
    variadic: false,
    source_offset: None,
}));

let config = GecConfig::new("mylib_sys");
let output = generate_from_source(source, &config).unwrap();
let source = emit_source(&output.projection);
let emitted = emit_crate(
    &output.projection,
    &config,
    std::path::Path::new("/tmp/mylib_sys"),
    OutputMode::Crate,
    OverwritePolicy::Clean,
).unwrap();

assert!(source.contains("pub fn init"));
assert!(emitted.root.join("Cargo.toml").exists());
```

The crate root now re-exports the main generation and emission entrypoints:
`generate`, `generate_from_source`, `emit_source`, `emit_crate`,
`emit_build_rs`, `OutputMode`, and `OverwritePolicy`.

For explicit staged workflows, the crate root also re-exports `GecInput`,
`EvidenceInputs`, `gate_package`, `GateDecision`, and `lower_package`.

## Validation-gated generation

When a `ValidationReport` is attached, `gec` only projects declarations with
usable validation evidence. Missing matches, ABI mismatches, duplicate
providers, hidden providers, decoration mismatches, and wrong-kind matches are
rejected instead of being projected speculatively.

`gec` also treats partially-populated representation evidence conservatively.
If a record or enum carries representation metadata but is missing critical
fields like record size, record alignment, or enum underlying size, generation
rejects that item instead of inventing layout facts.

Generated Rust source now includes source-comment notes for preserved
provenance and other non-routine projection notes, so downstream readers can
see where declarations came from and why items were only partially supported.

## Intentional output differences

`gec` is the canonical Rust emitter in this pipeline now, so some differences
from older `linc` Rust output are intentional:

- opaque handles stay named Rust types (`pub struct NAME { _opaque: [u8; 0] }`)
  instead of being erased into comments
- enums emit as `#[repr(...)] pub enum` items instead of typedef-plus-const
  blocks
- function-pointer aliases emit as `Option<unsafe extern "C" fn(...)>` instead
  of bare function-pointer aliases

These are current `gec` decisions, not compatibility regressions.

## Split-pipeline coverage

The integration suite now covers realistic vendored split-pipeline paths:

- `parc -> gec` source-only generation for zlib and libpng fixtures
- `parc -> linc -> gec` generation with declared link surfaces
- `parc -> linc -> gec` generation with resolved link-plan evidence

That keeps coverage anchored in upstream-produced fixtures instead of
synthetic-only packages.
