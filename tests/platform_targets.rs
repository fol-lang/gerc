use gerc::c::{
    LinkFramework, LinkInput, LinkRequirementSource, LinkResolutionMode, NativeSurfaceKind,
    ResolvedLinkPlan, ResolvedLinkRequirement, RequirementResolution,
};
use gerc::{
    emit_build_rs, emit_rustc_args, generate, GercConfig, GercInput, SourceDeclaration,
    SourceFunction, SourcePackage, SourceType,
};

fn minimal_platform_source() -> SourcePackage {
    let mut source = SourcePackage::default();
    source
        .declarations
        .push(SourceDeclaration::Function(SourceFunction {
            name: "platform_anchor".into(),
            parameters: vec![],
            return_type: SourceType::Int,
            variadic: false,
            source_offset: None,
        }));
    source
}

#[test]
fn apple_framework_target_emits_framework_directives() {
    let declared = LinkInput::Framework(LinkFramework {
        name: "Security".into(),
        source: LinkRequirementSource::Declared,
    });
    let plan = ResolvedLinkPlan {
        preferred_mode: LinkResolutionMode::PreferDynamic,
        native_surface_kind: NativeSurfaceKind::LibraryNames,
        platform_constraints: vec!["macos".into()],
        inputs: vec![declared.clone()],
        requirements: vec![ResolvedLinkRequirement {
            declared,
            source: LinkRequirementSource::Declared,
            resolution: RequirementResolution::Resolved,
            providers: vec![],
        }],
        ..ResolvedLinkPlan::default()
    };

    let input = GercInput::from_source_package(minimal_platform_source()).with_link_plan(plan);
    let output = generate(&input, &GercConfig::new("security_sys")).unwrap();
    let build_rs = emit_build_rs(&output.projection);
    let rustc_args = emit_rustc_args(&output.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=framework=Security"));
    assert!(rustc_args.contains("-lframework=Security"));
}
