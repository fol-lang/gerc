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

## Tested Scope

The suite covers:

- source-only generation
- evidence-aware generation
- staged gate/lower usage
- root API re-exports
- emitted crate output
- raw `rustc` argument output
- larger fixture surfaces and artifact-boundary tests

## Build And Test

```sh
make build
make test
```
