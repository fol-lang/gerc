# Release Policy and Checklist

The root `RELEASE.md`, which is included in the package archive, is the
normative distribution policy. This chapter summarizes the repository checks.

## Identity and compatibility

The current identities are:

- package `follang-gerc` 0.1.0, imported as `gerc`;
- MSRV Rust 1.89;
- in-memory generation domain `follang.gerc.generation` version 1;
- generation algorithm `gerc-rust-ffi-projection-v1`;
- H5 implementation baseline
  `26f04db4e1b3d0b0d24789f20ced3a0db7875c4c`;
- exact upstream `follang-parc` 0.16.0 revision
  `11ca2be6d3dcda7227c0d9eb6c90259838f289fc`; and
- exact upstream `follang-linc` 0.1.0 revision
  `96ae7c108a34063d8463f7ddbd4bd1d4d6fd57e2`.

Registry publication is disabled. The project does not claim crates.io name
ownership or availability. Distribution uses an exact Git tag and its tested
self-contained Cargo archive.

Rust API and behavior changes follow SemVer. Before 1.0, breaking changes
require a minor bump. The frozen generation domain and algorithm are never
changed in place: incompatible domain meaning or projection/fingerprint
changes require new identities, golden vectors, and a breaking SemVer bump. A
patch release does not raise the MSRV. `GENERATOR_IDENTITY` contains the package
version, so generation fingerprints intentionally identify their exact release.
Detailed rules are in `RELEASE.md`.

## Certified boundary

The production corpus covers the explicitly checked C17 GNU x86-64 Linux ELF
LP64/SysV pipeline with GCC, exact provider inputs, measured LINC evidence, and
the supported/rejected type matrix in this book. Explicit Clang is the optional
full differential lane and remains experimental-not-for-FOL. Unsupported
targets, ABIs, source types, provider states, and projections fail at their
owning boundary. Distribution metadata does not broaden that matrix.

## Candidate gate

The operator must first fetch and review the tracked upstream and tags in GERC,
LINC, and PARC. On clean branches whose `HEAD`s exactly equal their tracked
upstreams, run:

```sh
make release-check
```

The target refuses detached, dirty, untracked, non-upstream, already-tagged,
registry-publishable, or wrong-sibling state, then runs the full `make verify`
gate. It is non-mutating: it does not fetch, edit a version, commit, tag, push,
upload, or publish.

The full gate proves:

- formatting, Clippy, feature combinations, tests, and doctests;
- the frozen generation contract and preservation corpus;
- generated Rust parsing, compilation, and a real GCC ABI roundtrip;
- the full exact-revision PARC -> LINC -> GERC production corpus;
- extracted package archives and a nonzero clean typed consumer;
- mdBook and Rust API documentation; and
- no worktree change during verification.

## Dependency order

Tag PARC first, then LINC against that exact PARC state, then GERC against both
exact upstream states. Finally update FOL's lock to all three exact revisions.
Never tag a downstream crate against uncommitted or local-only upstream state.
