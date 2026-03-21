# gec

`gec` is the Rust generation layer in the `PARC -> LINC -> GERC` pipeline.

It consumes `linc` source/link/evidence contracts and produces deterministic
Rust FFI output: a projected Rust IR, emitted Rust source, and optionally a
Cargo-compatible crate bundle with `build.rs`.

## Responsibilities

- intake of `linc::BindingPackage` plus optional validation and link-plan evidence
- conservative gating of unsupported or under-evidenced declarations
- lowering into Rust projection IR
- deterministic Rust source emission
- emitted crate and build-script generation

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

## Minimal usage

```rust
use gec::{generate, GecConfig, GecInput};
use gec::emit::emit_source;
use linc::{
    BindingItem, BindingPackage, BindingType, CallingConvention, FunctionBinding,
};

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

assert!(source.contains("pub fn init"));
```
