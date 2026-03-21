//! Validation-driven regression tests patterned after `linc` fixture states.

use gec::config::GecConfig;
use gec::contract::generate;
use gec::intake::GecInput;
use gec::ir::RustItem;
use linc::{
    BindingItem, BindingPackage, BindingType, CallingConvention, EvidenceKind, FunctionBinding,
    ItemKind, MatchConfidence, MatchStatus, SymbolMatch, SymbolVisibility, ValidationReport,
    ValidationSummary, VariableBinding,
};

fn input_from_binding(pkg: BindingPackage) -> GecInput {
    GecInput::from_source_package(linc::intake::adapters::from_binding_package(&pkg))
}

fn fixture_package() -> BindingPackage {
    let mut pkg = BindingPackage::new();
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "demo_init".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "decorated_only".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Void,
        variadic: false,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Function(FunctionBinding {
        name: "dup_provider".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Void,
        variadic: false,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "VISIBLE_DATA".into(),
        ty: BindingType::Int,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "hidden_data".into(),
        ty: BindingType::Int,
        source_offset: None,
    }));
    pkg.items.push(BindingItem::Variable(VariableBinding {
        name: "wrong_kind_data".into(),
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
                name: "demo_init".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::Matched,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["libdemo.a:demo_init.o".into()],
                confidence: MatchConfidence::High,
                evidence_kind: EvidenceKind::ExactExported,
            },
            SymbolMatch {
                name: "decorated_only".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::DecorationMismatch,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["demo.lib:decorated.obj".into()],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::DecoratedCandidate,
            },
            SymbolMatch {
                name: "dup_provider".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::DuplicateProviders,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec![
                    "libfoo_one.a:foo1.o".into(),
                    "libfoo_two.a:foo2.o".into(),
                ],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::DuplicateVisibleProviders,
            },
            SymbolMatch {
                name: "VISIBLE_DATA".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::Matched,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["libdemo.a:data.o".into()],
                confidence: MatchConfidence::High,
                evidence_kind: EvidenceKind::ExactExported,
            },
            SymbolMatch {
                name: "hidden_data".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::Hidden,
                visibility: Some(SymbolVisibility::Hidden),
                provider_artifacts: vec!["libdemo_hidden.a:hidden.o".into()],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::HiddenProvider,
            },
            SymbolMatch {
                name: "wrong_kind_data".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::NotAVariable,
                visibility: Some(SymbolVisibility::Default),
                provider_artifacts: vec!["libdemo.a:functions.o".into()],
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::KindMismatch,
            },
        ],
    }
}

#[test]
fn validation_fixture_filters_unusable_function_states() {
    let input = input_from_binding(fixture_package()).with_validation(fixture_validation());
    let output = generate(&input, &GecConfig::new("demo_sys")).unwrap();

    assert!(output
        .projection
        .items
        .iter()
        .any(|item| matches!(item, RustItem::Function(function) if function.name == "demo_init")));
    assert!(!output.projection.items.iter().any(
        |item| matches!(item, RustItem::Function(function) if function.name == "decorated_only")
    ));
    assert!(!output.projection.items.iter().any(
        |item| matches!(item, RustItem::Function(function) if function.name == "dup_provider")
    ));
    assert!(output
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("decorated_only")
            && diag.message.contains("DecorationMismatch")));
    assert!(output
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("dup_provider")
            && diag.message.contains("duplicate provider")));
}

#[test]
fn validation_fixture_filters_unusable_variable_states() {
    let input = input_from_binding(fixture_package()).with_validation(fixture_validation());
    let output = generate(&input, &GecConfig::new("demo_sys")).unwrap();

    assert!(output
        .projection
        .items
        .iter()
        .any(|item| matches!(item, RustItem::Static(variable) if variable.name == "VISIBLE_DATA")));
    assert!(!output
        .projection
        .items
        .iter()
        .any(|item| matches!(item, RustItem::Static(variable) if variable.name == "hidden_data")));
    assert!(!output.projection.items.iter().any(
        |item| matches!(item, RustItem::Static(variable) if variable.name == "wrong_kind_data")
    ));
    assert!(output
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("hidden_data") && diag.message.contains("Hidden")));
    assert!(output
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("wrong_kind_data")
            && diag.message.contains("NotAVariable")));
}
