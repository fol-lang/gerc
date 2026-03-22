mod common;

use gerc::config::GercConfig;
use gerc::contract::generate;
use gerc::emit::emit_source;
use gerc::intake::GercInput;
use linc::ir::{
    BindingItem, BindingPackage, BindingType, CallingConvention, FieldBinding, FunctionBinding,
    ParameterBinding, RecordBinding, RecordKind, TypeAliasBinding,
};

fn input_from_binding(pkg: BindingPackage) -> GercInput {
    GercInput::from_source_package(common::from_binding_package(&pkg))
}

#[test]
fn incomplete_support_pointer_only_handle_patterns_lower_source_only() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("sqlite3".into()),
        fields: None,
        source_offset: None,
        representation: None,
        abi_confidence: None,
    }));
    pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
        name: "sqlite3_t".into(),
        target: BindingType::RecordRef("sqlite3".into()),
        source_offset: None,
        canonical_resolution: None,
        abi_confidence: None,
    }));
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "sqlite3_open".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding {
                name: Some("path".into()),
                ty: BindingType::const_ptr(BindingType::Char),
            },
            ParameterBinding {
                name: Some("db".into()),
                ty: BindingType::ptr(BindingType::ptr(BindingType::TypedefRef("sqlite3_t".into()))),
            },
        ],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));

    let output = generate(&input_from_binding(pkg), &GercConfig::new("sqlite3_sys")).unwrap();
    let source = emit_source(&output.projection);

    assert!(source.contains("pub struct sqlite3 { _opaque: [u8; 0] }"));
    assert!(source.contains("pub type sqlite3_t = sqlite3;"));
    assert!(source.contains("pub fn sqlite3_open"));
    assert!(source.contains("*mut *mut sqlite3_t"));
}

#[test]
fn incomplete_support_pointer_to_opaque_fields_remain_projectable() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("queue_holder".into()),
        fields: Some(vec![FieldBinding {
            name: Some("next".into()),
            ty: BindingType::ptr(BindingType::Opaque("dispatch_queue".into())),
            bit_width: None,
            layout: None,
        }]),
        source_offset: None,
        representation: None,
        abi_confidence: None,
    }));

    let output = generate(&input_from_binding(pkg), &GercConfig::new("queue_sys")).unwrap();
    let source = emit_source(&output.projection);

    assert!(source.contains("pub struct queue_holder"));
    assert!(source.contains("pub next: *mut dispatch_queue"));
    assert!(source.contains("pub struct dispatch_queue { _opaque: [u8; 0] }"));
}

#[test]
fn incomplete_support_direct_pointers_to_anonymous_records_lower_as_opaque_ptrs() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "borrow_payload".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding {
                name: Some("payload".into()),
                ty: BindingType::ptr(BindingType::RecordRef("<anonymous>".into())),
            },
            ParameterBinding {
                name: Some("out_payload".into()),
                ty: BindingType::ptr(BindingType::ptr(BindingType::RecordRef(
                    "<anonymous>".into(),
                ))),
            },
        ],
        return_type: BindingType::const_ptr(BindingType::EnumRef("<anonymous>".into())),
        variadic: false,
        source_offset: None,
    }));

    let output = generate(&input_from_binding(pkg), &GercConfig::new("anon_ptr_sys")).unwrap();
    let source = emit_source(&output.projection);

    assert!(source.contains("pub fn borrow_payload"));
    assert!(source.contains("payload: *mut core::ffi::c_void"));
    assert!(source.contains("out_payload: *mut *mut core::ffi::c_void"));
    assert!(source.contains(") -> *const core::ffi::c_void;"));
}
