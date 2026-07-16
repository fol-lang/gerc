# GERC

GERC is the Rust FFI projection stage in the typed
`PARC -> LINC -> GERC` pipeline.

```rust,no_run
use gerc::{generate, GenerationRequest, ItemSelection};

# fn example(
#     source: &parc::contract::CompleteSourcePackage,
#     evidence: &linc::contract::ValidatedLinkAnalysis,
# ) -> Result<(), gerc::GenerationError> {
let selection = ItemSelection::from_complete(source);
let request = GenerationRequest::try_new(source, evidence, &selection)?;
let bundle = generate(request)?;

let rust_source = bundle
    .files()
    .get("src/lib.rs")
    .expect("the Rust source is always generated")
    .utf8_contents()
    .expect("generated Rust is UTF-8");
println!("{rust_source}");
# Ok(())
# }
```

The production boundary is strict:

- source must be a checked `parc::contract::CompleteSourcePackage`
- evidence must be a checked `linc::contract::ValidatedLinkAnalysis`
- selection uses `DeclarationId`, never names
- output is an immutable `GenerationBundle`
- generated files stay in memory; GERC has no overwrite or arbitrary-directory API
- native link order, repetition, paths, and `OsString` names remain lossless
- `RustLinkPlan::rustc_arguments()` returns exact per-argument native values;
  it never creates a shell string or `-Clink-args` blob, and GNU projection
  rejects a non-ELF target
- unsupported ABI shapes return typed errors; there is no unknown-type fallback
- every emitted Rust file is parsed as a production postcondition

Generated raw bindings use `#![no_std]` and `core::ffi`. The certified H4
projection covers measured natural and packed records, unions, raw C enums,
fixed arrays, raw pointers, non-variadic C functions/callbacks, and non-TLS
globals. Incomplete records are pointer-only. Extended floating types,
complex/vector/bitfield forms, variadics, unsupported calling conventions,
by-value opaque records, and TLS fail before emission.

The domain values deliberately have no JSON decoder. PARC and LINC own their
respective strict transport schemas; GERC H1 owns an in-memory projection.

## Verification

The MSRV is Rust 1.89.

```sh
make verify
```

`make verify` includes the generated-source parser/build lane, the explicit GCC
C/Rust ABI link-and-run fixture, package extraction with a nonzero clean
consumer test, and the preservation corpus.
