# Testing And Release

`gerc` is the lowering and emission crate in the toolchain, so its test and
release posture is different from `parc` and `linc`.

The core questions are:

- can `gerc` gate unsafe or unsupported input correctly
- can it lower accepted input deterministically
- can it emit stable Rust and build artifacts
- can tests/examples prove upstream composition without turning upstream crates
  into library dependencies

## Contract Tests

The main contract tests should cover:

- source-first intake through `GercInput` and `generate_from_source`
- evidence-aware gating when validation and link evidence are attached
- staged `BindingPackage -> gate -> lower` paths
- deterministic projection and emission
- emitted crate and source-bundle output for the documented modes
- grouped gate, lowering, and pipeline failure matrices

Those tests are the practical statement of what downstream code may rely on.

## Artifact-Boundary Integration Proof

`gerc/src/**` must not import `parc` or `linc`.

Cross-package proof belongs in:

- `gerc` tests/examples that translate `parc` source artifacts
- `gerc` tests/examples that translate `linc` evidence artifacts
- external harnesses that exercise the whole pipeline

If a change requires `gerc` library code to know another pipeline crate's
internal types, the change is architecturally wrong.

## Determinism Rules

Generated output should be deterministic for the same intake model.

That means tests should prefer:

- stable item ordering
- stable build sidecar output
- stable Rust source snapshots where formatting is part of the contract
- semantic assertions when exact text is intentionally flexible

## Release Checklist

Before releasing `gerc`:

1. run `make build`
2. run `make test`
3. confirm the canonical hardening anchors still pass
   - source-only sqlite3
   - source-only zlib
   - source-only libpng
   - emitted crate output on deterministic fixtures
   - OpenSSL link directives when available
   - libxml2 link directives when available
   - Apple framework link directives
   - Windows system-library link directives
   - libcurl link directives when available
   - combined Linux event-loop link directives when available
4. confirm the preferred public workflow in the README and book still matches
   the tested API
5. confirm emitted Cargo and raw `rustc` paths still match the documented
   output story
6. confirm tests/examples still keep PARC/LINC translation outside `gerc/src/**`

## Hermeticity Split

Read the large test surfaces in three groups:

- always-on hermetic baselines
- host-dependent but high-value evidence ladders
- conservative rejection and degradation paths

The hermetic baselines are the confidence floor. The host-dependent ladders
raise confidence on real targets. The conservative-failure paths prove that
GERC refuses unsound lowering instead of inventing answers.

The grouped failure suites now live in:

- `failure_matrix_gate` for validation-driven gate refusals
- `failure_matrix_lower` for anonymous and unsupported lowering failures
- `failure_matrix_pipeline` for the closed source-only anonymous-type cargo-check
  regression

## Maintenance Rule

When `gerc` changes:

1. update the smallest meaningful test first
2. update emitted-output docs in the same patch when behavior changes
3. keep upstream translation in tests/examples/harnesses only
4. delete stale workflow language instead of preserving it for history

## What "Supported" Means

For `gerc`, support means:

- accepted declarations lower to deterministic Rust and build artifacts
- rejected declarations fail conservatively and diagnostically
- documented output modes are covered by tests

It does not mean:

- every upstream C declaration can already be emitted
- every rejected item is a bug rather than a deliberate safety boundary
- `gerc` library code is allowed to absorb upstream crate internals
