# Distribution and release policy

This file defines GERC's distribution identity and compatibility rules. It is
included in every Cargo archive.

## Current identity

| Item | Value |
| --- | --- |
| Cargo package | `follang-gerc` 0.1.0 |
| Rust library/import | `gerc` |
| Edition | Rust 2021 |
| MSRV | Rust 1.89 |
| License | `MIT OR Apache-2.0` |
| Generation domain | `follang.gerc.generation`, version 1 |
| Generation algorithm | `gerc-rust-ffi-projection-v1` |
| Generator identity | `follang-gerc/0.1.0` |
| Certified implementation surface | H5 typed Linux ELF pipeline |
| H5 implementation baseline | `26f04db4e1b3d0b0d24789f20ced3a0db7875c4c` |
| Required PARC package | `follang-parc` exactly 0.16.0 |
| Required PARC revision | `0f52aeeeeec47a082c0d8a515130ee853aa1101d` |
| Required LINC package | `follang-linc` exactly 0.1.0 |
| Required LINC revision | `c874d5b0332249524422d9d08c35b3d4edd7e3fa` |

The Rust constants `GENERATION_SCHEMA_ID`, `GENERATION_SCHEMA_VERSION`,
`GENERATION_ALGORITHM_ID`, and `GENERATOR_IDENTITY` are the authority for
consumers. The generation domain is an immutable in-memory contract, not a JSON
schema. The H5 baseline identifies the implementation certified before this
distribution-only hardening change. A release tag records the exact
archive-producing commit, including later documentation or packaging changes.

The production certification surface is the explicitly tested C17 GNU
x86-64 Linux ELF LP64/SysV pipeline with GCC, checked PARC source, certified
LINC evidence, and the supported/rejected type matrix in the book. Explicit
Clang repeats the full differential roundtrip but remains
experimental-not-for-FOL. GERC does not claim certified Mach-O, COFF,
framework, non-SysV ABI, C++, arbitrary C extensions, unchecked evidence,
filesystem materialization, or shell-interpreted link support.

## Distribution channel

`Cargo.toml` sets `publish = false`. No crates.io name ownership, availability,
or published release is asserted. The supported distribution channel is a
self-contained `.crate` archive produced from an exact Git tag. Consumers use
that archive or the exact tag commit and import the library as `gerc`.

`make test-package` requires the exact clean PARC and LINC revisions above,
builds all three candidate archives, and unpacks them outside the repositories.
It checks normalized archive metadata and dependencies, runs the extracted
workspace's promised tests and full Linux pipeline, and builds/tests a clean
external consumer against package `follang-gerc` under the crate name `gerc`.
The consumer selects the extracted package identities by exact version and
performs a nonempty typed generation; it does not depend on repository source
paths or path-only development dependencies.

The `pipeline-native` feature adds LINC's native certification implementation
for the repository's full pipeline lane. It does not widen the documented type
or platform matrix.

## Compatibility versions

The Cargo package version follows SemVer for the Rust API and documented
behavior. Before 1.0, a breaking Rust API or behavior change requires a minor
version bump; a backwards-compatible fix or additive change may use a patch
bump. After 1.0, normal SemVer major/minor/patch rules apply.

The generation domain and algorithm are independent compatibility axes:

- Generation domain version 1 is frozen. An incompatible typed-domain shape,
  invariant, or meaning change requires a new domain version and a breaking
  SemVer bump (minor before 1.0, major after 1.0).
- Changing fingerprint field order, canonical projection inputs, hashing, or
  emission semantics requires a new generation algorithm ID, new golden
  vectors, and the same breaking SemVer bump. If domain meaning also changes,
  its version must change too.
- Compatible implementation fixes that leave domain meaning, the algorithm,
  generated files, link plans, and fingerprints unchanged do not bump either
  identity.
- `GENERATOR_IDENTITY` contains the Cargo package version, so every package
  version intentionally produces release-specific generation fingerprints.
  That expected provenance change alone does not change the domain or algorithm
  identity.
- Consumers accept only domain and algorithm versions they explicitly
  implement. A package version bump never makes an unknown identity acceptable.

Changing the certified target or supported-type matrix must update executable
inventory and documentation together. Broadening a unit enum or string is not
evidence that an untested platform or ABI is certified.

The MSRV is the `package.rust-version` value in `Cargo.toml` and is exercised by
CI. A patch release does not raise it. Before 1.0, raising the MSRV requires at
least a minor package-version bump; after 1.0 it requires at least a minor bump.
The change must update `Cargo.toml`, this file, and CI together.

## Release order and clean-upstream rule

Sibling releases or tags are ordered:

1. PARC contract archive/tag.
2. LINC against that exact PARC version and commit.
3. GERC against those exact PARC and LINC versions and commits.
4. FOL after its lock records all three exact revisions.

Never tag GERC against uncommitted or merely local sibling state.

Before proposing `follang-gerc-v<version>`:

1. merge the candidate and its required PARC/LINC revisions to their tracked
   upstream branches;
2. run `git fetch --tags origin` in all three repositories and review the
   fetched state;
3. check out all three release branches with clean worktrees;
4. run `make release-check` from GERC;
5. review the reported version, tag name, full GERC commit ID, and both full
   upstream commit IDs;
6. create the tag/archive manually under the repository's review policy;
7. record the exact PARC/LINC/GERC tag commits and package/domain/algorithm
   versions in FOL before updating its lock.

`make release-check` refuses detached, dirty, untracked, non-upstream,
already-tagged, registry-publishable, or wrong-sibling state. It then runs
`make verify`. It performs no fetch, version edit, commit, tag, push, upload, or
publication.
