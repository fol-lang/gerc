# Hardening Matrix

This chapter translates the large GERC generation suite into an explicit
hardening ladder.

The point is not to count tests. The point is to make it obvious which surfaces
are carrying confidence for lowering, emission, and build-output generation.

## Tier 1: Hermetic Canonical Baselines

These should stay green everywhere:

- source-only zlib lowering
- source-only libpng lowering
- deterministic emitted crate output on vendored fixtures
- large internal corpus fixtures and root API tests

These surfaces prove that GERC can:

- ingest its own source model
- gate declarations conservatively
- lower accepted declarations deterministically
- emit stable Rust and build artifacts

## Tier 2: Host-Dependent High-Value Ladders

These strengthen confidence on real native environments:

- OpenSSL link-directive generation
- combined Linux event-loop link-directive generation
- libc and system-library evidence-aware generation families

These surfaces matter because they prove that GERC can use real upstream
evidence without taking a library dependency on upstream crates.

## Tier 3: Conservative-Rejection Surfaces

These are good failures, not bad coverage:

- anonymous-type lowering fallback and refusal when by-value naming is not
  honest
- unsupported layout or ABI-sensitive gating
- source-only degradation when link evidence is absent
- explicit rejection of declarations that would produce unsound Rust

Those tests should remain:

- deterministic
- diagnostic
- easy to point to in release discussions

## Determinism Anchors

The most important repeat-run anchors right now are:

- source-only zlib projection
- source-only libpng projection
- emitted crate output on deterministic fixtures
- OpenSSL link directives when available
- combined Linux event-loop link directives when available
