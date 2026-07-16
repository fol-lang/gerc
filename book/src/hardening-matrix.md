# Hardening Matrix

This chapter translates the large GERC generation suite into an explicit
hardening ladder.

The point is not to count tests. The point is to make it obvious which surfaces
are carrying confidence for lowering, emission, and build-output generation.

## Hermetic Regression Baselines

These should stay green everywhere:

- source-only sqlite3 lowering
- source-only zlib lowering
- source-only libpng lowering
- deterministic emitted crate output on vendored fixtures
- large internal corpus fixtures and root API tests
- source-only incomplete-handle lowering
- selected Rust-keyword placeholder regression fixtures

These surfaces prove that GERC can:

- ingest its own source model
- gate declarations conservatively
- lower accepted declarations deterministically
- emit deterministic Rust and build outputs for controlled fixtures

## Host-Dependent And Synthetic Ladders

These strengthen confidence on real native environments:

- OpenSSL link-directive generation
- libxml2 link-directive generation
- synthetic Apple framework link-directive generation
- synthetic Windows system-library link-directive generation
- combined Linux event-loop link-directive generation
- libc and system-library evidence-aware generation families

Linux host-dependent surfaces add real-host evidence. Apple and Windows rows
are synthetic/configuration fixtures only because H0 has no native CI for those
platforms. None of these fixtures prove provider identity or a certified ABI.

## Tier 3: Conservative-Rejection Surfaces

These are good failures, not bad coverage:

- anonymous-type lowering fallback and refusal when by-value naming is not
  honest
- bitfield-by-value and representation-sensitive record rejection
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
- source-only sqlite3 projection
- emitted crate output on deterministic fixtures
- libxml2 link directives when available
- synthetic Apple framework link directives
- synthetic Windows system-library link directives
- incomplete-handle lowering
- keyword-placeholder emission
- OpenSSL link directives when available
- combined Linux event-loop link directives when available

This matrix records regression evidence. It does not complete H1-H5 or certify
packed layouts, identifier policy, provider state, emitted-crate publication,
or any platform.
