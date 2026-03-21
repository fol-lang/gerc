use gec::emit::emit_source;
use gec::{generate_from_source, GecConfig};
use linc::{
    intake::source::SourceLinkKind, SourceDeclaration, SourceFunction, SourceLinkRequirement,
    SourcePackage, SourceRecord, SourceType,
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
