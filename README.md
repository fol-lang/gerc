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
- unsupported ABI shapes return typed errors; there is no unknown-type fallback

The domain values deliberately have no JSON decoder. PARC and LINC own their
respective strict transport schemas; GERC H1 owns an in-memory projection.

## Verification

The MSRV is Rust 1.89.

```sh
make verify
```
