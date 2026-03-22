//! Integration tests exercising realistic library surfaces through the full
//! gerc pipeline: intake → gate → lower → emit → crate generation.

mod common;

use std::path::{Path, PathBuf};

#[path = "../test/stress/freetype.rs"]
mod freetype;
#[path = "../test/stress/openssl.rs"]
mod openssl;
#[path = "../test/stress/sqlite.rs"]
mod sqlite;
#[path = "../test/stress/zlib.rs"]
mod zlib;

use gerc::config::GercConfig;
use gerc::consumer::{build_sidecar, sidecar_to_json, GercConsumer, PassthroughConsumer};
use gerc::contract::{generate, meta_to_json, output_meta, projection_to_json};
use gerc::crategen::{emit_crate, OutputMode, OverwritePolicy};
use gerc::emit::emit_source;
use gerc::intake::GercInput;

fn run_full_pipeline(pkg: linc::ir::BindingPackage, crate_name: &str) -> PipelineResult {
    let input = GercInput::from_source_package(common::from_binding_package(&pkg));
    let cfg = GercConfig::new(crate_name);
    let output = generate(&input, &cfg).unwrap();
    let source = emit_source(&output.projection);
    let meta = output_meta(&cfg, &output);
    let meta_json = meta_to_json(&meta).unwrap();
    let proj_json = projection_to_json(&output.projection).unwrap();
    let sidecar = build_sidecar(crate_name, &output.projection);
    let sidecar_json = sidecar_to_json(&sidecar).unwrap();
    let consumer = PassthroughConsumer;
    let report = consumer.inspect(&output.projection);

    PipelineResult {
        item_count: output.item_count(),
        diagnostic_count: output.diagnostics.len(),
        source,
        meta_json,
        proj_json,
        sidecar_json,
        consumer_findings: report.findings.len(),
        link_libs: output.projection.link_requirements.len(),
    }
}

#[allow(dead_code)]
struct PipelineResult {
    item_count: usize,
    diagnostic_count: usize,
    source: String,
    meta_json: String,
    proj_json: String,
    sidecar_json: String,
    consumer_findings: usize,
    link_libs: usize,
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gerc_library_examples_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
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

// ---- zlib ----

#[test]
fn zlib_full_pipeline() {
    let r = run_full_pipeline(zlib::zlib_package(), "zlib_sys");

    // zlib has 5 typedefs + 2 structs + 14 functions + bindable macros
    assert!(
        r.item_count >= 20,
        "expected ≥20 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("pub fn deflate"));
    assert!(r.source.contains("pub fn inflate"));
    assert!(r.source.contains("pub fn compress"));
    assert!(r.source.contains("pub fn adler32"));
    assert!(r.source.contains("pub fn crc32"));
    assert!(r.source.contains("pub fn zlibVersion"));
    assert!(r.source.contains("pub struct z_stream"));
    assert!(r.source.contains("pub struct gz_header"));
    assert!(r.source.contains("pub type Bytef"));
    assert!(r.link_libs >= 1);
}

#[test]
fn zlib_deterministic() {
    let pkg = zlib::zlib_package();
    let s1 = emit_source(
        &generate(
            &GercInput::from_source_package(common::from_binding_package(&pkg.clone())),
            &GercConfig::new("z"),
        )
            .unwrap()
            .projection,
    );
    let s2 = emit_source(
        &generate(
            &GercInput::from_source_package(common::from_binding_package(&pkg)),
            &GercConfig::new("z"),
        )
            .unwrap()
            .projection,
    );
    assert_eq!(s1, s2);
}

#[test]
fn zlib_source_has_balanced_braces() {
    let r = run_full_pipeline(zlib::zlib_package(), "zlib_sys");
    let opens = r.source.matches('{').count();
    let closes = r.source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in zlib output");
}

// ---- sqlite3 ----

#[test]
fn sqlite3_full_pipeline() {
    let r = run_full_pipeline(sqlite::sqlite3_package(), "sqlite3_sys");

    // SQLite: ~10 opaque structs + 4 typedefs + ~42 functions + 1 enum + macros
    assert!(
        r.item_count >= 50,
        "expected ≥50 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("pub fn sqlite3_open"));
    assert!(r.source.contains("pub fn sqlite3_close"));
    assert!(r.source.contains("pub fn sqlite3_exec"));
    assert!(r.source.contains("pub fn sqlite3_prepare_v2"));
    assert!(r.source.contains("pub fn sqlite3_step"));
    assert!(r.source.contains("pub fn sqlite3_finalize"));
    assert!(r.source.contains("pub fn sqlite3_bind_text"));
    assert!(r.source.contains("pub fn sqlite3_column_text"));
    assert!(r.source.contains("pub fn sqlite3_mprintf")); // variadic
    assert!(r.source.contains("pub fn sqlite3_malloc"));
    assert!(r.source.contains("pub fn sqlite3_free"));
    // opaque handles
    assert!(r.source.contains("pub struct sqlite3"));
    assert!(r.source.contains("pub struct sqlite3_stmt"));
    assert!(r.link_libs >= 1);
}

#[test]
fn sqlite3_opaque_handles_are_zero_sized() {
    let r = run_full_pipeline(sqlite::sqlite3_package(), "sqlite3_sys");
    // opaque structs should use the _opaque: [u8; 0] pattern
    assert!(r.source.contains("_opaque: [u8; 0]"));
}

#[test]
fn sqlite3_variadic_functions() {
    let r = run_full_pipeline(sqlite::sqlite3_package(), "sqlite3_sys");
    assert!(r.source.contains("sqlite3_mprintf"));
    assert!(r.source.contains("...")); // variadic marker
}

#[test]
fn sqlite3_json_roundtrip() {
    let pkg = sqlite::sqlite3_package();
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&pkg)),
        &GercConfig::new("sqlite3_sys"),
    )
    .unwrap();
    let json = projection_to_json(&output.projection).unwrap();
    let proj2 = gerc::contract::projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), output.projection.len());
}

#[test]
fn sqlite3_deterministic() {
    let make = || {
        let pkg = sqlite::sqlite3_package();
        let output = generate(
            &GercInput::from_source_package(common::from_binding_package(&pkg)),
            &GercConfig::new("sqlite3_sys"),
        )
        .unwrap();
        emit_source(&output.projection)
    };

    assert_eq!(make(), make());
}

#[test]
fn sqlite3_sidecar_completeness() {
    let pkg = sqlite::sqlite3_package();
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&pkg)),
        &GercConfig::new("sqlite3_sys"),
    )
    .unwrap();
    let sidecar = build_sidecar("sqlite3_sys", &output.projection);
    assert_eq!(sidecar.crate_name, "sqlite3_sys");
    assert_eq!(sidecar.items.len(), output.projection.len());
}

#[test]
fn sqlite3_surface_preserves_typedef_handle_alloc_flow() {
    let r = run_full_pipeline(sqlite::sqlite3_package(), "sqlite3_sys");

    assert!(r.source.contains(
        "pub type sqlite3_destructor_type = Option<unsafe extern \"C\" fn(*mut core::ffi::c_void)>;"
    ));
    assert!(r.source.contains("pub struct sqlite3 { _opaque: [u8; 0] }"));
    assert!(r.source.contains("pub struct sqlite3_stmt { _opaque: [u8; 0] }"));
    assert!(
        r.source.contains(
            "pub fn sqlite3_open(filename: *const core::ffi::c_char, ppDb: *mut *mut sqlite3) -> core::ffi::c_int;"
        )
    );
    assert!(r.source.contains(
        "pub fn sqlite3_prepare_v2(db: *mut sqlite3, sql: *const core::ffi::c_char, nByte: core::ffi::c_int, ppStmt: *mut *mut sqlite3_stmt, pzTail: *mut *const core::ffi::c_char) -> core::ffi::c_int;"
    ));
    assert!(
        r.source
            .contains("pub fn sqlite3_malloc(n: core::ffi::c_int) -> *mut core::ffi::c_void;")
    );
    assert!(
        r.source.contains("pub fn sqlite3_free(ptr: *mut core::ffi::c_void);")
    );
}

#[test]
fn sqlite3_emitted_crate_passes_cargo_check_and_keeps_link_args() {
    let pkg = sqlite::sqlite3_package();
    let cfg = GercConfig::new("sqlite3_sys");
    let output = generate(
        &GercInput::from_source_package(common::from_binding_package(&pkg)),
        &cfg,
    )
    .unwrap();
    let dir = tempdir("sqlite3_crate_check");
    let emitted = emit_crate(
        &output.projection,
        &cfg,
        &dir,
        OutputMode::Crate,
        OverwritePolicy::Overwrite,
    )
    .unwrap();

    let rustc_args = std::fs::read_to_string(emitted.root.join("rustc-link-args.txt")).unwrap();
    let check = cargo_check(&emitted.root);

    assert!(rustc_args.contains("sqlite3"));
    assert!(check.status.success());
}

// ---- openssl ----

#[test]
fn openssl_full_pipeline() {
    let r = run_full_pipeline(openssl::openssl_package(), "openssl_sys");

    assert!(
        r.item_count >= 60,
        "expected ≥60 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("pub fn SSL_CTX_new"));
    assert!(r.source.contains("pub fn SSL_new"));
    assert!(r.source.contains("pub fn SSL_read"));
    assert!(r.source.contains("pub fn SSL_write"));
    assert!(r.source.contains("pub fn SSL_shutdown"));
    assert!(r.source.contains("pub fn EVP_sha256"));
    assert!(r.source.contains("pub fn BN_new"));
    assert!(r.source.contains("pub fn ERR_get_error"));
    // opaque types
    assert!(r.source.contains("pub struct SSL"));
    assert!(r.source.contains("pub struct X509"));
    assert!(r.source.contains("pub struct EVP_PKEY"));
    assert!(r.source.contains("pub struct BIGNUM"));
    // link: ssl + crypto
    assert!(r.link_libs >= 2);
}

#[test]
fn openssl_mostly_opaque() {
    let r = run_full_pipeline(openssl::openssl_package(), "openssl_sys");
    // OpenSSL is almost entirely opaque — most structs should use _opaque pattern
    let opaque_count = r.source.matches("_opaque: [u8; 0]").count();
    assert!(
        opaque_count >= 20,
        "expected ≥20 opaque structs, got {opaque_count}"
    );
}

// ---- freetype ----

#[test]
fn freetype_full_pipeline() {
    let r = run_full_pipeline(freetype::freetype_package(), "freetype_sys");

    assert!(
        r.item_count >= 30,
        "expected ≥30 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("pub fn FT_Init_FreeType"));
    assert!(r.source.contains("pub fn FT_Done_FreeType"));
    assert!(r.source.contains("pub fn FT_New_Face"));
    assert!(r.source.contains("pub fn FT_Load_Glyph"));
    assert!(r.source.contains("pub fn FT_Render_Glyph"));
    assert!(r.source.contains("pub struct FT_Vector"));
    assert!(r.source.contains("pub struct FT_Bitmap"));
    assert!(r.source.contains("pub type FT_Error"));
    assert!(r.link_libs >= 1);
}

#[test]
fn freetype_balanced_braces() {
    let r = run_full_pipeline(freetype::freetype_package(), "freetype_sys");
    let opens = r.source.matches('{').count();
    let closes = r.source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in freetype output");
}
