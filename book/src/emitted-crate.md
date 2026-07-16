# Emitted Crate

## Output Modes

`gerc` supports two output modes via the crate-root `OutputMode` re-export:

### Crate Mode

Writes a Cargo build skeleton:

```text
output_dir/
├── Cargo.toml
├── build.rs              (if link requirements exist and build_script is enabled)
├── rustc-link-args.txt   (if link requirements exist)
└── src/
    └── lib.rs
```

### Source Bundle Mode

Writes only the Rust source file:

```text
output_dir/
├── rustc-link-args.txt   (if link requirements exist)
└── src/
    └── lib.rs
```

## Cargo.toml

The generated `Cargo.toml` includes the crate name, version, edition, and a
short generated-artifact description.

It does not yet include the complete legal metadata, notices, provenance,
reproducibility material, or publication policy required for a publishable
crate. Registry publication and generated-crate release hygiene are deferred to
H6.

## build.rs

When link requirements exist, `gerc` generates a `build.rs` that emits Cargo
link directives.

## rustc-link-args.txt

When link requirements exist, `gerc` also generates a plain text file with
direct `rustc` arguments. This file is intended for non-Cargo toolchains or
custom build orchestration.

## Overwrite Policies

`OverwritePolicy` controls behavior when the output directory already exists:

| Policy | Behavior |
|---|---|
| `Fail` | Return an error if the directory is not empty |
| `Clean` | Remove existing contents before writing |
| `Overwrite` | Write over existing files without removing extras |

## Crate Naming

`normalize_crate_name()` ensures valid Cargo package names:

- replaces non-alphanumeric characters (except `_`) with `_`
- rejects empty names
- rejects names starting with a digit

This helper only normalizes the generated package name. It is not a general
identifier sanitizer and does not prove collision-free Rust identifiers for
emitted C declarations.

## Usage

```rust
use gerc::{emit_crate, OutputMode, OverwritePolicy};

let emitted = emit_crate(
    &output.projection,
    &config,
    std::path::Path::new("/tmp/mylib_sys"),
    OutputMode::Crate,
    OverwritePolicy::Clean,
).unwrap();

assert!(emitted.root.join("Cargo.toml").exists());
assert!(emitted.root.join("src/lib.rs").exists());
assert!(emitted.files.iter().any(|path| path.ends_with("Cargo.toml")));
```

`EmittedCrate` records the crate root directory and the concrete files that
were written. In crate mode that file list is deterministic and includes
`Cargo.toml`, `src/lib.rs`, `rustc-link-args.txt`, and `build.rs` when link
requirements require them.
