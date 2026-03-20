//! Type mapping from `bic::BindingType` to `gec::ir::RustType`.
//!
//! This module is the core C→Rust type projection.  It maps primitive types,
//! pointers, arrays, function pointers, typedefs, and opaque handles into
//! their Rust FFI equivalents.

use bic::BindingType;

use crate::ir::RustType;

/// Map a `bic::BindingType` to a `gec::ir::RustType`.
pub fn map_type(ty: &BindingType) -> RustType {
    match ty {
        // 4.1: primitive integer and float types
        BindingType::Void => RustType::Void,
        BindingType::Bool => RustType::Bool,
        BindingType::Char => RustType::CChar,
        BindingType::SChar => RustType::CSChar,
        BindingType::UChar => RustType::CUChar,
        BindingType::Short => RustType::CShort,
        BindingType::UShort => RustType::CUShort,
        BindingType::Int => RustType::CInt,
        BindingType::UInt => RustType::CUInt,
        BindingType::Long => RustType::CLong,
        BindingType::ULong => RustType::CULong,
        BindingType::LongLong => RustType::CLongLong,
        BindingType::ULongLong => RustType::CULongLong,
        BindingType::Float => RustType::F32,
        BindingType::Double => RustType::F64,
        BindingType::LongDouble => RustType::Unknown("c_longdouble".into()),

        // 4.2 & 4.3: pointers, mutability, void*, const void*
        BindingType::Pointer {
            pointee,
            const_pointee,
            ..
        } => {
            if pointee.is_void() {
                // 4.3: void* / const void* → OpaquePtr
                RustType::OpaquePtr {
                    is_const: *const_pointee,
                }
            } else {
                // 4.2: regular pointers
                RustType::Pointer {
                    pointee: Box::new(map_type(pointee)),
                    is_const: *const_pointee,
                }
            }
        }

        // 4.4 & 4.5: arrays and flexible-array tails
        BindingType::Array(element, len) => RustType::Array {
            element: Box::new(map_type(element)),
            len: *len,
        },

        // 4.6: function pointer types
        BindingType::FunctionPointer {
            return_type,
            parameters,
            variadic,
        } => RustType::FnPointer {
            params: parameters.iter().map(map_type).collect(),
            ret: Box::new(map_type(return_type)),
            variadic: *variadic,
        },

        // 4.7: opaque record handles
        BindingType::Opaque(name) => RustType::Named(name.clone()),

        // 4.8: typedef chains and canonical aliases
        BindingType::TypedefRef(name) => RustType::Named(name.clone()),
        BindingType::RecordRef(name) => RustType::Named(name.clone()),
        BindingType::EnumRef(name) => RustType::Named(name.clone()),

        // Qualified types: strip qualifiers and map inner type
        BindingType::Qualified { ty, .. } => map_type(ty),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bic::{BindingType, TypeQualifiers};

    // 4.1: primitive types
    #[test]
    fn map_void() {
        assert_eq!(map_type(&BindingType::Void), RustType::Void);
    }

    #[test]
    fn map_bool() {
        assert_eq!(map_type(&BindingType::Bool), RustType::Bool);
    }

    #[test]
    fn map_char() {
        assert_eq!(map_type(&BindingType::Char), RustType::CChar);
    }

    #[test]
    fn map_int() {
        assert_eq!(map_type(&BindingType::Int), RustType::CInt);
    }

    #[test]
    fn map_uint() {
        assert_eq!(map_type(&BindingType::UInt), RustType::CUInt);
    }

    #[test]
    fn map_long() {
        assert_eq!(map_type(&BindingType::Long), RustType::CLong);
    }

    #[test]
    fn map_ulong() {
        assert_eq!(map_type(&BindingType::ULong), RustType::CULong);
    }

    #[test]
    fn map_longlong() {
        assert_eq!(map_type(&BindingType::LongLong), RustType::CLongLong);
    }

    #[test]
    fn map_float() {
        assert_eq!(map_type(&BindingType::Float), RustType::F32);
    }

    #[test]
    fn map_double() {
        assert_eq!(map_type(&BindingType::Double), RustType::F64);
    }

    #[test]
    fn map_long_double_unknown() {
        match map_type(&BindingType::LongDouble) {
            RustType::Unknown(s) => assert_eq!(s, "c_longdouble"),
            other => panic!("expected Unknown, got {:?}", other),
        }
    }

    // 4.2: pointers and mutability
    #[test]
    fn map_mut_pointer() {
        let ty = BindingType::ptr(BindingType::Int);
        match map_type(&ty) {
            RustType::Pointer { pointee, is_const } => {
                assert!(!is_const);
                assert_eq!(*pointee, RustType::CInt);
            }
            other => panic!("expected Pointer, got {:?}", other),
        }
    }

    #[test]
    fn map_const_pointer() {
        let ty = BindingType::const_ptr(BindingType::Int);
        match map_type(&ty) {
            RustType::Pointer { is_const, .. } => assert!(is_const),
            other => panic!("expected Pointer, got {:?}", other),
        }
    }

    // 4.3: void* and const void*
    #[test]
    fn map_void_ptr() {
        let ty = BindingType::ptr(BindingType::Void);
        match map_type(&ty) {
            RustType::OpaquePtr { is_const } => assert!(!is_const),
            other => panic!("expected OpaquePtr, got {:?}", other),
        }
    }

    #[test]
    fn map_const_void_ptr() {
        let ty = BindingType::const_ptr(BindingType::Void);
        match map_type(&ty) {
            RustType::OpaquePtr { is_const } => assert!(is_const),
            other => panic!("expected OpaquePtr, got {:?}", other),
        }
    }

    // 4.4: arrays
    #[test]
    fn map_fixed_array() {
        let ty = BindingType::Array(Box::new(BindingType::Int), Some(10));
        match map_type(&ty) {
            RustType::Array { element, len } => {
                assert_eq!(*element, RustType::CInt);
                assert_eq!(len, Some(10));
            }
            other => panic!("expected Array, got {:?}", other),
        }
    }

    // 4.5: flexible-array (unsized)
    #[test]
    fn map_flexible_array() {
        let ty = BindingType::Array(Box::new(BindingType::Int), None);
        match map_type(&ty) {
            RustType::Array { len, .. } => assert_eq!(len, None),
            other => panic!("expected Array, got {:?}", other),
        }
    }

    #[test]
    fn map_zero_length_array() {
        let ty = BindingType::Array(Box::new(BindingType::Int), Some(0));
        match map_type(&ty) {
            RustType::Array { len, .. } => assert_eq!(len, Some(0)),
            other => panic!("expected Array, got {:?}", other),
        }
    }

    // 4.6: function pointers
    #[test]
    fn map_function_pointer() {
        let ty = BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Void),
            parameters: vec![BindingType::Int, BindingType::Float],
            variadic: false,
        };
        match map_type(&ty) {
            RustType::FnPointer {
                params,
                ret,
                variadic,
            } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], RustType::CInt);
                assert_eq!(params[1], RustType::F32);
                assert_eq!(*ret, RustType::Void);
                assert!(!variadic);
            }
            other => panic!("expected FnPointer, got {:?}", other),
        }
    }

    #[test]
    fn map_variadic_function_pointer() {
        let ty = BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Int),
            parameters: vec![BindingType::Int],
            variadic: true,
        };
        match map_type(&ty) {
            RustType::FnPointer { variadic, .. } => assert!(variadic),
            other => panic!("expected FnPointer, got {:?}", other),
        }
    }

    // 4.7: opaque record handles
    #[test]
    fn map_opaque() {
        let ty = BindingType::Opaque("FILE".into());
        assert_eq!(map_type(&ty), RustType::Named("FILE".into()));
    }

    // 4.8: typedef chains
    #[test]
    fn map_typedef_ref() {
        let ty = BindingType::TypedefRef("size_t".into());
        assert_eq!(map_type(&ty), RustType::Named("size_t".into()));
    }

    #[test]
    fn map_record_ref() {
        let ty = BindingType::RecordRef("point".into());
        assert_eq!(map_type(&ty), RustType::Named("point".into()));
    }

    #[test]
    fn map_enum_ref() {
        let ty = BindingType::EnumRef("color".into());
        assert_eq!(map_type(&ty), RustType::Named("color".into()));
    }

    // Qualified types
    #[test]
    fn map_const_qualified() {
        let ty = BindingType::Qualified {
            ty: Box::new(BindingType::Int),
            qualifiers: TypeQualifiers {
                is_const: true,
                ..Default::default()
            },
        };
        assert_eq!(map_type(&ty), RustType::CInt);
    }

    // 4.9: deep pointer chain
    #[test]
    fn map_deep_pointer_chain() {
        // int ***
        let ty = BindingType::ptr(BindingType::ptr(BindingType::ptr(BindingType::Int)));
        let mapped = map_type(&ty);
        // Should be *mut *mut *mut c_int
        match mapped {
            RustType::Pointer { pointee, .. } => match *pointee {
                RustType::Pointer { pointee, .. } => match *pointee {
                    RustType::Pointer { pointee, .. } => {
                        assert_eq!(*pointee, RustType::CInt);
                    }
                    other => panic!("expected inner Pointer, got {:?}", other),
                },
                other => panic!("expected middle Pointer, got {:?}", other),
            },
            other => panic!("expected outer Pointer, got {:?}", other),
        }
    }

    // 4.10: regression suite
    #[test]
    fn map_all_primitives() {
        let cases = vec![
            (BindingType::Void, RustType::Void),
            (BindingType::Bool, RustType::Bool),
            (BindingType::Char, RustType::CChar),
            (BindingType::SChar, RustType::CSChar),
            (BindingType::UChar, RustType::CUChar),
            (BindingType::Short, RustType::CShort),
            (BindingType::UShort, RustType::CUShort),
            (BindingType::Int, RustType::CInt),
            (BindingType::UInt, RustType::CUInt),
            (BindingType::Long, RustType::CLong),
            (BindingType::ULong, RustType::CULong),
            (BindingType::LongLong, RustType::CLongLong),
            (BindingType::ULongLong, RustType::CULongLong),
            (BindingType::Float, RustType::F32),
            (BindingType::Double, RustType::F64),
        ];
        for (bic_ty, expected) in cases {
            assert_eq!(
                map_type(&bic_ty),
                expected,
                "failed for {:?}",
                bic_ty
            );
        }
    }
}
