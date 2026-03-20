//! Internal Rust projection IR.
//!
//! This module defines the intermediate representation that sits between
//! `bic` intake and Rust code emission.  Items in this IR describe *what
//! Rust code should be generated*, not C declarations.
//!
//! The IR is intentionally separate from both `bic::ir` (the C-side model)
//! and the final emitted source text.

use serde::{Deserialize, Serialize};

/// A complete Rust projection — the collection of all items to be emitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RustProjection {
    pub items: Vec<RustItem>,
}

impl RustProjection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// One projected Rust item ready for emission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RustItem {
    Function(RustFunction),
    Record(RustRecord),
    Enum(RustEnum),
    TypeAlias(RustTypeAlias),
    Constant(RustConstant),
    Static(RustStatic),
    Unsupported(RustUnsupported),
}

/// A projected `extern "C"` function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustFunction {
    pub name: String,
    pub parameters: Vec<RustParameter>,
    pub return_type: RustType,
    pub variadic: bool,
    pub doc: Option<String>,
}

/// A function parameter in the Rust projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustParameter {
    pub name: String,
    pub ty: RustType,
}

/// A projected `#[repr(C)]` struct or union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRecord {
    pub name: String,
    pub kind: RustRecordKind,
    pub fields: Vec<RustField>,
    pub is_opaque: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustRecordKind {
    Struct,
    Union,
}

/// A field inside a projected record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustField {
    pub name: String,
    pub ty: RustType,
}

/// A projected enum with explicit representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustEnum {
    pub name: String,
    pub variants: Vec<RustEnumVariant>,
    pub repr: String,
    pub doc: Option<String>,
}

/// One enum variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustEnumVariant {
    pub name: String,
    pub value: Option<i128>,
}

/// A projected type alias (`pub type X = Y;`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustTypeAlias {
    pub name: String,
    pub target: RustType,
    pub doc: Option<String>,
}

/// A projected Rust constant (`pub const X: T = V;`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConstant {
    pub name: String,
    pub ty: RustType,
    pub value: String,
    pub doc: Option<String>,
}

/// A projected `extern` static variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustStatic {
    pub name: String,
    pub ty: RustType,
    pub mutable: bool,
    pub doc: Option<String>,
}

/// Placeholder for items that could not be projected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustUnsupported {
    pub name: Option<String>,
    pub reason: String,
}

/// Rust type as used in the projection IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustType {
    Void,
    Bool,
    CChar,
    CSChar,
    CUChar,
    CShort,
    CUShort,
    CInt,
    CUInt,
    CLong,
    CULong,
    CLongLong,
    CULongLong,
    F32,
    F64,
    Pointer {
        pointee: Box<RustType>,
        is_const: bool,
    },
    Array {
        element: Box<RustType>,
        len: Option<u64>,
    },
    FnPointer {
        params: Vec<RustType>,
        ret: Box<RustType>,
        variadic: bool,
    },
    Named(String),
    /// Opaque pointer (`*mut core::ffi::c_void` / `*const core::ffi::c_void`).
    OpaquePtr {
        is_const: bool,
    },
    /// A type that could not be mapped.
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_default_is_empty() {
        let proj = RustProjection::new();
        assert!(proj.is_empty());
        assert_eq!(proj.len(), 0);
    }

    #[test]
    fn projection_len() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Unsupported(RustUnsupported {
            name: Some("test".into()),
            reason: "testing".into(),
        }));
        assert_eq!(proj.len(), 1);
        assert!(!proj.is_empty());
    }

    #[test]
    fn rust_type_equality() {
        assert_eq!(RustType::CInt, RustType::CInt);
        assert_ne!(RustType::CInt, RustType::CUInt);
    }

    #[test]
    fn pointer_type() {
        let p = RustType::Pointer {
            pointee: Box::new(RustType::CInt),
            is_const: true,
        };
        if let RustType::Pointer { is_const, .. } = &p {
            assert!(is_const);
        } else {
            panic!("expected Pointer");
        }
    }

    #[test]
    fn roundtrip_json() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "foo".into(),
            parameters: vec![RustParameter {
                name: "x".into(),
                ty: RustType::CInt,
            }],
            return_type: RustType::Void,
            variadic: false,
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }
}
