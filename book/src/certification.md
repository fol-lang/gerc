# H5 Certification Matrix

`tests/pipeline_support/matrix.rs` is executable inventory. The tables below
mirror it: no row means implied support. Production certification accepts only
the evidence returned by `linc::native::NativeAnalyzer::certify`.

## Types

| Construct | State | Owner | Stable result |
| --- | --- | --- | --- |
| void | supported-and-tested | PARC/LINC/GERC | — |
| _Bool | supported-and-tested | PARC/LINC/GERC | — |
| plain char | supported-and-tested | PARC/LINC/GERC | — |
| signed char | supported-and-tested | PARC/LINC/GERC | — |
| unsigned char | supported-and-tested | PARC/LINC/GERC | — |
| signed short | supported-and-tested | PARC/LINC/GERC | — |
| unsigned short | supported-and-tested | PARC/LINC/GERC | — |
| signed int | supported-and-tested | PARC/LINC/GERC | — |
| unsigned int | supported-and-tested | PARC/LINC/GERC | — |
| signed long | supported-and-tested | PARC/LINC/GERC | — |
| unsigned long | supported-and-tested | PARC/LINC/GERC | — |
| signed long long | supported-and-tested | PARC/LINC/GERC | — |
| unsigned long long | supported-and-tested | PARC/LINC/GERC | — |
| float | supported-and-tested | PARC/LINC/GERC | — |
| double | supported-and-tested | PARC/LINC/GERC | — |
| raw and nullable pointers | supported-and-tested | PARC/LINC/GERC | — |
| nonzero fixed arrays | supported-and-tested | PARC/LINC/GERC | — |
| complete records with measured layout | supported-and-tested | PARC/LINC/GERC | — |
| complete unions with measured layout | supported-and-tested | PARC/LINC/GERC | — |
| incomplete records behind pointers | supported-and-tested | PARC/LINC/GERC | — |
| C enums as integer aliases and constants | supported-and-tested | PARC/LINC/GERC | — |
| C calling-convention routines | supported-and-tested | PARC/LINC/GERC | — |
| C calling-convention callbacks | supported-and-tested | PARC/LINC/GERC | — |
| long double and extended floating types | explicitly-rejected | LINC | LINC-E3014 |
| complex floating types | explicitly-rejected | LINC | LINC-E3014 |
| compiler vector types | explicitly-rejected | PARC | CompletionBlocker::Unsupported |
| _BitInt | explicitly-rejected | PARC | PARC-P1107 |
| 128-bit C integers | explicitly-rejected | LINC | LINC-E3014 |
| by-value opaque or incomplete records | explicitly-rejected | PARC | CompletionBlocker::IncompleteRecord |
| bitfield layouts | explicitly-rejected | GERC | GERC-E2002 |
| unsupported calling conventions | explicitly-rejected | LINC | LINC-E3050 |
| variadic or unspecified callables | explicitly-rejected | LINC | LINC-E3050 |
| C++ types and ABI | explicitly-rejected | PARC | PARC-P0002 |
| thread-local globals | explicitly-rejected | GERC | GERC-E2002 |
| function-like macros | experimental-not-for-FOL | GERC | GERC-N3000 |
| string macros | experimental-not-for-FOL | GERC | GERC-N3000 |

## Platforms

| Platform | State | Owner | Stable result |
| --- | --- | --- | --- |
| x86_64-unknown-linux-gnu / ELF / explicit GCC | supported-and-tested | H5 pipeline | — |
| x86_64-unknown-linux-gnu / ELF / Clang differential | experimental-not-for-FOL | H5 differential lane | optional full typed value roundtrip |
| x86_64-unknown-linux-musl | explicitly-rejected | H5 gate | not certified |
| second Linux architecture | explicitly-rejected | H5 gate | not certified |
| aarch64-apple-darwin | explicitly-rejected | H5 gate | not certified |
| x86_64-pc-windows-msvc and MinGW | explicitly-rejected | H5 gate | not certified |

## Provider and provenance failures

| Failure | State | Owner | Stable result |
| --- | --- | --- | --- |
| missing provider symbol | explicitly-rejected | LINC | LINC-E3040 |
| hidden provider symbol | explicitly-rejected | LINC | LINC-E3040 |
| weak provider symbol | explicitly-rejected | LINC | LINC-E3040 |
| duplicate symbols in one provider | explicitly-rejected | LINC | LINC-E3040 |
| ambiguous symbols across providers | explicitly-rejected | LINC | LINC-E3040 |
| wrong-target provider | explicitly-rejected | LINC | LINC-E3007 |
| partial external-preprocessor source | explicitly-rejected | PARC | PARC-P0001 |
| stale source-bound evidence | explicitly-rejected | GERC | GERC-E1100 |

The required lane is explicit GCC on x86-64 GNU/Linux ELF. Explicit Clang is
optional and, when supplied, repeats the complete typed scan, certification,
generation, provider build, Rust link, and value roundtrip with distinct
target-bound evidence. No evidence is shared between compiler lanes.
