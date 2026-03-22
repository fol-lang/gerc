# Testing And Release

`gerc` is the lowering and emission crate in the toolchain, so its test and
release posture is different from `parc` and `linc`.

The core questions are:

- can `gerc` gate unsafe or unsupported input correctly
- can it lower accepted input deterministically
- can it emit stable Rust/build artifacts
- can tests/examples prove upstream composition without turning upstream crates
  into library dependencies

## Contract tests

The main contract tests should cover:

- source-first intake through `GecInput` and `generate_from_source`
- evidence-aware gating when validation and link evidence are attached
- deterministic projection and emission
- emitted crate/build output for the documented modes

Those tests are the practical statement of what downstream code may rely on.

## Artifact-boundary integration proof

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
3. confirm the preferred public workflow in the README and book still matches
   the tested API
4. confirm emitted Cargo and raw `rustc` paths still match the documented
   output story
5. confirm tests/examples still keep PARC/LINC translation outside `gerc/src/**`

## Maintenance Rule

When `gerc` changes:

1. update the smallest meaningful test first
2. update emitted-output docs in the same patch when behavior changes
3. keep upstream translation in tests/examples/harnesses only
4. delete stale workflow language instead of preserving it for history

## What "supported" means

For `gerc`, support means:

- accepted declarations lower to deterministic Rust/build artifacts
- rejected declarations fail conservatively and diagnostically
- documented output modes are covered by tests

It does not mean:

- every upstream C declaration can already be emitted
- every rejected item is a bug rather than a deliberate safety boundary
- `gerc` library code is allowed to absorb upstream crate internals
