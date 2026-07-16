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

Package validation extracts PARC, then LINC, then GERC and builds a scratch
consumer using only `[patch.crates-io]` entries for the extracted packages. The
consumer runs a nonzero typed generation test and checks the generated file and
ordered link plan.
