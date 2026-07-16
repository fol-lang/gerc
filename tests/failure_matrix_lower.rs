use gerc::c::{
    AbiConfidence, BindingItem, BindingPackage, BindingType, EnumBinding, EnumRepresentation,
    EnumVariant, FieldBinding, RecordBinding, RecordKind, RecordRepresentation, UnsupportedItem,
};
use gerc::ir::RustItem;
use gerc::lower::lower_package;

#[test]
fn failure_matrix_lower_anonymous_and_unsupported_items_stay_explicit() {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: None,
        fields: Some(vec![FieldBinding {
            name: Some("value".into()),
            ty: BindingType::Int,
            bit_width: None,
            layout: None,
        }]),
        source_offset: None,
        representation: Some(RecordRepresentation {
            size: Some(4),
            align: Some(4),
            completeness: Some("Complete".into()),
        }),
        abi_confidence: Some(AbiConfidence::RepresentationProbed),
    }));
    pkg.items.push(BindingItem::Enum(EnumBinding {
        name: None,
        variants: vec![EnumVariant {
            name: "ANON_VALUE".into(),
            value: Some(1),
        }],
        source_offset: None,
        representation: Some(EnumRepresentation {
            underlying_size: Some(4),
            is_signed: Some(true),
        }),
        abi_confidence: Some(AbiConfidence::RepresentationProbed),
    }));
    pkg.items.push(BindingItem::Unsupported(UnsupportedItem {
        name: Some("bad_decl".into()),
        reason: "bitfield".into(),
        source_offset: None,
    }));

    let (projection, diagnostics) = lower_package(&pkg);

    assert_eq!(projection.items.len(), 3);
    assert_eq!(diagnostics.len(), 3);
    assert!(projection
        .items
        .iter()
        .all(|item| matches!(item, RustItem::Unsupported(_))));
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("anonymous record")));
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("anonymous enum")));
    assert!(diagnostics
        .iter()
        .any(|d| d.message.contains("unsupported")));
}
