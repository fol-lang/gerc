# GERC

`gerc` is the Rust lowering and emission layer in the `parc -> linc -> gerc`
toolchain.

It produces Rust-facing output from translated C-side inputs:

- a `RustProjection`
- emitted Rust source
- emitted crate directories and `build.rs`
- raw `rustc` link arguments
- metadata sidecars for downstream consumers

## Hardening Status

GERC is being hardened as the raw Rust projection and emission owner for the
sibling PARC/LINC/GERC pipeline. It is not yet certified for FOL V4.

The distribution package is `follang-gerc`; the Rust library name remains
`gerc`. Registry publication is deferred until the H6 distribution gate, and
the crate version remains unchanged during baseline hardening. The declared
minimum supported Rust version (MSRV) is Rust 1.89.

## Current Support Boundary

| Area | Current evidence | Boundary |
|---|---|---|
| Source lowering and emission | Repository fixtures exercise current mappings | This is pre-certification behavior; accepted output is not proof of a correct ABI for arbitrary declarations. |
| Emitted crate mode | Tests cover the generated `Cargo.toml`, `src/lib.rs`, and optional link files | This is a build skeleton, not a publication-ready crate. Legal metadata, notices, provenance, and reproducible publication work are deferred to H6. |
| Packed records/unions | Narrow regression fixtures exercise current non-bitfield output | GERC has not certified packed-layout equivalence; do not treat fixture acceptance as a general packed-layout guarantee. |
| Provider evidence | GERC gates on translated report/plan values | It does not inspect binaries or independently prove provider identity, linkability, or runtime availability. |
| Identifier handling | Tests cover selected Rust keywords and placeholder names | There is no certified, collision-free C-to-Rust identifier policy yet. |
| Apple and Windows link directives | Synthetic/configuration fixtures exist | Neither platform is certified; H0 has no native Apple or Windows CI gate. |
| Metadata/sidecar schemas | Schema version 1 roundtrips are exercised | These are current artifact shapes, not the frozen H1 contract. |

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
- crate/build-skeleton output generation
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

## Artifact Boundary

`gerc` owns its own lowering and emission model plus the emitted build-side
artifacts.

The durable boundaries are:

- the crate-owned intake model in `gerc::c`
- emitted Rust source
- emitted crate directories and `build.rs`
- rendered raw `rustc` link-argument files
- metadata sidecars for downstream consumers

Cross-package translation still belongs outside `gerc/src/**`. GERC can be used
in integration tests and harnesses, but its library code is not where upstream
`parc` or `linc` wiring should live.

## Tested Scope

The suite covers:

- source-only generation
- evidence-aware generation
- staged gate/lower usage
- root API re-exports
- emitted crate output
- raw `rustc` argument output
- larger fixture surfaces and artifact-boundary tests
- OpenSSL, libpng, and combined Linux event-loop link-directive generation
- explicit gate, lowering, and pipeline failure matrices

The tests are the best statement of what GERC actually supports.

## Current Contract

For the current hardening baseline, GERC's contract is:

- supported families lower deterministically on the named canonical corpus
- evidence-aware lowering may expand support beyond source-only mode
- explicitly rejected families remain rejected until an honest representation
  strategy exists
- conservative rejection is current behavior, not proof that the H4 soundness
  boundary has been completed

This is a pre-certification contract. H1 through H5 of the hardening plan
remain future milestones.

## Current Test Evidence

The current hardening ladder is easiest to read in four buckets:

- hermetic vendored baselines
  - source-only zlib lowering
  - source-only libpng lowering
  - emitted crate output on deterministic vendored surfaces
- host-dependent evidence ladders
  - OpenSSL link-directive generation
  - libcurl link-directive generation
  - libxml2 link-directive generation
  - combined Linux event-loop link-directive generation
- failure and conservative-lowering surfaces
  - anonymous-type fallback and rejection paths
  - incomplete-handle support for pointer-only opaque families
  - narrow packed non-bitfield record/union regression fixtures
  - bitfield-bearing record rejection when representation would be unsound
  - selected Rust-keyword placeholder regression fixtures
  - unsupported layout and ABI-sensitive gating
  - source-only degradation when link evidence is absent
  - explicit gate, lowering, and pipeline failure matrices
- determinism anchors
  - source-only sqlite3 projection
  - source-only zlib projection
  - source-only libpng projection
  - libxml2 link directives when available
  - OpenSSL link directives when available
  - synthetic Apple framework link-directive fixtures
  - synthetic Windows system-library link-directive fixtures
  - combined Linux event-loop link directives

Read those as the current confidence anchors, not as a claim that every native
surface lowers equally well today.

Host-dependent evidence includes:

- OpenSSL link directives
- libxml2 link directives
- libcurl link directives
- synthetic Apple framework link-directive fixtures
- synthetic Windows system-library link-directive fixtures
- combined Linux event-loop link directives

The current canonical generation surfaces are:

- source-only zlib
- source-only libpng
- source-only sqlite3
- source-only support-tier widget fixture
- emitted crate output from deterministic fixtures
- OpenSSL link directives
- libxml2 link directives
- synthetic Apple framework link-directive fixtures
- synthetic Windows system-library link-directive fixtures
- libcurl link directives
- combined Linux event-loop link directives

The current GERC test corpus is intentionally named:

- hermetic vendored
  - source-only zlib
  - source-only libpng
  - source-only sqlite3
  - deterministic emitted-crate checks on vendored fixtures
- hermetic support-tier anchors
  - source-only supported widget fixture
  - source-only rejected bitfield fixture
  - evidence-aware link-plan fixture
  - pointer-only incomplete-handle fixture
  - keyword-placeholder emission fixture
- host-dependent raises
  - OpenSSL evidence-aware generation
  - libxml2 evidence-aware generation
  - synthetic Apple framework evidence translation
  - synthetic Windows system-library evidence translation
  - combined Linux event-loop evidence-aware generation
- conservative-failure anchors
  - anonymous-type rejection ledger
  - explicit gate/lower/pipeline failure matrices

Those are test anchors, not packed-layout, provider, identifier, ABI, or
platform certification.

## Verification

```sh
make build
make fmt-check
make lint
make check-features
make test
make test-contract
make test-package
make test-system
make docs-check
make verify
```

`make test` is the hermetic required lane. `make test-system` sets
`GERC_SYSTEM_TEST_MODE=required` and runs the prerequisite-dependent pipeline
tests; a missing compiler or required development header/library is `FAIL`, not
`SKIP`. Required CI installs those prerequisites. `make docs-check` requires
`mdbook` and builds both the book and Rust API documentation without staging or
committing output.

`make verify` expects a clean worktree, runs the common gates above, and proves
that validation did not change Git state. During local review of an already
dirty tree, `VERIFY_ALLOW_DIRTY=1 make verify` retains the before/after check.
