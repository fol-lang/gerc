//! Internal Rust projection IR.
//!
//! This module defines the intermediate representation that sits between
//! `linc` intake and Rust code emission.  Items in this IR describe *what
//! Rust code should be generated*, not C declarations.
//!
//! The IR is intentionally separate from both `linc::ir` (the C-side model)
//! and the final emitted source text.

use serde::{Deserialize, Serialize};

/// A complete Rust projection — the collection of all items to be emitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RustProjection {
    pub items: Vec<RustItem>,
    /// Module organization: maps module names to item indices.
    pub modules: Vec<RustModule>,
    /// Native link requirements for the generated crate.
    pub link_requirements: Vec<RustLinkRequirement>,
    /// Provenance notes attached to projected items.
    pub notes: Vec<ProjectionNote>,
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

/// A module grouping in the projected crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustModule {
    pub name: String,
    /// Indices into `RustProjection::items`.
    pub item_indices: Vec<usize>,
}

/// A native link requirement for the generated crate's `build.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustLinkRequirement {
    pub kind: RustLinkKind,
    pub name: String,
    /// Optional search path for this requirement.
    pub search_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustLinkKind {
    DynamicLibrary,
    StaticLibrary,
    Framework,
}

/// Provenance or diagnostic note attached to a projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionNote {
    pub kind: NoteKind,
    pub message: String,
    pub item_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteKind {
    /// The item was projected successfully.
    Projected,
    /// The item was skipped as unsupported.
    Unsupported,
    /// The item had partial information.
    Partial,
    /// Provenance from the original C source.
    Provenance,
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
    fn roundtrip_json_function() {
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

    #[test]
    fn roundtrip_json_record() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Record(RustRecord {
            name: "Point".into(),
            kind: RustRecordKind::Struct,
            fields: vec![
                RustField {
                    name: "x".into(),
                    ty: RustType::CInt,
                },
                RustField {
                    name: "y".into(),
                    ty: RustType::CInt,
                },
            ],
            is_opaque: false,
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }

    #[test]
    fn roundtrip_json_enum() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Enum(RustEnum {
            name: "Color".into(),
            variants: vec![
                RustEnumVariant {
                    name: "Red".into(),
                    value: Some(0),
                },
                RustEnumVariant {
                    name: "Green".into(),
                    value: Some(1),
                },
            ],
            repr: "c_int".into(),
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }

    #[test]
    fn roundtrip_json_type_alias() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::TypeAlias(RustTypeAlias {
            name: "size_t".into(),
            target: RustType::CULong,
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }

    #[test]
    fn roundtrip_json_constant() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Constant(RustConstant {
            name: "API_VERSION".into(),
            ty: RustType::CInt,
            value: "3".into(),
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }

    #[test]
    fn roundtrip_json_static() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Static(RustStatic {
            name: "errno".into(),
            ty: RustType::CInt,
            mutable: true,
            doc: None,
        }));
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 1);
    }

    #[test]
    fn link_requirements() {
        let mut proj = RustProjection::new();
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: None,
        });
        let json = serde_json::to_string(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.link_requirements.len(), 1);
        assert_eq!(
            proj2.link_requirements[0].kind,
            RustLinkKind::DynamicLibrary
        );
    }

    #[test]
    fn module_organization() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "foo".into(),
            parameters: vec![],
            return_type: RustType::Void,
            variadic: false,
            doc: None,
        }));
        proj.modules.push(RustModule {
            name: "ffi".into(),
            item_indices: vec![0],
        });
        assert_eq!(proj.modules[0].item_indices, vec![0]);
    }

    #[test]
    fn projection_notes() {
        let mut proj = RustProjection::new();
        proj.notes.push(ProjectionNote {
            kind: NoteKind::Unsupported,
            message: "bitfield not projected".into(),
            item_name: Some("flags".into()),
        });
        assert_eq!(proj.notes[0].kind, NoteKind::Unsupported);
    }

    #[test]
    fn fn_pointer_type() {
        let fptr = RustType::FnPointer {
            params: vec![RustType::CInt],
            ret: Box::new(RustType::Void),
            variadic: false,
        };
        assert_ne!(fptr, RustType::CInt);
    }

    #[test]
    fn array_type() {
        let arr = RustType::Array {
            element: Box::new(RustType::CInt),
            len: Some(10),
        };
        if let RustType::Array { len, .. } = &arr {
            assert_eq!(*len, Some(10));
        } else {
            panic!("expected Array");
        }
    }

    #[test]
    fn opaque_ptr_type() {
        let p = RustType::OpaquePtr { is_const: false };
        assert_ne!(p, RustType::OpaquePtr { is_const: true });
    }

    #[test]
    fn full_projection_roundtrip() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "init".into(),
            parameters: vec![RustParameter {
                name: "cfg".into(),
                ty: RustType::Pointer {
                    pointee: Box::new(RustType::Named("Config".into())),
                    is_const: false,
                },
            }],
            return_type: RustType::CInt,
            variadic: false,
            doc: Some("Initialize the library".into()),
        }));
        proj.items.push(RustItem::Record(RustRecord {
            name: "Config".into(),
            kind: RustRecordKind::Struct,
            fields: vec![RustField {
                name: "flags".into(),
                ty: RustType::CUInt,
            }],
            is_opaque: false,
            doc: None,
        }));
        proj.modules.push(RustModule {
            name: "ffi".into(),
            item_indices: vec![0, 1],
        });
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::StaticLibrary,
            name: "mylib".into(),
            search_path: Some("/usr/local/lib".into()),
        });
        proj.notes.push(ProjectionNote {
            kind: NoteKind::Projected,
            message: "projected init".into(),
            item_name: Some("init".into()),
        });

        let json = serde_json::to_string_pretty(&proj).unwrap();
        let proj2: RustProjection = serde_json::from_str(&json).unwrap();
        assert_eq!(proj2.len(), 2);
        assert_eq!(proj2.modules.len(), 1);
        assert_eq!(proj2.link_requirements.len(), 1);
        assert_eq!(proj2.notes.len(), 1);
    }
}
