# Hardening Evidence Scorecard

This chapter ties GERC readiness to the current hardening ladder instead of
general optimism.

## Overall Posture

GERC is in H0 hardening and is not production-certified. Current tests provide
useful lowering and deterministic-output regression evidence, but do not
certify packed layout, providers, identifiers, ABI correctness, or a
publication-ready generated crate. Apple and Windows coverage is synthetic and
has no native CI gate.

## Subsystem Scorecard

- source-first intake: version-1 fixture-backed behavior
- gate and refusal diagnostics: regression evidence
- lowering and typemapping: fixture-backed, pre-H4 behavior
- deterministic source emission: controlled-input evidence
- emitted crate output: build-skeleton evidence, not publication readiness
- raw `rustc` argument rendering: string-rendering evidence, not link proof
- provider/validation intake: trusted translated data, not independent proof
- Apple/Windows: synthetic evidence only

## Canonical Readiness Anchors

The regression baseline should be checked against these anchors first:

- source-only sqlite3
- source-only zlib
- source-only libpng
- emitted crate output from deterministic fixtures
- source-only pointer-only opaque-handle lowering
- evidence-aware framework link rendering
- narrow packed non-bitfield union fixture behavior
- OpenSSL link directives
- libxml2 link directives
- synthetic Windows system-library link directives
- combined Linux event-loop link directives

If those anchors drift, confidence in the current baseline should drop even if
the smaller unit tests still look healthy. Green anchors do not complete H1-H5
or establish a production floor.
