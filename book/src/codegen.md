# Code Generation

`gerc` owns Rust FFI emission in the `PARC -> LINC -> GERC` toolchain.
Useful older emitter behavior is rehomed here when it still belongs in `gerc`;
obsolete output shapes are not kept alive just to preserve dead paths.

## Pipeline Stages

Code generation in `gerc` proceeds through several stages:

### 1. Safety Gating (`gate`)

Each item in the source-derived lowering package is evaluated against generation rules:

| Rule | Effect |
|---|---|
| Bitfield records | Rejected - no safe `repr(C)` representation |
| Packed non-bitfield records and unions | Allowed when representation evidence is explicit |
| Anonymous records | Rejected - Rust requires named types |
| Anonymous enums | Rejected - Rust requires named types |
| Incomplete or opaque fields | Rejected - cannot determine layout |
| `Unsupported` items | Rejected - explicitly unsupported upstream |

Pointer-only references to anonymous or otherwise unnameable declarations are a
special case: those degrade to opaque `*mut/*const core::ffi::c_void` instead
of forcing the whole surface to fail.

Rejected items produce diagnostics in the output but no Rust code.

### 2. Type Mapping (`typemap`)

C types are mapped to Rust FFI-safe equivalents:

| C type | Rust type |
|---|---|
| `void` | `()` (or omitted) |
| `_Bool` | `bool` |
| `int` | `core::ffi::c_int` |
| `unsigned int` | `core::ffi::c_uint` |
| `long` | `core::ffi::c_long` |
| `float` / `double` | `core::ffi::c_float` / `core::ffi::c_double` |
| `void*` | `*mut core::ffi::c_void` |
| `const void*` | `*const core::ffi::c_void` |
| `T*` | `*mut T` / `*const T` |
| `T[N]` | `[T; N]` |
| `T[]` (flexible array) | `[T; 0]` |
| `int (*)(int)` | `Option<unsafe extern "C" fn(c_int) -> c_int>` |
| `long double` | `Unknown` (not representable in Rust) |

### 3. Lowering (`lower`)

Accepted items are lowered from `gerc`'s C-side model to the internal IR:

- **Functions** -> `RustFunction`
- **Structs** -> `RustRecord` with `RustRecordKind::Struct`
- **Unions** -> `RustRecord` with `RustRecordKind::Union`
- **Opaque records** -> `RustRecord` with `is_opaque: true`
- **Enums** -> `RustEnum` with variants and repr selection
- **Type aliases** -> `RustTypeAlias`
- **Variables** -> `RustStatic`
- **Integer macros** -> `RustConstant`

### 4. Source Emission (`emit`)

The IR is rendered into Rust source in a deterministic order:

1. Constants
2. Type aliases
3. Enums
4. Records
5. `extern "C"` declarations

This ordering is stable and deterministic: the same input always produces the
same output.

## Intentional Canonicalizations

Some emitted forms intentionally differ from older `linc` Rust output:

- named opaque handles emit as named zero-sized structs, not erased comments
- enums emit as `#[repr(...)] pub enum NAME { ... }`, not typedef-plus-const groups
- function-pointer aliases emit as `Option<unsafe extern "C" fn(...)>`

These are the supported `gerc` forms going forward.

## Link Metadata (`linkgen`)

Link evidence is lowered into Cargo-compatible `build.rs` directives and plain
`rustc` argument files:

- `cargo:rustc-link-lib=NAME`
- `cargo:rustc-link-lib=static=NAME`
- `cargo:rustc-link-lib=dylib=NAME`
- `cargo:rustc-link-lib=framework=NAME`
- `cargo:rustc-link-search=native=PATH`

Platform filtering is supported via `cfg!()` guards when platform constraints
are specified.

When a resolved `ResolvedLinkPlan` is attached to `GercInput`, `gerc` prefers
that evidence over the source-declared raw link surface.
