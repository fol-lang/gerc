use gec::ir::{RustFunction, RustItem, RustProjection, RustType};
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
    let (decisions, gate_diags) = gec::gate_package(&input.package, input.evidence.validation.as_ref());
    assert!(gate_diags.is_empty());
    assert!(matches!(decisions[0], gec::GateDecision::Accept));

    let (projection, lower_diags) = gec::lower_package(&input.package);
    assert!(lower_diags.is_empty());
    assert!(gec::emit_source(&projection).contains("pub fn workflow_gate"));
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gec_test_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
