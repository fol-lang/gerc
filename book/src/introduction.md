# gec

`gec` is the Rust projection layer in the `PARC -> LINC -> GERC` pipeline.

It consumes `linc` contracts and generates Rust FFI bindings from C
declarations.  The output is a complete Cargo-compatible Rust crate (or a loose
source bundle) that compiles cleanly with `cargo build`.

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
println!("{source}");
```
