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
generation fingerprint binds those typed facts. Initial certified Linux
emission only accepts exact, undecorated native symbol bytes; control-bearing, transformed,
versioned, and otherwise decorated symbols fail with a typed error.

Generated Rust is `#![no_std]`. Complete structs and unions are accepted only
when measured size, alignment, and offsets have an exact natural `repr(C)` or
power-of-two `repr(C, packed(N))` representation. Generated source carries
compile-time size, alignment, field-size, and field-offset assertions. Opaque
records remain pointer-only. Flexible arrays are legal only as a final struct
field. Raw C enums are integer aliases plus constants, so duplicate and unknown
values remain ABI-safe. Global constness is explicit and TLS is rejected.

`RustLinkPlan` consumes LINC's resolved atoms without reparsing. Its
`rustc_arguments()` and `gnu_linker_arguments()` values retain `OsString` and
`PathBuf` units, exact objects, grouping, order, and meaningful repetition.
The plan retains the checked target fingerprint and object format; GNU argv
projection rejects non-ELF targets rather than selecting syntax from the host.
Each Rust native linker value occupies its own repeated `-C link-arg=...`
argument; GERC never emits a whitespace-split `-Clink-args` blob. Apple
framework atoms fail the certified GNU projection rather than borrowing host
syntax.

The host crate forbids unsafe Rust. The generated raw crate necessarily uses
`unsafe extern` declarations but has no host dependency and does not enable
`std`.
