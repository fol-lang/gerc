use gec::emit::emit_source;
use gec::{
    generate, generate_from_source, GecConfig, GecInput, SourceDeclaration, SourceFunction,
    SourceLinkKind, SourceLinkRequirement, SourcePackage, SourceRecord, SourceType,
    SourceVariable,
};
use linc::{
    EvidenceKind, ItemKind, LinkAnalysisPackage, MatchConfidence, MatchStatus, ResolvedLinkPlan,
    SymbolMatch, SymbolVisibility, ValidationReport, ValidationSummary,
};
use linc::ir::{
    LinkInput, LinkLibrary, LinkLibraryKind, LinkRequirementSource, LinkResolutionMode,
    NativeSurfaceKind,
};

fn source_fixture() -> SourcePackage {
    let mut source = SourcePackage {
        source_path: Some("fixtures/source/demo.h".into()),
        ..SourcePackage::default()
    };
    source
        .declarations
        .push(SourceDeclaration::Function(SourceFunction {
            name: "demo_init".into(),
            parameters: vec![],
            return_type: SourceType::Int,
            variadic: false,
            source_offset: Some(12),
        }));
    source
        .declarations
        .push(SourceDeclaration::Record(SourceRecord {
            name: Some("demo_options".into()),
            is_union: false,
            fields: Some(vec![]),
            source_offset: Some(24),
        }));
    source.link_requirements.push(SourceLinkRequirement {
        name: "demo".into(),
        kind: SourceLinkKind::DynamicLibrary,
    });
    source
}

#[test]
fn generate_from_source_integrates_real_source_contracts() {
    let output = generate_from_source(source_fixture(), &GecConfig::new("demo_sys")).unwrap();
    let emitted = emit_source(&output.projection);

    assert_eq!(output.item_count(), 2);
    assert!(emitted.contains("pub fn demo_init"));
    assert!(emitted.contains("pub struct demo_options"));
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "demo"));
}

#[test]
fn gec_input_from_source_accepts_evidence() {
    let mut source = source_fixture();
    source
        .declarations
        .push(SourceDeclaration::Variable(SourceVariable {
            name: "hidden_value".into(),
            ty: SourceType::Int,
            source_offset: Some(40),
        }));
    source.link_requirements[0].name = "rawdemo".into();

    let input = GecInput::from_source_package(source)
        .with_validation(ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![
                SymbolMatch {
                    name: "demo_init".into(),
                    item_kind: ItemKind::Function,
                    status: MatchStatus::Matched,
                    visibility: Some(SymbolVisibility::Default),
                    provider_artifacts: vec!["libresolveddemo.so".into()],
                    confidence: MatchConfidence::High,
                    evidence_kind: EvidenceKind::ExactExported,
                },
                SymbolMatch {
                    name: "hidden_value".into(),
                    item_kind: ItemKind::Variable,
                    status: MatchStatus::Hidden,
                    visibility: Some(SymbolVisibility::Hidden),
                    provider_artifacts: vec!["libresolveddemo.so".into()],
                    confidence: MatchConfidence::Low,
                    evidence_kind: EvidenceKind::HiddenProvider,
                },
            ],
        })
        .with_link_plan(ResolvedLinkPlan {
            preferred_mode: LinkResolutionMode::Default,
            native_surface_kind: NativeSurfaceKind::LibraryNames,
            platform_constraints: Vec::new(),
            inputs: vec![LinkInput::Library(LinkLibrary {
                name: "resolveddemo".into(),
                kind: LinkLibraryKind::Default,
                source: LinkRequirementSource::Declared,
            })],
            requirements: Vec::new(),
            transitive_dependencies: Vec::new(),
        });

    let output = generate(&input, &GecConfig::new("demo_sys")).unwrap();
    let emitted = emit_source(&output.projection);

    assert!(emitted.contains("pub fn demo_init"));
    assert!(!emitted.contains("hidden_value"));
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "resolveddemo"));
    assert!(!output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "rawdemo"));
}

#[test]
fn generate_prefers_parallel_linc_analysis_evidence() {
    let mut source = source_fixture();
    source
        .declarations
        .push(SourceDeclaration::Variable(SourceVariable {
            name: "hidden_value".into(),
            ty: SourceType::Int,
            source_offset: Some(40),
        }));
    source.link_requirements[0].name = "rawdemo".into();

    let analysis = LinkAnalysisPackage {
        resolved_link_plan: Some(ResolvedLinkPlan {
            preferred_mode: LinkResolutionMode::Default,
            native_surface_kind: NativeSurfaceKind::LibraryNames,
            platform_constraints: Vec::new(),
            inputs: vec![LinkInput::Library(LinkLibrary {
                name: "analysisdemo".into(),
                kind: LinkLibraryKind::Default,
                source: LinkRequirementSource::Declared,
            })],
            requirements: Vec::new(),
            transitive_dependencies: Vec::new(),
        }),
        validation: Some(ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![
                SymbolMatch {
                    name: "demo_init".into(),
                    item_kind: ItemKind::Function,
                    status: MatchStatus::Matched,
                    visibility: Some(SymbolVisibility::Default),
                    provider_artifacts: vec!["libanalysisdemo.so".into()],
                    confidence: MatchConfidence::High,
                    evidence_kind: EvidenceKind::ExactExported,
                },
                SymbolMatch {
                    name: "hidden_value".into(),
                    item_kind: ItemKind::Variable,
                    status: MatchStatus::Hidden,
                    visibility: Some(SymbolVisibility::Hidden),
                    provider_artifacts: vec!["libanalysisdemo.so".into()],
                    confidence: MatchConfidence::Low,
                    evidence_kind: EvidenceKind::HiddenProvider,
                },
            ],
        }),
        ..LinkAnalysisPackage::default()
    };

    let output = generate(
        &GecInput::from_source_package(source).with_analysis(analysis),
        &GecConfig::new("demo_sys"),
    )
    .unwrap();
    let emitted = emit_source(&output.projection);

    assert!(emitted.contains("pub fn demo_init"));
    assert!(!emitted.contains("hidden_value"));
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "analysisdemo"));
    assert!(!output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "rawdemo"));
}
