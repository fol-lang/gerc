# GERC H4 Sound Generation

GERC owns raw Rust FFI projection, deterministic Rust source, and the typed
Rust link projection. PARC owns source meaning and LINC owns native provider,
layout, callable-ABI, and resolved-link evidence.

Production flow is one typed DAG:

```text
parc::contract::CompleteSourcePackage
  + linc::contract::ValidatedLinkAnalysis
  -> gerc::GenerationRequest
  -> gerc::GenerationBundle
```

There are no crate-owned copies of PARC or LINC models, source-only entrypoints,
JSON transmuters, sibling test-module inclusion, optional evidence, or fallback
link surfaces.

GERC refuses a selected or transitive declaration when the Rust ABI projection
cannot honor it. Refusal is a successful safety property, not a request to emit
an opaque unknown type.

The preferred `generate()` path first verifies the complete closed declaration
graph, per-declaration source/target fingerprints, measured layouts, callable
ABI, exact provider-bound symbols, and the ordered provider plan. An independent
post-lowering verifier then rejects unresolved names, alias or by-value record
cycles, opaque values, unsupported callbacks, invalid flexible arrays, and
layout state that cannot be emitted soundly. Every generated Rust translation
unit is parsed before the bundle is returned.

The raw crate is `#![no_std]` and uses `core::ffi`. GERC only returns a sorted
in-memory `GeneratedFileSet`; it exposes no recursive-clean or arbitrary-path
writer. Normal materialization belongs to FOL's action graph.
