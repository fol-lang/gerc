# GERC H1 Contract

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
