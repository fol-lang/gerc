# Verification

The repository Makefile is the validation interface:

```sh
make fmt-check
make lint
make check-features
make test
make test-contract
make test-generated
make test-package
make test-system
make verify-pipeline
make docs-check
```

The preservation corpus proves an ABI-supported record/enum subset, exact
fingerprint propagation, identical output across separate processes and
working directories, generated `no_std` compilation, and ordered repeated
native link atoms. The same typed corpus also proves that its Win64 function is
rejected on the Linux target rather than emitted under a guessed ABI.

`make test-generated` requires explicit `gcc` and `rustc` executables. It
compiles a C17 provider object, builds the generated raw Rust crate, links a
Rust consumer against that exact object as one process argument, runs it, and
checks record and enum values across the ABI boundary. `GERC_H4_GCC` may name
an explicit GCC executable; ambient `CC` is deliberately ignored.

Package validation requires the recorded clean PARC and LINC revisions,
extracts PARC, then LINC, then GERC, and rejects path dependencies in the
normalized archive manifests. It builds a scratch consumer using only
`[patch.crates-io]` entries for the extracted packages. The consumer selects
each package by exact version, runs a nonzero typed generation test, and checks
the generated file and ordered link plan. On Linux the package gate also runs
the full pipeline from the extracted archive, including explicit GCC and the
optional explicit Clang differential when Clang is installed.

`make verify-pipeline` first requires the pinned PARC and LINC Git revisions to
be clean, validates both siblings through their Makefiles, and then runs the H5
production corpus. The corpus uses LINC-owned compiler observation and
certification, exact ordered static/shared/object providers, generated `no_std`
Rust compilation, a linked value roundtrip, and owning-layer negative cases.
`GERC_H5_CLANG` optionally names an explicit certifiable Clang binary for the
full differential lane; it is never inferred from ambient `CC`.

The pinned release inputs are PARC revision
`ba603cdccc9375473eca0c42e5462cf90b6da249` and LINC revision
`37c8fb16171114b39e2283ff4b9e351fa2d5975b`. See the release-policy chapter for
the package, SemVer, generation-domain, algorithm, MSRV, and tag rules.
