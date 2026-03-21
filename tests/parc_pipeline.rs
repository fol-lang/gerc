#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use std::path::{Path, PathBuf};

use gec::{emit_source, generate, generate_from_source, GecConfig, GecInput};

fn vendored_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../linc/tests/full_apps/external")
        .join(name)
        .join("header")
}

fn parse_vendored_source(entry: &Path, include_dirs: &[PathBuf]) -> Option<linc::SourcePackage> {
    let mut cpp_options = vec!["-E".to_string()];
    for dir in include_dirs {
        cpp_options.push(format!("-I{}", dir.display()));
    }

    let config = parc::driver::Config {
        cpp_command: std::env::var("CC").unwrap_or_else(|_| "gcc".into()),
        cpp_options,
        flavor: parc::driver::Flavor::GnuC11,
    };

    let parsed = parc::driver::parse(&config, entry).ok()?;
    let extracted = parc::extract::extract_from_translation_unit(&parsed.unit, None);
    let binding = linc_common::from_parc_package(&extracted);
    Some(linc::intake::adapters::from_binding_package(&binding))
}

#[test]
fn vendored_zlib_parc_to_gec_source_only() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let output = generate_from_source(source, &GecConfig::new("zlib_sys")).unwrap();
    let emitted = emit_source(&output.projection);

    assert!(
        output.item_count() >= 20,
        "expected at least 20 vendored zlib items, got {}",
        output.item_count()
    );
    assert!(emitted.contains("pub fn deflate"));
    assert!(emitted.contains("pub fn inflate"));
    assert!(emitted.contains("pub type Bytef"));
}

#[test]
fn vendored_zlib_parc_to_gec_is_deterministic() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let make = || {
        let source = parse_vendored_source(&entry, std::slice::from_ref(&include_dir)).unwrap();
        let output = generate_from_source(source, &GecConfig::new("zlib_sys")).unwrap();
        emit_source(&output.projection)
    };

    assert_eq!(make(), make());
}

#[test]
fn vendored_libpng_parc_to_gec_source_only() {
    let root = vendored_root("libpng");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let output = generate_from_source(source, &GecConfig::new("png_sys")).unwrap();
    let emitted = emit_source(&output.projection);

    assert!(
        output.item_count() >= 10,
        "expected at least 10 vendored libpng items, got {}",
        output.item_count()
    );
    assert!(emitted.contains("png_"));
    assert!(
        emitted.contains("pub fn png_create_read_struct")
            || emitted.contains("pub fn png_read_png")
            || emitted.contains("pub fn png_init_io")
    );
}

#[test]
fn vendored_zlib_parc_linc_gec_link_surface() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("z")
            .no_origin_filter(),
    )
    .unwrap();

    let output = generate(
        &GecInput::from_package(result.package),
        &GecConfig::new("zlib_sys"),
    )
    .unwrap();
    let emitted = emit_source(&output.projection);

    assert!(emitted.contains("pub fn deflate"));
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "z"));
}

#[test]
fn vendored_zlib_parc_linc_gec_resolved_link_plan() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("z")
            .no_origin_filter(),
    )
    .unwrap();

    let plan = linc::resolve_link_plan(&result.package);
    let output = generate(
        &GecInput::from_package(result.package).with_link_plan(plan.clone()),
        &GecConfig::new("zlib_sys"),
    )
    .unwrap();

    assert_eq!(plan.inputs.len(), 1);
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "z"));
    assert!(gec::emit_build_rs(&output.projection).contains("cargo:rustc-link-lib=dylib=z"));
}
