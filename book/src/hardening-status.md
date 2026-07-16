# Hardening Status

This book documents the current H0 baseline. GERC is not production-certified,
the PARC/LINC/GERC pipeline is not yet certified for FOL V4, and the H1 through
H5 contracts described in the hardening plan are not implemented milestones.

## Identity And Toolchain

| Item | Current value |
|---|---|
| Distribution package | `follang-gerc` |
| Rust library/import name | `gerc` |
| Declared MSRV | Rust 1.89 |
| Registry publication | Deferred to the H6 distribution gate |
| Metadata and sidecar schema | Version 1; not the frozen H1 schema |

The package and library names are intentionally different. Cargo dependency
metadata uses `follang-gerc`; Rust code imports `gerc`.

## Generation Boundary

| Surface | Current evidence | Not certified |
|---|---|---|
| Rust lowering/emission | Repository regression and corpus fixtures | Correct ABI output for arbitrary C declarations |
| Emitted crate mode | `Cargo.toml`, `src/lib.rs`, and optional link files are generated and tested | A publication-ready crate with legal metadata, notices, provenance, and reproducible release material |
| Packed non-bitfield records/unions | Narrow fixture behavior | General packed-layout equivalence |
| Attached provider/validation data | Gating and link rendering consume translated values | Binary inspection, provider identity, linkability, or runtime availability |
| Identifier handling | Selected keyword and placeholder regressions | A complete collision-free C-to-Rust identifier policy |
| Apple/Windows directives | Synthetic/configuration fixtures | Native platform certification; H0 has no native Apple or Windows CI gate |

## Verification Interface

| Command | Purpose | Prerequisites |
|---|---|---|
| `make build` | Release build | Rust 1.89 toolchain |
| `make fmt-check` | Rust formatting check | `rustfmt` |
| `make lint` | Clippy with warnings denied | `clippy` |
| `make check-features` | Default, all-feature, and no-default checks | Cargo |
| `make test` | Hermetic required tests and doctests | Cargo |
| `make test-contract` | Artifact and root-API contract tests | Cargo |
| `make test-package` | Package archive and clean-consumer check | Cargo and the repository script |
| `make test-system` | Compiler/header/library-dependent pipeline tests | Every prerequisite required by the selected fixtures |
| `make docs-check` | mdBook and Rust API docs | `mdbook`, Cargo/rustdoc |
| `make verify` | Full non-mutating gate | All required prerequisites and a clean worktree |

`make test-system` sets `GERC_SYSTEM_TEST_MODE=required`; missing prerequisites
are `FAIL`, not `SKIP`. Required CI installs them. Documentation builds write
under `target/` and never stage or commit files.
