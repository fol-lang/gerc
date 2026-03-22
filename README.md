# GERC (`gerc` today)

`gerc` is the current crate name for `GERC`, the Rust generation layer in the
`PARC -> LINC -> GERC` pipeline.

It consumes `gerc`'s own source/evidence intake contracts and produces
deterministic Rust FFI output: a projected Rust IR, emitted Rust source, and
optionally a Cargo-compatible crate bundle with `build.rs`.

Architecturally:

- `gerc/src/**` must not depend on `parc` or `linc`
- `gerc` owns its own intake model, projection model, and emitted artifacts
- translation from PARC or LINC artifacts belongs only in tests, examples, or external harnesses
- there is no shared ABI crate and no compatibility layer for discarded pipeline shapes

In the intended architecture:

- `parc` owns source meaning
- `linc` owns link and binary meaning
- `gerc` owns Rust lowering and emitted build metadata

## Responsibilities

- source-first intake from `gerc::SourcePackage`
- optional `gerc::LinkAnalysisPackage`, `ValidationReport`, and `ResolvedLinkPlan` evidence
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
    -> GERC (`gerc` crate today)
    -> generated Rust bindings crate
```

That diagram is a responsibility diagram, not a library-dependency diagram.
`gerc` may be fed source and evidence that originated upstream, but the
translation into `gerc`'s own intake types must stay outside `gerc/src/**`.

## Status

This implementation plan is now complete at the crate level. The remaining
name mismatch is packaging: the crate is still published and imported as
`gerc`, while the architecture and emitted artifacts now identify the role as
`GERC`.

## Preferred usage

```rust
use gerc::{emit_crate, emit_source, generate_from_source, GercConfig, OutputMode, OverwritePolicy};
use gerc::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

let mut source = SourcePackage::default();
source.declarations.push(SourceDeclaration::Function(SourceFunction {
    name: "init".into(),
    parameters: vec![],
    return_type: SourceType::Int,
    variadic: false,
    source_offset: None,
}));

let config = GercConfig::new("mylib_sys");
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

The crate root now re-exports the main API families:

- generation and emission: `generate`, `generate_from_source`, `emit_source`,
  `emit_type`, `emit_crate`, `emit_build_rs`, `OutputMode`,
  `OverwritePolicy`, `CrateManifest`, `EmittedCrate`
- staged intake and evidence attachment: `GercInput`, `EvidenceInputs`,
  `GateDecision`
- JSON contracts: `output_meta`, `meta_to_json`, `meta_from_json`,
  `projection_to_json`, `projection_from_json`, `GercOutputMeta`,
  `SCHEMA_VERSION`
- consumer inspection and sidecars: `GercConsumer`, `ConsumerReport`,
  `ConsumerFinding`, `FindingKind`, `PassthroughConsumer`, `FolConsumer`,
  `build_sidecar`, `sidecar_to_json`, `sidecar_from_json`,
  `extern_function_names`, `record_names`, `type_alias_names`

Generated crate manifests and `src/lib.rs` markers now use `GERC` naming for
the emitter identity.

## Artifact Boundary

`gerc` consumes its own input contracts and emits its own output artifacts.
The practical split is:

- `parc` owns source artifacts
- `linc` owns evidence artifacts
- tests/examples/harnesses may translate those artifacts into `gerc` input
- `gerc` emits Rust source, build files, sidecars, and rustc argument files

That keeps generation independent from upstream crate internals.

## Tested Scope

The suite currently exercises:

- source-only generation from translated source artifacts
- evidence-aware generation from translated source and evidence artifacts
- deterministic projection and emitted-source behavior
- emitted Cargo crate output and raw `rustc` argument output
- larger fixture corpora including hostile surfaces and real-library examples

The tests are the main statement of supported behavior.

## Build And Test

```sh
make build
make test
```

## Validation-gated generation

When a `ValidationReport` is attached, `gerc` only projects declarations with
usable validation evidence. Missing matches, ABI mismatches, duplicate
providers, hidden providers, decoration mismatches, and wrong-kind matches are
rejected instead of being projected speculatively.

`gerc` also treats partially-populated representation evidence conservatively.
If a record or enum carries representation metadata but is missing critical
fields like record size, record alignment, or enum underlying size, generation
rejects that item instead of inventing layout facts.

Generated Rust source now includes source-comment notes for preserved
provenance and other non-routine projection notes, so downstream readers can
see where declarations came from and why items were only partially supported.

## Intentional output differences

`gerc` is the canonical Rust emitter in this pipeline now, so some differences
from older `linc` Rust output are intentional:

- opaque handles stay named Rust types (`pub struct NAME { _opaque: [u8; 0] }`)
  instead of being erased into comments
- enums emit as `#[repr(...)] pub enum` items instead of typedef-plus-const
  blocks
- function-pointer aliases emit as `Option<unsafe extern "C" fn(...)>` instead
  of bare function-pointer aliases

These are current `gerc` decisions, not compatibility regressions.

## Split-pipeline coverage

The integration suite now covers realistic vendored split-pipeline paths:

- `parc -> gerc` source-only generation for zlib and libpng fixtures
- `parc -> linc -> gerc` generation with declared link surfaces
- `parc -> linc -> gerc` generation with resolved link-plan evidence

Explicit staged inspection still exists, but it now lives under module paths:

- `gerc::gate::gate_package(...)`
- `gerc::lower::lower_package(...)`

That keeps coverage anchored in upstream-produced fixtures instead of
synthetic-only packages.
