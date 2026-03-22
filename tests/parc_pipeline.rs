mod common;

#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use std::path::{Path, PathBuf};

use gerc::{
    emit_crate, emit_source, generate, generate_from_source, GercConfig, GercInput, OutputMode,
    OverwritePolicy,
};

fn vendored_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../linc/tests/full_apps/external")
        .join(name)
        .join("header")
}

fn parse_vendored_source(entry: &Path, include_dirs: &[PathBuf]) -> Option<gerc::SourcePackage> {
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
    Some(common::from_binding_package(&binding))
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gerc_parc_pipeline_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn cargo_check(crate_dir: &Path) -> std::process::Output {
    std::process::Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(crate_dir)
        .output()
        .expect("spawn cargo check")
}

#[test]
fn vendored_zlib_parc_to_gerc_source_only() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let output = generate_from_source(source, &GercConfig::new("zlib_sys")).unwrap();
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
fn vendored_zlib_parc_to_gerc_is_deterministic() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let make = || {
        let source = parse_vendored_source(&entry, std::slice::from_ref(&include_dir)).unwrap();
        let output = generate_from_source(source, &GercConfig::new("zlib_sys")).unwrap();
        emit_source(&output.projection)
    };

    assert_eq!(make(), make());
}

#[test]
fn vendored_zlib_parc_to_gerc_emits_checkable_crate() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let cfg = GercConfig::new("zlib_sys");
    let output = generate_from_source(source, &cfg).unwrap();
    let dir = tempdir("zlib_source_only");
    let emitted = emit_crate(
        &output.projection,
        &cfg,
        &dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    assert!(emitted.root.join("Cargo.toml").exists());
    assert!(emitted.root.join("src/lib.rs").exists());
    assert!(std::fs::read_to_string(emitted.root.join("src/lib.rs")).is_ok());
}

#[test]
fn vendored_libpng_parc_to_gerc_source_only() {
    let root = vendored_root("libpng");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let output = generate_from_source(source, &GercConfig::new("png_sys")).unwrap();
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
fn vendored_libpng_parc_to_gerc_emits_checkable_crate() {
    let root = vendored_root("libpng");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
        return;
    };

    let cfg = GercConfig::new("png_sys");
    let output = generate_from_source(source, &cfg).unwrap();
    let dir = tempdir("libpng_source_only");
    let emitted = emit_crate(
        &output.projection,
        &cfg,
        &dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let lib_rs = std::fs::read_to_string(emitted.root.join("src/lib.rs")).unwrap();

    assert!(emitted.root.join("Cargo.toml").exists());
    assert!(lib_rs.contains("png_"));
}

#[test]
fn vendored_source_only_crates_fail_on_known_anonymous_type_lowering_gap() {
    let libpng_root = vendored_root("libpng");
    let libpng_include_dir = libpng_root.join("include");
    let libpng_entry = libpng_root.join("main.c");

    let Some(libpng_source) = parse_vendored_source(&libpng_entry, &[libpng_include_dir]) else {
        return;
    };

    let libpng_cfg = GercConfig::new("png_sys");
    let libpng_output = generate_from_source(libpng_source, &libpng_cfg).unwrap();
    let libpng_dir = tempdir("libpng_cargo_check");
    let libpng_emitted = emit_crate(
        &libpng_output.projection,
        &libpng_cfg,
        &libpng_dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let libpng_check = cargo_check(&libpng_emitted.root);
    let libpng_stderr = String::from_utf8_lossy(&libpng_check.stderr);
    assert!(!libpng_check.status.success());
    assert!(libpng_stderr.contains("pub type max_align_t = <anonymous>;"));
    assert!(libpng_stderr.contains("expected `::`, found `;`"));

    let zlib_root = vendored_root("zlib");
    let zlib_include_dir = zlib_root.join("include");
    let zlib_entry = zlib_root.join("main.c");

    let Some(zlib_source) = parse_vendored_source(&zlib_entry, &[zlib_include_dir]) else {
        return;
    };

    let zlib_cfg = GercConfig::new("zlib_sys");
    let zlib_output = generate_from_source(zlib_source, &zlib_cfg).unwrap();
    let zlib_dir = tempdir("zlib_cargo_check");
    let zlib_emitted = emit_crate(
        &zlib_output.projection,
        &zlib_cfg,
        &zlib_dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let zlib_check = cargo_check(&zlib_emitted.root);
    let zlib_stderr = String::from_utf8_lossy(&zlib_check.stderr);
    assert!(!zlib_check.status.success());
    assert!(zlib_stderr.contains("pub type max_align_t = <anonymous>;"));
    assert!(zlib_stderr.contains("expected `::`, found `;`"));
}

#[test]
fn vendored_zlib_parc_linc_gerc_link_surface() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::raw_headers::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("z")
            .no_origin_filter(),
    )
    .unwrap();

    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&result.package)),
        &GercConfig::new("zlib_sys"),
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
fn vendored_zlib_parc_linc_gerc_resolved_link_plan() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::raw_headers::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("z")
            .no_origin_filter(),
    )
    .unwrap();

    let plan = linc::resolve_link_plan(&result.package);
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&result.package))
            .with_link_plan(common::from_linc_link_plan(&plan)),
        &GercConfig::new("zlib_sys"),
    )
    .unwrap();

    assert_eq!(plan.inputs.len(), 1);
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "z"));
    assert!(gerc::emit_build_rs(&output.projection).contains("cargo:rustc-link-lib=dylib=z"));
}

#[test]
fn vendored_zlib_parc_linc_gerc_emits_link_aware_crate() {
    let root = vendored_root("zlib");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::raw_headers::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("z")
            .no_origin_filter(),
    )
    .unwrap();

    let plan = linc::resolve_link_plan(&result.package);
    let cfg = GercConfig::new("zlib_sys");
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&result.package))
            .with_link_plan(common::from_linc_link_plan(&plan)),
        &cfg,
    )
    .unwrap();
    let dir = tempdir("zlib_with_evidence");
    let emitted = emit_crate(
        &output.projection,
        &cfg,
        &dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let build_rs = std::fs::read_to_string(emitted.root.join("build.rs")).unwrap();
    let rustc_args =
        std::fs::read_to_string(emitted.root.join("rustc-link-args.txt")).unwrap();

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=z"));
    assert!(rustc_args.contains("-ldylib=z"));
}

#[test]
fn vendored_libpng_parc_linc_gerc_link_surface() {
    let root = vendored_root("libpng");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::raw_headers::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("png")
            .no_origin_filter(),
    )
    .unwrap();

    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&result.package)),
        &GercConfig::new("png_sys"),
    )
    .unwrap();
    let emitted = emit_source(&output.projection);

    assert!(emitted.contains("png_"));
    assert!(output
        .projection
        .link_requirements
        .iter()
        .any(|req| req.name == "png"));
}

#[test]
fn vendored_libpng_parc_linc_gerc_emits_link_aware_crate() {
    let root = vendored_root("libpng");
    let include_dir = root.join("include");
    let entry = root.join("main.c");

    let result = linc_common::process(
        &linc::raw_headers::HeaderConfig::new()
            .header(&entry)
            .include_dir(&include_dir)
            .link_lib("png")
            .no_origin_filter(),
    )
    .unwrap();

    let cfg = GercConfig::new("png_sys");
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&result.package)),
        &cfg,
    )
    .unwrap();
    let dir = tempdir("libpng_with_evidence");
    let emitted = emit_crate(
        &output.projection,
        &cfg,
        &dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let build_rs = std::fs::read_to_string(emitted.root.join("build.rs")).unwrap();
    let rustc_args =
        std::fs::read_to_string(emitted.root.join("rustc-link-args.txt")).unwrap();

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=png"));
    assert!(rustc_args.contains("-ldylib=png"));
}
