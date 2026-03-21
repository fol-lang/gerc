# Code Generation

`gec` owns Rust FFI emission in the split `PARC -> LINC -> GERC` pipeline.
Useful legacy emitter behavior is rehomed here; obsolete output shapes are not
kept just for compatibility.

## Pipeline stages

Code generation in `gec` proceeds through several stages:

### 1. Safety gating (`gate`)

Each item in the source-derived lowering package is evaluated against generation rules:

| Rule | Effect |
|---|---|
| Bitfield records | Rejected — no safe `repr(C)` representation |
| Anonymous records | Rejected — Rust requires named types |
| Anonymous enums | Rejected — Rust requires named types |
| Incomplete/opaque fields | Rejected — cannot determine layout |
| `Unsupported` items | Rejected — explicitly unsupported upstream |

Rejected items produce diagnostics in the output but no Rust code.

### 2. Type mapping (`typemap`)

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

Accepted items are lowered from `gec`'s C-side model to `gec`'s internal IR:

- **Functions** → `RustFunction` (name, parameters, return type, variadic flag)
- **Structs** → `RustRecord` with `RustRecordKind::Struct`
- **Unions** → `RustRecord` with `RustRecordKind::Union`
- **Opaque records** → `RustRecord` with `is_opaque: true`
- **Enums** → `RustEnum` with variants and repr selection (`c_int` or `c_uint`)
- **Type aliases** → `RustTypeAlias`
- **Variables** → `RustStatic`
- **Integer macros** → `RustConstant`

### 4. Source emission (`emit`)

The IR is rendered into Rust source in a deterministic order:

1. Constants (`pub const NAME: TYPE = VALUE;`)
2. Type aliases (`pub type NAME = TARGET;`)
3. Enums (`#[repr(REPR)] pub enum NAME { ... }`)
4. Records — structs (`#[repr(C)] pub struct NAME { ... }`), unions, and
   opaque structs (`pub struct NAME { _opaque: [u8; 0] }`)
5. `extern "C"` block containing function declarations and statics

This ordering is stable and deterministic: the same input always produces
the same output.

## Intentional canonicalizations

Some emitted forms intentionally differ from older `linc` Rust output:

- named opaque handles emit as named zero-sized structs, not erased comments
- enums emit as `#[repr(...)] pub enum NAME { ... }`, not typedef-plus-const groups
- function-pointer aliases emit as `Option<unsafe extern "C" fn(...)>`

These are the supported `gec` forms going forward.

## Link metadata (`linkgen`)

Link evidence is lowered into Cargo-compatible `build.rs` directives:

- `cargo:rustc-link-lib=NAME` — link a library
- `cargo:rustc-link-lib=static=NAME` — link a static library
- `cargo:rustc-link-lib=dylib=NAME` — link a dynamic library
- `cargo:rustc-link-lib=framework=NAME` — link a macOS framework
- `cargo:rustc-link-search=native=PATH` — add a library search path

Platform filtering is supported via `cfg!()` guards when platform constraints
are specified.

When a resolved `ResolvedLinkPlan` is attached to `GecInput`, `gec` prefers
that evidence over the source-declared raw link surface.
