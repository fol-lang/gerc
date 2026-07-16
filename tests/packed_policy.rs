use gerc::c::{
    AbiConfidence, BindingItem, BindingPackage, BindingType, FieldBinding, FieldLayout,
    RecordBinding, RecordKind, RecordRepresentation,
};
use gerc::emit::emit_source;
use gerc::gate::{gate_package, GateDecision};
use gerc::lower::lower_package;

fn packed_record(name: &str, bitfield: bool) -> BindingPackage {
    let mut package = BindingPackage::new();
    package.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Struct,
        name: Some(name.into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("tag".into()),
                ty: BindingType::UChar,
                bit_width: None,
                layout: Some(FieldLayout {
                    offset_bytes: Some(0),
                }),
            },
            FieldBinding {
                name: Some("value".into()),
                ty: BindingType::UInt,
                bit_width: bitfield.then_some(3),
                layout: Some(FieldLayout {
                    offset_bytes: (!bitfield).then_some(1),
                }),
            },
        ]),
        representation: Some(RecordRepresentation {
            size: Some(5),
            align: Some(1),
            completeness: Some("Complete".into()),
        }),
        abi_confidence: Some(AbiConfidence::FieldOffsetsProbed),
        source_offset: None,
    }));
    package
}

fn packed_union(name: &str) -> BindingPackage {
    let mut package = BindingPackage::new();
    package.items.push(BindingItem::Record(RecordBinding {
        kind: RecordKind::Union,
        name: Some(name.into()),
        fields: Some(vec![
            FieldBinding {
                name: Some("word".into()),
                ty: BindingType::UInt,
                bit_width: None,
                layout: Some(FieldLayout {
                    offset_bytes: Some(0),
                }),
            },
            FieldBinding {
                name: Some("bytes".into()),
                ty: BindingType::Array(Box::new(BindingType::UChar), Some(4)),
                bit_width: None,
                layout: Some(FieldLayout {
                    offset_bytes: Some(0),
                }),
            },
        ]),
        representation: Some(RecordRepresentation {
            size: Some(4),
            align: Some(1),
            completeness: Some("Complete".into()),
        }),
        abi_confidence: Some(AbiConfidence::FieldOffsetsProbed),
        source_offset: None,
    }));
    package
}

#[test]
fn packed_policy_non_bitfield_record_stays_projectable() {
    let package = packed_record("packed_payload", false);
    let (decisions, diagnostics) = gate_package(&package, None);
    let (projection, lower_diags) = lower_package(&package);
    let source = emit_source(&projection);

    assert_eq!(decisions, vec![GateDecision::Accept]);
    assert!(diagnostics.is_empty());
    assert!(lower_diags.is_empty());
    assert!(source.contains("pub struct packed_payload"));
    assert!(source.contains("pub tag:"));
    assert!(source.contains("pub value:"));
}

#[test]
fn packed_policy_bitfield_record_remains_rejected() {
    let package = packed_record("packed_flags", true);
    let (decisions, diagnostics) = gate_package(&package, None);

    assert!(matches!(
        &decisions[0],
        GateDecision::Reject(reason) if reason.contains("bitfields")
    ));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.message.contains("bitfields")));
}

#[test]
fn packed_policy_non_bitfield_union_stays_projectable() {
    let package = packed_union("packed_union_payload");
    let (decisions, diagnostics) = gate_package(&package, None);
    let (projection, lower_diags) = lower_package(&package);
    let source = emit_source(&projection);

    assert_eq!(decisions, vec![GateDecision::Accept]);
    assert!(diagnostics.is_empty());
    assert!(lower_diags.is_empty());
    assert!(source.contains("pub union packed_union_payload"));
    assert!(source.contains("pub word: core::ffi::c_uint"));
    assert!(source.contains("pub bytes: [core::ffi::c_uchar; 4]"));
}
