mod common;

use gerc::config::GercConfig;
use gerc::contract::generate;
use gerc::emit::emit_source;
use gerc::intake::GercInput;
use linc::ir::{
    BindingItem, BindingPackage, BindingType, CallingConvention, FunctionBinding, ParameterBinding,
};

fn input_from_binding(pkg: BindingPackage) -> GercInput {
    GercInput::from_source_package(common::from_binding_package(&pkg))
}

#[test]
fn placeholder_hardening_escapes_keyword_named_placeholders_everywhere() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "wire_handles".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![
            ParameterBinding {
                name: Some("type".into()),
                ty: BindingType::ptr(BindingType::Opaque("type".into())),
            },
            ParameterBinding {
                name: Some("match".into()),
                ty: BindingType::const_ptr(BindingType::Opaque("match".into())),
            },
            ParameterBinding {
                name: Some("typeof".into()),
                ty: BindingType::ptr(BindingType::Opaque("typeof".into())),
            },
        ],
        return_type: BindingType::Opaque("Self".into()),
        variadic: false,
        source_offset: None,
    }));

    let output = generate(&input_from_binding(pkg), &GercConfig::new("keyword_sys")).unwrap();
    let source = emit_source(&output.projection);

    assert!(source.contains("pub struct r#type { _opaque: [u8; 0] }"));
    assert!(source.contains("pub struct r#match { _opaque: [u8; 0] }"));
    assert!(source.contains("pub struct r#typeof { _opaque: [u8; 0] }"));
    assert!(source.contains("pub struct r#Self { _opaque: [u8; 0] }"));
    assert!(source.contains("r#type: *mut r#type"));
    assert!(source.contains("r#match: *const r#match"));
    assert!(source.contains("r#typeof: *mut r#typeof"));
    assert!(source.contains("-> r#Self"));
}
