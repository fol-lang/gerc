# Typed API

`ItemSelection` is a nonempty `DeclarationId` subset of the roots proved by the
borrowed `CompleteSourcePackage`. `GenerationRequest::try_new` recomputes the
selected transitive closure and requires checked LINC evidence for every member.

`generate` returns:

- `ValidatedRustProjection` — private construction, no deserialization
- `GeneratedFileSet` — sorted, in-memory logical files
- `RustLinkPlan` — ordered lossless native link atoms
- `GenerationManifest` — source, target, evidence, and generation fingerprints
- typed diagnostics with stable codes and fingerprint context

The generation fingerprint binds the frozen schema/algorithm/tool identity,
selection, canonical projection, diagnostics, generated files, and ordered link
plan. Native strings are hashed in their platform units.

The projection retains PARC declaration identity, original and normalized C
names, linkage, visibility, support, occurrences, child attributes and support,
and anonymous-child identity facts alongside sanitized `RustName` values. The
generation fingerprint binds those typed facts. Initial Linux H1 emission only
accepts exact, undecorated native symbol bytes; control-bearing, transformed,
versioned, and otherwise decorated symbols fail with a typed error.

Generated Rust is `#![no_std]`. Complete records are accepted only when measured
layout equals natural `repr(C)` field size, alignment, and offsets; generated
source carries compile-time size, alignment, and field-offset assertions.
