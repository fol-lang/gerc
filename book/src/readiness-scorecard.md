# Readiness Scorecard

This chapter ties GERC readiness to the current hardening ladder instead of
general optimism.

## Overall Posture

GERC should currently be read as:

- strong on source-only lowering fundamentals
- strong on deterministic emission and emitted-crate generation
- strong on conservative rejection of unsupported shapes
- useful and increasingly hardened on evidence-aware large surfaces
- still dependent on host availability for the biggest OpenSSL and Linux-system
  evidence ladders

That is a good release posture for a young lowering crate, but it is not yet a
claim that every ugly native surface will lower cleanly.

## Subsystem Scorecard

- source-first intake: high
- gate and refusal diagnostics: high
- lowering and typemapping: high
- deterministic source emission: high
- emitted crate output: high
- raw `rustc` argument rendering: high
- source-only large-surface confidence: high
- evidence-aware large-surface confidence: medium-high
- conservative rejection on difficult layouts: high

## Canonical Readiness Anchors

The release posture should be judged against these anchors first:

- source-only zlib
- source-only libpng
- emitted crate output from deterministic fixtures
- OpenSSL link directives
- combined Linux event-loop link directives

If those anchors drift, the scorecard should drop even if the smaller unit tests
still look healthy.
