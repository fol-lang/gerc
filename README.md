# GERC (`gerc` crate)

`gerc` is the Rust lowering and emission layer in the `PARC -> LINC -> GERC`
toolchain.

It consumes `gerc`-owned source and evidence inputs and produces deterministic
Rust FFI output: projected Rust IR, emitted Rust source, Cargo crate
scaffolding, `build.rs`, and plain `rustc` link arguments.

Architecturally:

- `gerc/src/**` must not depend on `parc` or `linc`
- translation from PARC or LINC artifacts belongs only in tests, examples, or external harnesses
- `gerc` owns its own intake model, projection model, and emitted artifacts
- there is no shared ABI crate
- there is no backward-compatibility burden for discarded pipeline shapes

In the intended split:

- `parc` owns source meaning
- `linc` owns link and binary meaning
- `gerc` owns Rust lowering, Rust emission, and emitted build metadata

## Responsibilities

- intake of `gerc::SourcePackage` plus optional evidence inputs
- conservative gating of unsupported or under-evidenced declarations
- lowering into a `RustProjection`
- deterministic Rust source emission
- Cargo crate and source-bundle emission
- native link metadata emission for Cargo and direct `rustc`

## Non-responsibilities

- parsing or preprocessing C source
- inspecting native binaries directly
- inventing ABI facts that should come from upstream contracts
- downstream runtime policy or loader strategy
- library-level dependency on `parc` or `linc`

## Artifact Boundary

`gerc` consumes its own contracts and emits its own artifacts.

- `parc` may serialize source artifacts
- `linc` may serialize evidence artifacts
- tests/examples/harnesses may translate those artifacts into `gerc` input
- `gerc` library code stays on its own side of the boundary

That keeps generation independent from upstream crate internals and keeps the
book honest about what belongs in `src/**`.

## Tested Scope

The suite currently exercises:

- source-only generation from translated source artifacts
- evidence-aware generation from translated source and evidence artifacts
- deterministic projection and emitted-source behavior
- emitted Cargo crate output and raw `rustc` argument output
- larger fixture corpora including hostile surfaces and real-library examples

The tests are the main statement of supported behavior.

## Build And Test

```sh
make build
make test
```

## Book Structure

The book is arranged as a normal toolchain guide:

1. introduction
2. overview
3. architecture
4. intake contract
5. API contract
6. code generation
7. emitted crate
8. testing and release

Each chapter explains the same split from a different angle so the docs stay
parallel to `parc` and `linc`.
