use gec::ir::{RustFunction, RustItem, RustProjection, RustType};
use gec::GecConsumer;
use linc::{
    BindingItem, BindingPackage, BindingType, CallingConvention, FunctionBinding,
    SourceDeclaration, SourceFunction, SourcePackage, SourceType,
};

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

#[test]
fn root_reexports_legacy_binding_adapters() {
    let mut pkg = linc::BindingPackage::new();
    pkg.items.push(linc::BindingItem::Function(linc::FunctionBinding {
        name: "legacy_demo".into(),
        calling_convention: linc::CallingConvention::C,
        parameters: vec![],
        return_type: linc::BindingType::Void,
        variadic: false,
        source_offset: None,
    }));

    let input = gec::input_from_binding_package(pkg.clone());
    assert_eq!(input.item_count(), 1);

    let json = serde_json::to_string(&pkg).unwrap();
    let from_json = gec::input_from_binding_json(&json).unwrap();
    assert_eq!(from_json.item_count(), 1);

    let source = gec::source_from_binding_package(&pkg);
    assert_eq!(source.declarations.len(), 1);
}

#[test]
fn root_reexports_rustc_link_arg_renderer() {
    let args = gec::emit_rustc_link_args(&[
        gec::ir::RustLinkRequirement {
            kind: gec::ir::RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: Some("/usr/lib".into()),
        },
        gec::ir::RustLinkRequirement {
            kind: gec::ir::RustLinkKind::Framework,
            name: "Security".into(),
            search_path: None,
        },
    ]);

    assert_eq!(args, vec!["-Lnative=/usr/lib", "-ldylib=z", "-lframework=Security"]);
}

#[test]
fn root_reexports_crate_emit_helpers() {
    let dir = tempdir("root_emit_crate");
    let mut projection = RustProjection::new();
    projection.items.push(RustItem::Function(RustFunction {
        name: "demo_init".into(),
        parameters: vec![],
        return_type: RustType::CInt,
        variadic: false,
        doc: None,
    }));

    let emitted = gec::emit_crate(
        &projection,
        &gec::GecConfig::new("demo_sys"),
        &dir,
        gec::OutputMode::Crate,
        gec::OverwritePolicy::Overwrite,
    )
    .unwrap();

    assert!(dir.join("Cargo.toml").exists());
    assert!(dir.join("src/lib.rs").exists());
    assert_eq!(gec::normalize_crate_name("demo-sys").unwrap(), "demo_sys");
    assert_eq!(emitted.root, dir);
}

#[test]
fn root_public_api_supports_source_to_crate_workflow() {
    let mut source = SourcePackage::default();
    source
        .declarations
        .push(SourceDeclaration::Function(SourceFunction {
            name: "workflow_init".into(),
            parameters: vec![],
            return_type: SourceType::Int,
            variadic: false,
            source_offset: None,
        }));

    let cfg = gec::GecConfig::new("workflow_sys");
    let output = gec::generate_from_source(source, &cfg).unwrap();
    let emitted_source = gec::emit_source(&output.projection);
    let emitted_crate = gec::emit_crate(
        &output.projection,
        &cfg,
        &tempdir("root_workflow_crate"),
        gec::OutputMode::Crate,
        gec::OverwritePolicy::Overwrite,
    )
    .unwrap();

    assert!(emitted_source.contains("pub fn workflow_init"));
    assert!(emitted_crate.root.join("Cargo.toml").exists());
    assert!(emitted_crate.root.join("src/lib.rs").exists());
}

#[test]
fn root_reexports_evidence_inputs() {
    let evidence = gec::EvidenceInputs::default();
    assert!(evidence.validation.is_none());
    assert!(evidence.link_plan.is_none());
}

#[test]
fn root_reexports_gate_and_lower_helpers() {
    let mut package = BindingPackage::new();
    package.items.push(BindingItem::Function(FunctionBinding {
        name: "demo_init".into(),
        calling_convention: CallingConvention::C,
        parameters: vec![],
        return_type: BindingType::Int,
        variadic: false,
        source_offset: None,
    }));

    let (decisions, diagnostics) = gec::gate_package(&package, None);
    assert_eq!(decisions.len(), 1);
    assert!(diagnostics.is_empty());
    assert!(matches!(decisions[0], gec::GateDecision::Accept));

    let (projection, lower_diags) = gec::lower_package(&package);
    assert!(lower_diags.is_empty());
    assert!(gec::emit_source(&projection).contains("pub fn demo_init"));
}

#[test]
fn root_helpers_support_intake_gate_lower_workflow() {
    let mut source = SourcePackage::default();
    source
        .declarations
        .push(SourceDeclaration::Function(SourceFunction {
            name: "workflow_gate".into(),
            parameters: vec![],
            return_type: SourceType::Int,
            variadic: false,
            source_offset: None,
        }));

    let input = gec::GecInput::from_source_package(source).with_evidence(gec::EvidenceInputs::default());
    let package = linc::from_source_package(&input.source);
    let (decisions, gate_diags) = gec::gate_package(&package, input.evidence.validation.as_ref());
    assert!(gate_diags.is_empty());
    assert!(matches!(decisions[0], gec::GateDecision::Accept));

    let (projection, lower_diags) = gec::lower_package(&package);
    assert!(lower_diags.is_empty());
    assert!(gec::emit_source(&projection).contains("pub fn workflow_gate"));
}

#[test]
fn root_reexports_output_meta_helpers() {
    let mut source = SourcePackage::default();
    source
        .declarations
        .push(SourceDeclaration::Function(SourceFunction {
            name: "meta_demo".into(),
            parameters: vec![],
            return_type: SourceType::Int,
            variadic: false,
            source_offset: None,
        }));

    let cfg = gec::GecConfig::new("meta_demo_sys");
    let output = gec::generate_from_source(source, &cfg).unwrap();
    let meta = gec::output_meta(&cfg, &output);
    let json = gec::meta_to_json(&meta).unwrap();
    let roundtrip = gec::meta_from_json(&json).unwrap();

    assert_eq!(roundtrip.crate_name, "meta_demo_sys");
    assert_eq!(roundtrip.item_count, output.item_count());
}

#[test]
fn root_reexports_projection_json_helpers() {
    let mut projection = RustProjection::new();
    projection.items.push(RustItem::Function(RustFunction {
        name: "json_demo".into(),
        parameters: vec![],
        return_type: RustType::CInt,
        variadic: false,
        doc: None,
    }));

    let json = gec::projection_to_json(&projection).unwrap();
    let roundtrip = gec::projection_from_json(&json).unwrap();

    assert_eq!(roundtrip.len(), 1);
    assert!(gec::emit_source(&roundtrip).contains("pub fn json_demo"));
}

#[test]
fn root_reexports_sidecar_helpers() {
    let mut projection = RustProjection::new();
    projection.items.push(RustItem::Function(RustFunction {
        name: "sidecar_demo".into(),
        parameters: vec![],
        return_type: RustType::CInt,
        variadic: false,
        doc: None,
    }));

    let sidecar = gec::build_sidecar("sidecar_demo_sys", &projection);
    let json = gec::sidecar_to_json(&sidecar).unwrap();
    let roundtrip = gec::sidecar_from_json(&json).unwrap();

    assert_eq!(roundtrip.crate_name, "sidecar_demo_sys");
    assert_eq!(roundtrip.items.len(), 1);
}

#[test]
fn root_reexports_consumer_helpers() {
    let mut projection = RustProjection::new();
    projection.items.push(RustItem::Function(RustFunction {
        name: "consumer_demo".into(),
        parameters: vec![],
        return_type: RustType::OpaquePtr { is_const: false },
        variadic: false,
        doc: None,
    }));

    let passthrough = gec::PassthroughConsumer;
    let passthrough_report = passthrough.inspect(&projection);
    assert_eq!(passthrough_report.consumer_name, "passthrough");
    assert_eq!(passthrough_report.items_inspected, 1);
    assert!(matches!(
        passthrough_report.findings[0].kind,
        gec::FindingKind::Usable
    ));

    let fol = gec::FolConsumer;
    let fol_report = fol.inspect(&projection);
    assert_eq!(fol_report.consumer_name, "fol-interloop-rust");
    assert!(matches!(
        fol_report.findings[0].kind,
        gec::FindingKind::NeedsWrapper
    ));

    let externs = gec::extern_function_names(&projection);
    let records = gec::record_names(&projection);
    let aliases = gec::type_alias_names(&projection);

    assert_eq!(externs, vec!["consumer_demo"]);
    assert!(records.is_empty());
    assert!(aliases.is_empty());
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gec_test_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
