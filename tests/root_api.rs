use gec::ir::{RustFunction, RustItem, RustProjection, RustType};
use linc::{SourceDeclaration, SourceFunction, SourcePackage, SourceType};

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

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gec_test_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
