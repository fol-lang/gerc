# GERC (`gerc` today)

`gerc` is the current crate name for `GERC`, the Rust projection layer in the
`PARC -> LINC -> GERC` pipeline.

The target architecture is strict:

- `gerc` library code must not depend on `parc` or `linc`
- `gerc` owns its own generation model
- `gerc` consumes translated source and evidence inputs
- translation from PARC or LINC artifacts belongs only in tests, examples, or
  external harnesses
- there is no shared ABI crate and no backward-compatibility burden for discarded pipeline shapes

`gerc` generates Rust FFI bindings from C declarations. The output is a
complete Cargo-compatible Rust crate (or a loose source bundle), and it also
emits plain `rustc` link arguments for non-Cargo toolchains.

In the toolchain split:

- `parc` owns source meaning
- `linc` owns link and binary meaning
- `gerc` owns Rust lowering and emitted build output

## Audience

This book is for developers who:

- want to understand how `gerc` transforms C declarations into Rust FFI code
- need to integrate `gerc` as a library in their own tooling
- are building downstream consumers (like `fol-interloop-rust`) on top of `gerc`
  output

## Scope

`gerc` is intentionally narrow:

| Responsibility | Owner |
|---|---|
| C header parsing and source extraction | `parc` |
| Link, validation, and ABI evidence | `linc` |
| Rust FFI projection and code generation | **`gerc`** |
| Runtime loader policy and deployment | downstream tooling |
| `fol`-specific surface generation | `fol-interloop-rust` |

## Quick start

```rust
use gerc::{generate_from_source, GercConfig};
use gerc::emit::emit_source;
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
let source = emit_source(&output.projection);
println!("{source}");
```

If validation evidence is present, `gerc` treats it as a hard gating input for
functions and variables. Declarations without usable evidence are filtered out
and reported through diagnostics instead of being emitted.
