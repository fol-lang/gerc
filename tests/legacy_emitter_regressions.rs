mod common;

use gerc::{emit_source, generate, GercConfig, GercInput};
use linc::ir::{
    BindingItem, BindingPackage, BindingType, CallingConvention, EnumBinding, EnumVariant,
    FieldBinding, FunctionBinding, ParameterBinding, RecordBinding, RecordKind, TypeAliasBinding,
};

fn generate_source(pkg: BindingPackage) -> String {
    let input = GercInput::from_source_package(common::from_binding_package(&pkg));
    let output = generate(&input, &GercConfig::new("legacy_sys")).unwrap();
    emit_source(&output.projection)
}

#[test]
fn legacy_unnamed_record_fields_get_deterministic_fallback_names() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("legacy_buf".into()),
        fields: Some(vec![
            FieldBinding {
                name: None,
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("len".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("pub struct legacy_buf"));
    assert!(source.contains("pub __field0: core::ffi::c_int,"));
    assert!(source.contains("pub len: core::ffi::c_uint,"));
}

#[test]
fn legacy_memcpy_style_void_pointer_signatures_stay_explicit() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "memcpy".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding {
                name: Some("dest".into()),
                ty: BindingType::ptr(BindingType::Void),
            },
            ParameterBinding {
                name: Some("src".into()),
                ty: BindingType::const_ptr(BindingType::Void),
            },
            ParameterBinding {
                name: Some("n".into()),
                ty: BindingType::ULong,
            },
        ],
        return_type: BindingType::ptr(BindingType::Void),
        variadic: false,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("pub fn memcpy("));
    assert!(source.contains("dest: *mut core::ffi::c_void"));
    assert!(source.contains("src: *const core::ffi::c_void"));
    assert!(source.contains("n: core::ffi::c_ulong"));
    assert!(source.contains("-> *mut core::ffi::c_void;"));
}

#[test]
fn legacy_const_void_pointer_returns_stay_const_in_emission() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "find".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::const_ptr(BindingType::Void),
        variadic: false,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("pub fn find() -> *const core::ffi::c_void;"));
}

#[test]
fn named_opaque_types_stay_named_instead_of_becoming_erased_comments() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("FILE".into()),
        fields: None,
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "borrow_file".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![ParameterBinding {
            name: Some("file".into()),
            ty: BindingType::ptr(BindingType::Opaque("FILE".into())),
        }],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("pub struct FILE { _opaque: [u8; 0] }"));
    assert!(source.contains("pub fn borrow_file(file: *mut FILE) -> core::ffi::c_int;"));
    assert!(!source.contains("opaque: FILE"));
}

#[test]
fn flexible_array_members_emit_as_zero_length_arrays() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("legacy_packet".into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("len".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: None,
            },
            FieldBinding {
                name: Some("data".into()),
                ty: BindingType::Array(Box::new(BindingType::UChar), None),
                bit_width: None,
                layout: None,
            },
        ]),
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("pub data: [core::ffi::c_uchar; 0],"));
}

#[test]
fn long_double_falls_back_to_explicit_unknown_marker() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "ld_func".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![ParameterBinding {
            name: Some("x".into()),
            ty: BindingType::LongDouble,
        }],
        return_type: BindingType::LongDouble,
        variadic: false,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("x: /* unknown: c_longdouble */ ()"));
    assert!(source.contains("-> /* unknown: c_longdouble */ ();"));
}

#[test]
fn enums_emit_as_repr_enums_not_typedef_plus_const_blocks() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: Some("color".into()),
        variants: vec![
            EnumVariant {
                name: "RED".into(),
                value: Some(0),
            },
            EnumVariant {
                name: "GREEN".into(),
                value: Some(1),
            },
        ],
        representation: None,
        abi_confidence: None,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains("#[repr(c_int)]"));
    assert!(source.contains("pub enum color"));
    assert!(source.contains("RED = 0,"));
    assert!(source.contains("GREEN = 1,"));
    assert!(!source.contains("pub type color ="));
    assert!(!source.contains("pub const RED: color"));
}

#[test]
fn function_pointer_aliases_emit_option_wrapped_signatures() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "handler_t".into(),
        target: BindingType::FunctionPointer {
            return_type: Box::new(BindingType::Void),
            parameters: vec![BindingType::Int],
            variadic: false,
        },
        canonical_resolution: None,
        abi_confidence: None,
        source_offset: None,
    }));

    let source = generate_source(pkg);
    assert!(source.contains(
        "pub type handler_t = Option<unsafe extern \"C\" fn(core::ffi::c_int)>;"
    ));
    assert!(!source.contains("pub type handler_t = unsafe extern \"C\" fn("));
}
