use gec::{emit_source, generate, GecConfig, GecInput};
use linc::{
    BindingItem, BindingPackage, BindingType, FieldBinding, RecordBinding, RecordKind,
};

fn generate_source(pkg: BindingPackage) -> String {
    let output = generate(&GecInput::from_package(pkg), &GecConfig::new("legacy_sys")).unwrap();
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
