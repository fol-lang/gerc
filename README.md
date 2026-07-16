# GERC

GERC is the Rust FFI projection stage in the typed
`PARC -> LINC -> GERC` pipeline.

For a repository development checkout, spell every package and library
identity explicitly:

```toml
[dependencies]
parc = { package = "follang-parc", path = "../parc" }
linc = { package = "follang-linc", path = "../linc", default-features = false, features = ["contracts"] }
gerc = { package = "follang-gerc", path = "../gerc" }
```

Registry publication is disabled. Released consumers must use the exact tested
Git tags/archives described by the release policy rather than inventing
registry versions or following unpinned branches.

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

Generated raw bindings use `#![no_std]` and `core::ffi`.

## H5 certification matrix

The production corpus admits only LINC-owned certification output; callers do
not supply layouts, probes, callable shapes, or declaration evidence. These are
the exact type states in `tests/pipeline_support/matrix.rs`:

- **supported-and-tested** (`PARC/LINC/GERC`): `void`, `_Bool`, plain/signed/
  unsigned `char`, signed/unsigned `short`, `int`, `long`, and `long long`,
  `float`, `double`, raw and nullable pointers, nonzero fixed arrays, complete
  records and unions with measured layout, incomplete records behind pointers,
  C enums as integer aliases and constants, C-calling-convention routines, and
  C-calling-convention callbacks.
- **explicitly-rejected**: long double and extended floating types
  (`LINC-E3014`), complex floating types (`LINC-E3014`), compiler vectors
  (`PARC` `CompletionBlocker::Unsupported`), `_BitInt` (`PARC-P1107`), C
  128-bit integers (`LINC-E3014`), by-value opaque/incomplete records (`PARC`
  `CompletionBlocker::IncompleteRecord`), bitfield layouts (`GERC-E2002`),
  unsupported calling conventions (`LINC-E3050`), variadic or unspecified
  callables (`LINC-E3050`), C++ types/ABI (`PARC-P0002`), and thread-local
  globals (`GERC-E2002`).
- **experimental-not-for-FOL**: function-like and string macros are preserved
  but not emitted (`GERC-N3000`).

| Platform | State | Owner / result |
| --- | --- | --- |
| x86_64-unknown-linux-gnu, ELF, explicit GCC | supported-and-tested | H5 pipeline |
| x86_64-unknown-linux-gnu, ELF, explicit Clang | experimental-not-for-FOL | optional full typed H5 differential value roundtrip |
| x86_64-unknown-linux-musl | explicitly-rejected | H5 gate: not certified |
| second Linux architecture | explicitly-rejected | H5 gate: not certified |
| aarch64-apple-darwin | explicitly-rejected | H5 gate: not certified |
| x86_64-pc-windows-msvc and MinGW | explicitly-rejected | H5 gate: not certified |

The domain values deliberately have no JSON decoder. PARC and LINC own their
respective strict transport schemas; GERC owns an immutable in-memory
projection.

## Verification

The MSRV is Rust 1.89.

```sh
make verify
make verify-pipeline
```

`make verify` includes the generated-source parser/build lane, the explicit GCC
C/Rust ABI link-and-run fixture, package extraction with a nonzero clean
consumer test, the preservation corpus, and `make verify-pipeline`.
`verify-pipeline` requires the exact clean audited PARC/LINC revisions, runs
their Makefile validation targets, then certifies, generates, links, and runs
the H5 value corpus. Set `GERC_H5_CLANG` to an explicit certifiable Clang binary
to add the optional full differential lane; ambient `CC` is ignored.

## Distribution and compatibility

The package identity is `follang-gerc` 0.1.0 and the Rust import name is
`gerc`. Registry publication is disabled (`publish = false`), so no crates.io
name ownership or availability is claimed. Candidate archives and the full
pipeline are tested against exact `follang-parc` 0.16.0 revision
`11ca2be6d3dcda7227c0d9eb6c90259838f289fc` and exact `follang-linc` 0.1.0
revision `96ae7c108a34063d8463f7ddbd4bd1d4d6fd57e2`.

`make release-check` is a non-mutating eligibility check. It never changes a
version, commits, tags, pushes, uploads, or publishes. SemVer, generation
domain/algorithm, MSRV, certified-surface, exact-upstream, and tag/archive
rules are recorded in [`RELEASE.md`](RELEASE.md).

## License

Dual-licensed under Apache 2.0 or MIT.
