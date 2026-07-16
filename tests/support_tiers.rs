use gerc::c::{
    LinkFramework, LinkInput, LinkLibrary, LinkLibraryKind, LinkRequirementSource,
    LinkResolutionMode, NativeSurfaceKind, RequirementResolution, ResolvedLinkPlan,
    ResolvedLinkRequirement,
};
use gerc::{
    emit_rustc_link_args, emit_source, generate, generate_from_source, GercConfig, GercInput,
    SourceDeclaration, SourceField, SourceFunction, SourcePackage, SourceParameter, SourceRecord,
    SourceType, SourceTypeAlias,
};

fn source_only_widget_api() -> SourcePackage {
    SourcePackage {
        declarations: vec![
            SourceDeclaration::Record(SourceRecord {
                name: Some("widget".into()),
                is_union: false,
                fields: None,
                source_offset: None,
            }),
            SourceDeclaration::TypeAlias(SourceTypeAlias {
                name: "widget_handle".into(),
                target: SourceType::Pointer(Box::new(SourceType::RecordRef("widget".into()))),
                source_offset: None,
            }),
            SourceDeclaration::Function(SourceFunction {
                name: "widget_open".into(),
                parameters: vec![],
                return_type: SourceType::TypedefRef("widget_handle".into()),
                variadic: false,
                source_offset: None,
            }),
            SourceDeclaration::Function(SourceFunction {
                name: "widget_close".into(),
                parameters: vec![SourceParameter {
                    name: Some("handle".into()),
                    ty: SourceType::TypedefRef("widget_handle".into()),
                }],
                return_type: SourceType::Int,
                variadic: false,
                source_offset: None,
            }),
        ],
        ..SourcePackage::default()
    }
}

#[test]
fn support_tier_source_only_minimal_api_is_supported() {
    let output = generate_from_source(source_only_widget_api(), &GercConfig::new("widget_sys"))
        .expect("source-only widget API should generate");
    let source = emit_source(&output.projection);

    assert!(output.diagnostics.is_empty());
    assert!(source.contains("pub struct widget"));
    assert!(source.contains("pub type widget_handle"));
    assert!(source.contains("pub fn widget_open"));
    assert!(source.contains("pub fn widget_close"));
}

#[test]
fn support_tier_evidence_aware_link_plan_enriches_generation() {
    let declared = LinkInput::Library(LinkLibrary {
        name: "ssl".into(),
        kind: LinkLibraryKind::Dynamic,
        source: LinkRequirementSource::Declared,
    });
    let plan = ResolvedLinkPlan {
        preferred_mode: LinkResolutionMode::PreferDynamic,
        native_surface_kind: NativeSurfaceKind::LibraryNames,
        inputs: vec![declared.clone()],
        requirements: vec![ResolvedLinkRequirement {
            declared,
            source: LinkRequirementSource::Declared,
            resolution: RequirementResolution::Resolved,
            providers: vec![],
        }],
        transitive_dependencies: vec!["crypto".into()],
        ..ResolvedLinkPlan::default()
    };

    let input = GercInput::from_source_package(source_only_widget_api()).with_link_plan(plan);
    let output = generate(&input, &GercConfig::new("widget_ssl_sys"))
        .expect("evidence-aware widget API should generate");
    let rustc_args = emit_rustc_link_args(&output.projection.link_requirements);

    assert_eq!(output.projection.link_requirements.len(), 1);
    assert!(rustc_args.iter().any(|arg| arg == "-ldylib=ssl"));
}

#[test]
fn support_tier_evidence_aware_framework_plan_is_supported() {
    let declared = LinkInput::Framework(LinkFramework {
        name: "Security".into(),
        source: LinkRequirementSource::Declared,
    });
    let plan = ResolvedLinkPlan {
        preferred_mode: LinkResolutionMode::PreferDynamic,
        native_surface_kind: NativeSurfaceKind::LibraryNames,
        inputs: vec![declared.clone()],
        requirements: vec![ResolvedLinkRequirement {
            declared,
            source: LinkRequirementSource::Declared,
            resolution: RequirementResolution::Resolved,
            providers: vec![],
        }],
        ..ResolvedLinkPlan::default()
    };

    let input = GercInput::from_source_package(source_only_widget_api()).with_link_plan(plan);
    let output = generate(&input, &GercConfig::new("widget_security_sys"))
        .expect("framework-backed widget API should generate");
    let rustc_args = emit_rustc_link_args(&output.projection.link_requirements);

    assert!(rustc_args.iter().any(|arg| arg == "-lframework=Security"));
}

#[test]
fn support_tier_bitfield_records_must_reject_in_source_only_mode() {
    let source = SourcePackage {
        declarations: vec![SourceDeclaration::Record(SourceRecord {
            name: Some("flags".into()),
            is_union: false,
            fields: Some(vec![
                SourceField {
                    name: Some("ready".into()),
                    ty: SourceType::UInt,
                    bit_width: Some(1),
                },
                SourceField {
                    name: Some("rest".into()),
                    ty: SourceType::UInt,
                    bit_width: None,
                },
            ]),
            source_offset: None,
        })],
        ..SourcePackage::default()
    };

    let output =
        generate_from_source(source, &GercConfig::new("flags_sys")).expect("generation runs");
    let source = emit_source(&output.projection);

    assert!(output
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("bitfields")));
    assert!(!source.contains("pub struct flags"));
}

#[test]
fn support_tier_source_only_projection_is_deterministic() {
    let make = || {
        let output = generate_from_source(source_only_widget_api(), &GercConfig::new("widget_sys"))
            .expect("source-only widget API should generate");
        serde_json::to_string(&output.projection).expect("projection json")
    };

    assert_eq!(make(), make());
}

#[test]
fn support_tier_evidence_aware_link_args_are_deterministic() {
    let declared = LinkInput::Library(LinkLibrary {
        name: "ssl".into(),
        kind: LinkLibraryKind::Dynamic,
        source: LinkRequirementSource::Declared,
    });
    let plan = ResolvedLinkPlan {
        preferred_mode: LinkResolutionMode::PreferDynamic,
        native_surface_kind: NativeSurfaceKind::LibraryNames,
        inputs: vec![declared.clone()],
        requirements: vec![ResolvedLinkRequirement {
            declared,
            source: LinkRequirementSource::Declared,
            resolution: RequirementResolution::Resolved,
            providers: vec![],
        }],
        ..ResolvedLinkPlan::default()
    };

    let make = || {
        let input =
            GercInput::from_source_package(source_only_widget_api()).with_link_plan(plan.clone());
        let output = generate(&input, &GercConfig::new("widget_ssl_sys"))
            .expect("evidence-aware widget API should generate");
        emit_rustc_link_args(&output.projection.link_requirements)
    };

    assert_eq!(make(), make());
}
