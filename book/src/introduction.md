# GERC (`gerc` crate)

`gerc` is the Rust lowering and emission layer in the `PARC -> LINC -> GERC`
toolchain.

It is the crate you use when you already have a normalized source contract and,
optionally, evidence about native symbols or link plans. `gerc` then turns that
input into Rust FFI output.

The target architecture is strict:

- `gerc` library code must not depend on `parc` or `linc`
- `gerc` owns its own generation model
- `gerc` consumes translated source and evidence inputs
- translation from PARC or LINC artifacts belongs only in tests, examples, or external harnesses
- there is no shared ABI crate and no backward-compatibility burden for discarded pipeline shapes

`gerc` generates Rust FFI bindings from C declarations. The output can be a
standalone Rust source bundle or a Cargo-compatible crate directory, and it
also emits plain `rustc` link arguments for non-Cargo toolchains.

One correction matters up front: the current public crate surface is broader
than just `generate()` and `generate_from_source()`. The crate also exposes:

- a crate-owned C-side model under `gerc::c`
- explicit staged `gate` / `lower` modules
- consumer-side inspection and sidecar helpers

In the toolchain split:

- `parc` owns source meaning
- `linc` owns link and binary meaning
- `gerc` owns Rust lowering, Rust emission, and emitted build output

## Audience

This book is for developers who:

- want to understand how `gerc` transforms C declarations into Rust FFI code
- need to integrate `gerc` as a library in their own tooling
- are building downstream consumers on top of `gerc` output

## Scope

`gerc` is intentionally narrow:

| Responsibility | Owner |
|---|---|
| C header parsing and source extraction | `parc` |
| Link, validation, and ABI evidence | `linc` |
| Rust FFI projection and code generation | `gerc` |
| Runtime loader policy and deployment | downstream tooling |
| `fol`-specific surface generation | downstream tooling |

## Quick Start

```rust
use gerc::{generate_from_source, GercConfig, emit_source};
use gerc::intake::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

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
let emitted = emit_source(&output.projection);
println!("{emitted}");
```

If validation evidence is present, `gerc` treats it as a gating input for
functions and variables. Declarations without usable evidence are filtered out
and reported through diagnostics instead of being emitted.
