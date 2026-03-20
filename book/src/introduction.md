# gec

`gec` is the Rust projection layer on top of [`bic`](https://github.com/bresilla/bic).

It consumes `bic` analysis results and generates Rust FFI bindings from C
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
| C header parsing and ABI extraction | `bic` |
| Rust FFI projection and code generation | **`gec`** |
| Runtime loader policy and deployment | downstream tooling |
| `fol`-specific surface generation | `fol-interloop-rust` |

## Quick start

```rust
use bic::*;
use gec::{GecConfig, GecInput, generate};
use gec::emit::emit_source;

let mut pkg = BindingPackage::new();
pkg.items.push(BindingItem::Function(FunctionBinding {
    name: "init".into(),
    calling_convention: CallingConvention::C,
    parameters: vec![],
    return_type: BindingType::Int,
    variadic: false,
    source_offset: None,
}));

let input = GecInput::from_package(pkg);
let config = GecConfig::new("mylib_sys");
let output = generate(&input, &config).unwrap();
let source = emit_source(&output.projection);
println!("{source}");
```
