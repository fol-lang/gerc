# GERC (`gec` today)

`gec` is the current crate name for `GERC`, the Rust projection layer in the
`PARC -> LINC -> GERC` pipeline.

The target architecture is strict:

- `gec` library code must not depend on `parc` or `linc`
- `gec` owns its own generation model
- `gec` consumes translated source and evidence inputs
- translation from PARC or LINC artifacts belongs only in tests, examples, or
  external harnesses
- there is no shared ABI crate and no backward-compatibility burden for discarded pipeline shapes

`gec` generates Rust FFI bindings from C declarations. The output is a
complete Cargo-compatible Rust crate (or a loose source bundle), and it also
emits plain `rustc` link arguments for non-Cargo toolchains.

In the toolchain split:

- `parc` owns source meaning
- `linc` owns link and binary meaning
- `gec` owns Rust lowering and emitted build output

## Audience

This book is for developers who:

- want to understand how `gec` transforms C declarations into Rust FFI code
- need to integrate `gec` as a library in their own tooling
- are building downstream consumers (like `fol-interloop-rust`) on top of `gec`
  output

## Scope

`gec` is intentionally narrow:

| Responsibility | Owner |
|---|---|
| C header parsing and source extraction | `parc` |
| Link, validation, and ABI evidence | `linc` |
| Rust FFI projection and code generation | **`gec`** |
| Runtime loader policy and deployment | downstream tooling |
| `fol`-specific surface generation | `fol-interloop-rust` |

## Quick start

```rust
use gec::{generate_from_source, GecConfig};
use gec::emit::emit_source;
use gec::intake::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

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
println!("{source}");
```

If validation evidence is present, `gec` treats it as a hard gating input for
functions and variables. Declarations without usable evidence are filtered out
and reported through diagnostics instead of being emitted.
