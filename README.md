# GERC (`gerc` crate)

`gerc` is the Rust lowering and emission layer in the `parc -> linc -> gerc`
toolchain.

It produces Rust-facing output from translated C-side inputs:

- a `RustProjection`
- emitted Rust source
- emitted crate directories and `build.rs`
- raw `rustc` link arguments
- metadata sidecars for downstream consumers

## What GERC Actually Exposes Today

The crate is source-first, but its public API is not only `generate()` plus a
few helpers.

It currently exposes:

- `GercInput`, `GercConfig`, `GercOutput`, and the root generation helpers
- staged `gate` and `lower` modules
- a large crate-owned C-side model in `gerc::c`
- emission and crate-writing helpers
- consumer/sidecar helpers

The docs need to match that breadth rather than flattening everything into one
perfectly clean layer.

## Responsibilities

- intake of GERC-owned source and evidence contracts
- conservative gating of declarations
- lowering into `RustProjection`
- deterministic Rust source emission
- crate/build output generation
- Cargo and raw `rustc` link metadata rendering

## Non-responsibilities

- parsing or preprocessing C source
- direct native binary inspection
- inventing missing ABI facts
- runtime loader policy
- library-level dependency on `parc` or `linc`

## Boundary

`gerc/src/**` must stay independent from `parc` and `linc`.

Cross-package translation belongs in tests, examples, or external harnesses.
The library consumes its own input model and emits its own artifacts.

## Fastest Working Paths

Source-first generation:

```rust
use gerc::{generate_from_source, emit_source, GercConfig};
use gerc::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

let mut source = SourcePackage::default();
source.declarations.push(SourceDeclaration::Function(SourceFunction {
    name: "demo_init".into(),
    parameters: vec![],
    return_type: SourceType::Int,
    variadic: false,
    source_offset: None,
}));

let output = generate_from_source(source, &GercConfig::new("demo_sys")).unwrap();
println!("{}", emit_source(&output.projection));
```

Staged workflow:

```rust
use gerc::gate::gate_package;
use gerc::lower::lower_package;
```

That staged flow is still public and tested.

## Artifact Boundary

`gerc` owns its own lowering and emission model plus the emitted build-side
artifacts.

The durable boundaries are:

- the crate-owned intake model in `gerc::c`
- emitted Rust source
- emitted crate directories and `build.rs`
- rendered raw `rustc` link-argument files
- metadata sidecars for downstream consumers

Cross-package translation still belongs outside `gerc/src/**`. GERC can be used
in integration tests and harnesses, but its library code is not where upstream
`parc` or `linc` wiring should live.

## Tested Scope

The suite covers:

- source-only generation
- evidence-aware generation
- staged gate/lower usage
- root API re-exports
- emitted crate output
- raw `rustc` argument output
- larger fixture surfaces and artifact-boundary tests
- OpenSSL, libpng, and combined Linux event-loop link-directive generation
- explicit gate, lowering, and pipeline failure matrices

The tests are the best statement of what GERC actually supports.

## Hardening Matrix

The current hardening ladder is easiest to read in four buckets:

- hermetic vendored baselines
  - source-only zlib lowering
  - source-only libpng lowering with conservative rejection
  - emitted crate output on deterministic vendored surfaces
- host-dependent evidence ladders
  - OpenSSL link-directive generation
  - combined Linux event-loop link-directive generation
- failure and conservative-lowering surfaces
  - anonymous-type rejection paths
  - unsupported layout and ABI-sensitive gating
  - source-only degradation when link evidence is absent
  - explicit gate, lowering, and pipeline failure matrices
- determinism anchors
  - source-only zlib projection
  - vendored libpng conservative failure path
  - OpenSSL link directives when available
  - combined Linux event-loop link directives

Read those as the current confidence anchors, not as a claim that every native
surface lowers equally well today.

## Release Gates

`gerc` should only be treated as release-ready when all of these remain green:

- `make build`
- `make test`
- source-only suites
- evidence-aware suites
- emitted-crate output checks
- raw `rustc` argument checks
- at least one OpenSSL-style host-dependent evidence target
- at least one combined Linux/system link-directive target

The current canonical generation surfaces are:

- source-only zlib
- source-only libpng conservative failure
- emitted crate output from deterministic fixtures
- OpenSSL link directives
- combined Linux event-loop link directives

## Build And Test

```sh
make build
make test
```
