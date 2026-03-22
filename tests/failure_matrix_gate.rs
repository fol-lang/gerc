use gerc::c::{
    BindingItem, BindingPackage, BindingType, CallingConvention, EvidenceKind, FunctionBinding,
    ItemKind, MatchConfidence, MatchStatus, ParameterBinding, SymbolMatch, SymbolVisibility,
    ValidationReport, ValidationSummary, VariableBinding,
};
use gerc::gate::{gate_package, GateDecision};

fn fixture_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "ok_fn".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![ParameterBinding {
            name: Some("value".into()),
            ty: BindingType::Int,
        }],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "dup_fn".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Void,
        variadic: false,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "hidden_data".into(),
        ty: BindingType::Int,
        source_offset: None,
    }));
    pkg
}

fn fixture_validation() -> ValidationReport {
    ValidationReport {
        phases: Vec::new(),
        entries: Vec::new(),
        summary: ValidationSummary::default(),
        matches: vec![
            SymbolMatch {
                name: "ok_fn".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::Matched,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["libok.a:ok.o".into()],
                confidence: MatchConfidence::High,
                evidence_kind: EvidenceKind::ExactExported,
            },
            SymbolMatch {
                name: "dup_fn".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::DuplicateProviders,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["liba.a:dup.o".into(), "libb.a:dup.o".into()],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::DuplicateVisibleProviders,
            },
            SymbolMatch {
                name: "hidden_data".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::Hidden,
                visibility: Some(SymbolVisibility::Hidden),
                provider_artifacts: vec!["libhidden.a:data.o".into()],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::HiddenProvider,
            },
        ],
    }
}

#[test]
fn failure_matrix_gate_groups_validation_refusals() {
    let pkg = fixture_package();
    let validation = fixture_validation();
    let (decisions, diagnostics) = gate_package(&pkg, Some(&validation));

    assert_eq!(decisions.len(), 3);
    assert!(matches!(decisions[0], GateDecision::Accept));
    assert!(matches!(&decisions[1], GateDecision::Reject(reason) if reason.contains("duplicate provider")));
    assert!(matches!(&decisions[2], GateDecision::Reject(reason) if reason.contains("Hidden")));
    assert!(diagnostics.iter().any(|d| d.message.contains("dup_fn")));
    assert!(diagnostics.iter().any(|d| d.message.contains("hidden_data")));
}
