use gec::ir::{RustFunction, RustItem, RustProjection, RustType};

#[test]
fn root_reexports_source_emit_helpers() {
    let mut projection = RustProjection::new();
    projection.items.push(RustItem::Function(RustFunction {
        name: "demo_init".into(),
        parameters: vec![],
        return_type: RustType::CInt,
        variadic: false,
        doc: None,
    }));

    let emitted = gec::emit_source(&projection);
    assert!(emitted.contains("pub fn demo_init"));
    assert_eq!(
        gec::emit_type(&RustType::Pointer {
            pointee: Box::new(RustType::CChar),
            is_const: false,
        }),
        "*mut core::ffi::c_char"
    );
}
