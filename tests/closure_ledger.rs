use gerc::c::{
    AbiConfidence, BindingItem, BindingPackage, BindingType, EnumBinding, EnumRepresentation,
    EnumVariant, FieldBinding, RecordBinding, RecordKind, RecordRepresentation, UnsupportedItem,
};
use gerc::gate::{gate_package, GateDecision};

fn representative_rejection_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some("flags".into()),
        fields: Some(vec![FieldBinding {
            name: Some("bits".into()),
            ty: BindingType::UInt,
            bit_width: Some(3),
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
        name: Some("opaque_decl".into()),
        reason: "vendor extension".into(),
        source_offset: None,
    }));
    pkg
}

#[test]
fn closure_ledger_safety_rejections_stay_explicit() {
    let pkg = representative_rejection_package();
    let (decisions, diagnostics) = gate_package(&pkg, None);

    assert_eq!(decisions.len(), 4);
    assert!(matches!(&decisions[0], GateDecision::Reject(reason) if reason.contains("bitfields")));
    assert!(matches!(&decisions[1], GateDecision::Reject(reason) if reason.contains("anonymous")));
    assert!(matches!(&decisions[2], GateDecision::Reject(reason) if reason.contains("anonymous")));
    assert!(matches!(&decisions[3], GateDecision::Reject(reason) if reason.contains("unsupported declaration")));
    assert_eq!(diagnostics.len(), 4);
}

#[test]
fn closure_ledger_rejection_messages_remain_actionable() {
    let pkg = representative_rejection_package();
    let (_decisions, diagnostics) = gate_package(&pkg, None);
    let messages: Vec<_> = diagnostics.iter().map(|diag| diag.message.as_str()).collect();

    assert!(messages.iter().any(|msg| msg.contains("record 'flags' contains bitfields")));
    assert!(messages.iter().any(|msg| msg.contains("anonymous record cannot be projected")));
    assert!(messages.iter().any(|msg| msg.contains("anonymous enum cannot be projected")));
    assert!(messages.iter().any(|msg| msg.contains("unsupported declaration: vendor extension")));
}
