# Verification

The repository Makefile is the validation interface:

```sh
make fmt-check
make lint
make check-features
make test
make test-contract
make test-package
make test-system
make docs-check
```

The preservation corpus proves an ABI-supported record/enum subset, exact
fingerprint propagation, deterministic generated files, and ordered repeated
native link atoms. The same typed corpus also proves that its Win64 function is
rejected on the Linux target rather than emitted under a guessed ABI.

Package validation extracts PARC, then LINC, then GERC and builds a scratch
consumer using only `[patch.crates-io]` entries for the extracted packages.
