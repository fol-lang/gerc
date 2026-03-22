mod common;

#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use gerc::{emit_source, generate, generate_from_source, GercConfig, GercInput};

fn parc_source_artifact(source: &str) -> parc::ir::SourcePackage {
    let package = parc::extract::extract_from_source(source).expect("parc extraction should work");
    let json = serde_json::to_string_pretty(&package).expect("parc artifact json");
    serde_json::from_str(&json).expect("parc artifact roundtrip")
}

fn gerc_source_from_parc_artifact(package: &parc::ir::SourcePackage) -> gerc::SourcePackage {
    let binding = linc_common::from_parc_package(package);
    common::from_binding_package(&binding)
}

#[test]
fn parc_artifact_roundtrip_can_drive_gerc_source_generation() {
    let parc_pkg = parc_source_artifact(
        r#"
        typedef unsigned long size_t;
        struct point { int x; int y; };
        extern int demo_init(struct point* p, size_t count);
        "#,
    );

    let output = generate_from_source(
        gerc_source_from_parc_artifact(&parc_pkg),
        &GercConfig::new("demo_sys"),
    )
    .unwrap();
    let emitted = emit_source(&output.projection);

    assert!(emitted.contains("pub type size_t"));
    assert!(emitted.contains("pub struct point"));
    assert!(emitted.contains("pub fn demo_init"));
}

#[test]
fn parc_and_linc_artifact_roundtrips_can_drive_gerc_evidence_aware_generation() {
    let parc_pkg = parc_source_artifact(
        r#"
        extern int demo_init(int flags);
        extern int hidden_value;
        "#,
    );

    let binding = linc_common::from_parc_package(&parc_pkg);
    let mut linc_source = linc::intake::adapters::from_binding_package(&binding);
    linc_source.link_requirements.push(linc::SourceLinkRequirement {
        name: "demo".into(),
        kind: linc::SourceLinkKind::DynamicLibrary,
    });

    let analysis = linc::analyze_source_package(&linc_source);
    let analysis_json = serde_json::to_string_pretty(&analysis).expect("analysis artifact json");
    let analysis_artifact: linc::LinkAnalysisPackage =
        serde_json::from_str(&analysis_json).expect("analysis artifact roundtrip");

    let output = generate(
        &GercInput::from_source_package(gerc_source_from_parc_artifact(&parc_pkg))
            .with_analysis(common::from_linc_analysis(&analysis_artifact)),
        &GercConfig::new("demo_sys"),
    )
    .unwrap();

    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "demo"));
    assert!(emit_source(&output.projection).contains("pub fn demo_init"));
}
