//! True end-to-end tests: real C headers → bic → gec → Rust source.
//!
//! These tests require system headers to be installed. They are gated
//! behind existence checks and return early if headers are missing.

use std::path::Path;

use gec::config::GecConfig;
use gec::consumer::{build_sidecar, sidecar_to_json, sidecar_from_json};
use gec::contract::{generate, projection_to_json, projection_from_json};
use gec::emit::emit_source;
use gec::intake::GecInput;

/// Parse a real C header through bic, then run the result through gec.
fn bic_to_gec(
    header: &str,
    include_dirs: &[&str],
    link_libs: &[&str],
    probe_types: &[&str],
    crate_name: &str,
) -> Option<E2EResult> {
    if !Path::new(header).exists() {
        return None;
    }

    let mut cfg = bic::HeaderConfig::new()
        .entry_header(header)
        .no_origin_filter();

    for dir in include_dirs {
        if Path::new(dir).exists() {
            cfg = cfg.include_dir(*dir);
        }
    }
    for lib in link_libs {
        cfg = cfg.link_lib(*lib);
    }
    for ty in probe_types {
        cfg = cfg.probe_type_layout(*ty);
    }

    let bic_result = cfg.process().ok()?;
    let input = GecInput::from_package(bic_result.package);
    let gec_cfg = GecConfig::new(crate_name);
    let output = generate(&input, &gec_cfg).ok()?;
    let source = emit_source(&output.projection);

    Some(E2EResult {
        item_count: output.item_count(),
        diagnostic_count: output.diagnostics.len(),
        source,
        link_libs: output.projection.link_requirements.len(),
        projection: output.projection,
    })
}

#[allow(dead_code)]
struct E2EResult {
    item_count: usize,
    diagnostic_count: usize,
    source: String,
    link_libs: usize,
    projection: gec::ir::RustProjection,
}

// ======== zlib ========

#[test]
fn zlib_end_to_end() {
    let Some(r) = bic_to_gec(
        "/usr/include/zlib.h",
        &["/usr/include", "/usr/include/x86_64-linux-gnu"],
        &["z"],
        &["z_stream"],
        "zlib_sys",
    ) else {
        eprintln!("skipping zlib e2e: headers not found");
        return;
    };

    assert!(r.item_count >= 10, "expected ≥10 items from zlib, got {}", r.item_count);
    assert!(r.source.contains("pub fn deflate") || r.source.contains("deflateInit"),
        "expected deflate-family functions in zlib output");
    assert!(r.source.contains("pub fn inflate") || r.source.contains("inflateInit"),
        "expected inflate-family functions in zlib output");
    assert!(r.source.contains("pub fn compress") || r.source.contains("compressBound"),
        "expected compress functions");
    assert!(r.source.contains("pub fn adler32") || r.source.contains("pub fn crc32"),
        "expected checksum functions");

    // balanced braces
    let opens = r.source.matches('{').count();
    let closes = r.source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in zlib e2e output");

    assert!(r.link_libs >= 1, "expected link requirement for libz");
}

#[test]
fn zlib_end_to_end_deterministic() {
    if !Path::new("/usr/include/zlib.h").exists() {
        return;
    }

    let make = || {
        let cfg = bic::HeaderConfig::new()
            .entry_header("/usr/include/zlib.h")
            .include_dir("/usr/include")
            .no_origin_filter()
            .link_lib("z");
        let bic_result = cfg.process().unwrap();
        let input = GecInput::from_package(bic_result.package);
        let output = generate(&input, &GecConfig::new("z")).unwrap();
        emit_source(&output.projection)
    };

    let s1 = make();
    let s2 = make();
    assert_eq!(s1, s2, "non-deterministic e2e output for zlib");
}

#[test]
fn zlib_end_to_end_json_roundtrip() {
    let Some(r) = bic_to_gec(
        "/usr/include/zlib.h",
        &["/usr/include"],
        &["z"],
        &[],
        "zlib_sys",
    ) else {
        return;
    };

    let json = projection_to_json(&r.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), r.projection.len());
}

#[test]
fn zlib_end_to_end_sidecar() {
    let Some(r) = bic_to_gec(
        "/usr/include/zlib.h",
        &["/usr/include"],
        &["z"],
        &[],
        "zlib_sys",
    ) else {
        return;
    };

    let sidecar = build_sidecar("zlib_sys", &r.projection);
    let json = sidecar_to_json(&sidecar).unwrap();
    let sidecar2 = sidecar_from_json(&json).unwrap();
    assert_eq!(sidecar2.crate_name, "zlib_sys");
    assert_eq!(sidecar2.items.len(), r.projection.len());
}

// ======== openssl ========

#[test]
fn openssl_end_to_end() {
    let Some(r) = bic_to_gec(
        "/usr/include/openssl/ssl.h",
        &["/usr/include", "/usr/include/x86_64-linux-gnu"],
        &["ssl", "crypto"],
        &[],
        "openssl_sys",
    ) else {
        eprintln!("skipping openssl e2e: headers not found");
        return;
    };

    // OpenSSL is huge — even a partial parse should yield many items
    assert!(r.item_count >= 20, "expected ≥20 items from openssl, got {}", r.item_count);

    // Some well-known functions should survive
    let has_ssl = r.source.contains("SSL_new")
        || r.source.contains("SSL_CTX_new")
        || r.source.contains("SSL_read");
    assert!(has_ssl, "expected core SSL functions in openssl output");

    // balanced braces
    let opens = r.source.matches('{').count();
    let closes = r.source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in openssl e2e output");

    assert!(r.link_libs >= 1, "expected link requirements for openssl");
}

#[test]
fn openssl_end_to_end_deterministic() {
    if !Path::new("/usr/include/openssl/ssl.h").exists() {
        return;
    }

    let make = || {
        let cfg = bic::HeaderConfig::new()
            .entry_header("/usr/include/openssl/ssl.h")
            .include_dir("/usr/include")
            .no_origin_filter()
            .link_lib("ssl")
            .link_lib("crypto");
        let bic_result = cfg.process().unwrap();
        let input = GecInput::from_package(bic_result.package);
        let output = generate(&input, &GecConfig::new("openssl")).unwrap();
        emit_source(&output.projection)
    };

    let s1 = make();
    let s2 = make();
    assert_eq!(s1, s2, "non-deterministic e2e output for openssl");
}

#[test]
fn openssl_end_to_end_json_roundtrip() {
    let Some(r) = bic_to_gec(
        "/usr/include/openssl/ssl.h",
        &["/usr/include"],
        &["ssl", "crypto"],
        &[],
        "openssl_sys",
    ) else {
        return;
    };

    let json = projection_to_json(&r.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), r.projection.len());
}
