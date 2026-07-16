#![cfg(feature = "system-tests")]

//! True end-to-end tests: real C headers -> linc -> gerc -> Rust source.
//!
//! These tests require system headers to be installed. In `optional` mode,
//! missing prerequisites print an explicit `SKIP` and return. In `required`
//! mode, every missing prerequisite is a test failure. Set the mode with
//! `GERC_SYSTEM_TEST_MODE=optional|required`.
//!
//! Categories covered:
//!   - Compression: zlib
//!   - Crypto/TLS: OpenSSL
//!   - Networking: libpcap, libcurl, Linux sockets/netlink
//!   - Linux kernel UAPI: input subsystem, CAN bus, perf_event
//!   - Graphics: libpng, X11/Xlib
//!   - Audio: ALSA
//!   - Text UI: ncurses
//!   - XML: libxml2
//!   - POSIX/libc: stdio, stdlib, string, math, pthreads, dlfcn, signal, mman, epoll
//!   - Combined multi-header surfaces

mod common;

#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use std::path::{Path, PathBuf};

use gerc::config::GercConfig;
use gerc::consumer::{
    build_sidecar, sidecar_from_json, sidecar_to_json, FolConsumer, GercConsumer,
};
use gerc::contract::{generate, projection_from_json, projection_to_json};
use gerc::emit::emit_source;
use gerc::intake::GercInput;

const INCLUDE: &[&str] = &["/usr/include", "/usr/include/x86_64-linux-gnu"];
const CONVENTIONAL_INCLUDE_ROOTS: &[&str] = &["/usr/include/x86_64-linux-gnu", "/usr/include"];

#[derive(Clone, Copy)]
enum SystemTestMode {
    Optional,
    Required,
}

fn system_test_mode() -> SystemTestMode {
    match std::env::var("GERC_SYSTEM_TEST_MODE").as_deref() {
        Ok("required") => SystemTestMode::Required,
        Ok("optional") | Err(_) => SystemTestMode::Optional,
        Ok(mode) => {
            panic!("invalid GERC_SYSTEM_TEST_MODE '{mode}'; expected 'optional' or 'required'")
        }
    }
}

fn handle_missing_prerequisite(reason: impl AsRef<str>) {
    let reason = reason.as_ref();
    match system_test_mode() {
        SystemTestMode::Required => panic!("FAIL system fixture: {reason}"),
        SystemTestMode::Optional => eprintln!("SKIP system fixture: {reason}"),
    }
}

fn push_unique_existing_dir(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() && !paths.contains(&path) {
        paths.push(path);
    }
}

fn relative_to_conventional_root(path: &Path) -> Option<&Path> {
    CONVENTIONAL_INCLUDE_ROOTS
        .iter()
        .find_map(|root| path.strip_prefix(root).ok())
}

fn include_search_paths(requested: &[&str]) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    for variable in ["CPATH", "C_INCLUDE_PATH"] {
        if let Some(value) = std::env::var_os(variable) {
            for path in std::env::split_paths(&value) {
                push_unique_existing_dir(&mut roots, path);
            }
        }
    }
    for root in CONVENTIONAL_INCLUDE_ROOTS {
        push_unique_existing_dir(&mut roots, PathBuf::from(root));
    }

    let base_roots = roots.clone();
    for requested_dir in requested {
        let requested_dir = Path::new(requested_dir);
        push_unique_existing_dir(&mut roots, requested_dir.to_path_buf());

        if let Some(relative) = relative_to_conventional_root(requested_dir) {
            for root in &base_roots {
                push_unique_existing_dir(&mut roots, root.join(relative));
            }
        }
    }

    roots
}

fn resolve_header(
    header: &Path,
    requested_include_dirs: &[&str],
    include_dirs: &[PathBuf],
) -> Option<PathBuf> {
    if header.is_file() {
        return Some(header.to_path_buf());
    }

    let mut relative_candidates = Vec::new();
    if header.is_relative() {
        relative_candidates.push(header.to_path_buf());
    }
    for root in CONVENTIONAL_INCLUDE_ROOTS {
        if let Ok(relative) = header.strip_prefix(root) {
            let relative = relative.to_path_buf();
            if !relative_candidates.contains(&relative) {
                relative_candidates.push(relative);
            }
        }
    }
    for requested_dir in requested_include_dirs {
        if let Ok(relative) = header.strip_prefix(requested_dir) {
            let relative = relative.to_path_buf();
            if !relative_candidates.contains(&relative) {
                relative_candidates.push(relative);
            }
        }
    }

    include_dirs.iter().find_map(|include_dir| {
        relative_candidates.iter().find_map(|relative| {
            let candidate = include_dir.join(relative);
            candidate.is_file().then_some(candidate)
        })
    })
}

fn discover_header(
    candidates: &[&str],
    include_dirs: &[&str],
    missing_description: &str,
) -> Option<PathBuf> {
    let search_paths = include_search_paths(include_dirs);
    for candidate in candidates {
        if let Some(header) = resolve_header(Path::new(candidate), include_dirs, &search_paths) {
            return Some(header);
        }
    }

    handle_missing_prerequisite(missing_description);
    None
}

fn prerequisites_available(include_dirs: &[PathBuf]) -> bool {
    let _ = system_test_mode();

    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    match std::process::Command::new(&compiler)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            handle_missing_prerequisite(format!(
                "C compiler '{:?}' returned status {}",
                compiler, output.status
            ));
            return false;
        }
        Err(error) => {
            handle_missing_prerequisite(format!(
                "C compiler '{:?}' is unavailable: {error}",
                compiler
            ));
            return false;
        }
    }

    if include_dirs.is_empty() {
        handle_missing_prerequisite(
            "no include search directory exists in CPATH, C_INCLUDE_PATH, or conventional roots",
        );
        return false;
    }

    true
}

/// Parse a real C header through linc, then run the result through gerc.
fn bic_to_gerc(
    header: impl AsRef<Path>,
    include_dirs: &[&str],
    link_libs: &[&str],
    probe_types: &[&str],
    crate_name: &str,
) -> Option<E2EResult> {
    let include_search_paths = include_search_paths(include_dirs);
    if !prerequisites_available(&include_search_paths) {
        return None;
    }
    let requested_header = header.as_ref();
    let Some(header) = resolve_header(requested_header, include_dirs, &include_search_paths) else {
        handle_missing_prerequisite(format!(
            "required header is absent from configured include paths: {}",
            requested_header.display()
        ));
        return None;
    };

    eprintln!("RUN system fixture: {}", header.display());

    let mut cfg = linc::raw_headers::HeaderConfig::new()
        .entry_header(&header)
        .no_origin_filter();

    for dir in &include_search_paths {
        cfg = cfg.include_dir(dir);
    }
    for lib in link_libs {
        cfg = cfg.link_lib(*lib);
    }
    for ty in probe_types {
        cfg = cfg.probe_type_layout(*ty);
    }

    let linc_result = linc_common::process(&cfg).unwrap_or_else(|error| {
        panic!(
            "LINC failed for system header '{}': {error:?}",
            header.display()
        )
    });
    assert!(
        !linc_result.package.items.is_empty(),
        "LINC produced no declarations for system header '{}'; diagnostics: {:#?}",
        header.display(),
        linc_result.package.diagnostics
    );
    let input = GercInput::from_source_package(common::from_binding_package(&linc_result.package));
    let gerc_cfg = GercConfig::new(crate_name);
    let output = generate(&input, &gerc_cfg).unwrap_or_else(|error| {
        panic!(
            "GERC failed for system header '{}': {error}",
            header.display()
        )
    });
    let source = emit_source(&output.projection);

    Some(E2EResult {
        item_count: output.item_count(),
        diagnostic_count: output.diagnostics.len(),
        source,
        link_libs: output.projection.link_requirements.len(),
        projection: output.projection,
    })
}

/// Same as linc_to_gerc but takes multiple headers.
fn bic_to_gerc_multi<P: AsRef<Path>>(
    headers: &[P],
    include_dirs: &[&str],
    link_libs: &[&str],
    probe_types: &[&str],
    crate_name: &str,
) -> Option<E2EResult> {
    let include_search_paths = include_search_paths(include_dirs);
    if !prerequisites_available(&include_search_paths) {
        return None;
    }
    let headers = headers
        .iter()
        .map(|header| {
            let requested_header = header.as_ref();
            resolve_header(requested_header, include_dirs, &include_search_paths).or_else(|| {
                handle_missing_prerequisite(format!(
                    "required header is absent from configured include paths: {}",
                    requested_header.display()
                ));
                None
            })
        })
        .collect::<Option<Vec<_>>>()?;

    eprintln!(
        "RUN system fixture: {}",
        headers
            .iter()
            .map(|header| header.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut cfg = linc::raw_headers::HeaderConfig::new().no_origin_filter();

    for header in &headers {
        cfg = cfg.entry_header(header);
    }
    for dir in &include_search_paths {
        cfg = cfg.include_dir(dir);
    }
    for lib in link_libs {
        cfg = cfg.link_lib(*lib);
    }
    for ty in probe_types {
        cfg = cfg.probe_type_layout(*ty);
    }

    let linc_result = linc_common::process(&cfg).unwrap_or_else(|error| {
        panic!(
            "LINC failed for system headers '{}': {error:?}",
            headers
                .iter()
                .map(|header| header.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    assert!(
        !linc_result.package.items.is_empty(),
        "LINC produced no declarations for system headers '{}'; diagnostics: {:#?}",
        headers
            .iter()
            .map(|header| header.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        linc_result.package.diagnostics
    );
    let input = GercInput::from_source_package(common::from_binding_package(&linc_result.package));
    let gerc_cfg = GercConfig::new(crate_name);
    let output = generate(&input, &gerc_cfg).unwrap_or_else(|error| {
        panic!(
            "GERC failed for system headers '{}': {error}",
            headers
                .iter()
                .map(|header| header.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    let source = emit_source(&output.projection);

    Some(E2EResult {
        item_count: output.item_count(),
        diagnostic_count: output.diagnostics.len(),
        source,
        link_libs: output.projection.link_requirements.len(),
        projection: output.projection,
    })
}

fn assert_balanced(source: &str, label: &str) {
    let opens = source.matches('{').count();
    let closes = source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in {label} output");
}

fn assert_deterministic(
    header: impl AsRef<Path> + Copy,
    include_dirs: &[&str],
    link_libs: &[&str],
    crate_name: &str,
) {
    let Some(first) = bic_to_gerc(header, include_dirs, link_libs, &[], crate_name) else {
        return;
    };
    let Some(second) = bic_to_gerc(header, include_dirs, link_libs, &[], crate_name) else {
        return;
    };
    assert_eq!(
        first.source, second.source,
        "non-deterministic e2e output for {crate_name}"
    );
}

#[allow(dead_code)]
struct E2EResult {
    item_count: usize,
    diagnostic_count: usize,
    source: String,
    link_libs: usize,
    projection: gerc::ir::RustProjection,
}

// ═══════════════════════════════════════════════════════════
//  COMPRESSION: zlib
// ═══════════════════════════════════════════════════════════

#[test]
fn zlib_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/zlib.h",
        INCLUDE,
        &["z"],
        &["z_stream"],
        "zlib_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 10,
        "zlib: expected ≥10 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("pub fn deflate") || r.source.contains("deflateInit"));
    assert!(r.source.contains("pub fn inflate") || r.source.contains("inflateInit"));
    assert!(r.source.contains("pub fn compress") || r.source.contains("compressBound"));
    assert!(r.source.contains("pub fn adler32") || r.source.contains("pub fn crc32"));
    assert_balanced(&r.source, "zlib");
    assert!(r.link_libs >= 1);
}

#[test]
fn zlib_e2e_deterministic() {
    assert_deterministic("/usr/include/zlib.h", INCLUDE, &["z"], "zlib_sys");
}

#[test]
fn zlib_e2e_json_roundtrip() {
    let Some(r) = bic_to_gerc("/usr/include/zlib.h", INCLUDE, &["z"], &[], "zlib_sys") else {
        return;
    };
    let json = projection_to_json(&r.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), r.projection.len());
}

#[test]
fn zlib_e2e_sidecar() {
    let Some(r) = bic_to_gerc("/usr/include/zlib.h", INCLUDE, &["z"], &[], "zlib_sys") else {
        return;
    };
    let sidecar = build_sidecar("zlib_sys", &r.projection);
    let json = sidecar_to_json(&sidecar).unwrap();
    let s2 = sidecar_from_json(&json).unwrap();
    assert_eq!(s2.crate_name, "zlib_sys");
    assert_eq!(s2.items.len(), r.projection.len());
}

// ═══════════════════════════════════════════════════════════
//  CRYPTO/TLS: OpenSSL
// ═══════════════════════════════════════════════════════════

#[test]
fn openssl_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/openssl/ssl.h",
        INCLUDE,
        &["ssl", "crypto"],
        &[],
        "openssl_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 20,
        "openssl: expected ≥20 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("SSL_new")
            || r.source.contains("SSL_CTX_new")
            || r.source.contains("SSL_read")
    );
    assert_balanced(&r.source, "openssl");
    assert!(r.link_libs >= 1);
}

#[test]
fn openssl_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/openssl/ssl.h",
        INCLUDE,
        &["ssl", "crypto"],
        "openssl_sys",
    );
}

#[test]
fn openssl_e2e_json_roundtrip() {
    let Some(r) = bic_to_gerc(
        "/usr/include/openssl/ssl.h",
        INCLUDE,
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

#[test]
fn openssl_e2e_fol_consumer() {
    let Some(r) = bic_to_gerc(
        "/usr/include/openssl/ssl.h",
        INCLUDE,
        &["ssl", "crypto"],
        &[],
        "openssl_sys",
    ) else {
        return;
    };
    let consumer = FolConsumer;
    let report = consumer.inspect(&r.projection);
    assert_eq!(report.consumer_name, "fol-interloop-rust");
    assert!(report.items_inspected >= 20);
}

#[test]
fn openssl_e2e_emits_expected_link_directives() {
    let Some(r) = bic_to_gerc(
        "/usr/include/openssl/ssl.h",
        INCLUDE,
        &["ssl", "crypto"],
        &[],
        "openssl_sys",
    ) else {
        return;
    };

    let build_rs = gerc::emit_build_rs(&r.projection);
    let rustc_args = gerc::emit_rustc_args(&r.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=ssl"));
    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=crypto"));
    assert!(rustc_args.contains("-ldylib=ssl"));
    assert!(rustc_args.contains("-ldylib=crypto"));
}

// ═══════════════════════════════════════════════════════════
//  NETWORKING: libpcap
// ═══════════════════════════════════════════════════════════

#[test]
fn pcap_e2e() {
    let Some(header) = discover_header(
        &["/usr/include/pcap/pcap.h", "/usr/include/pcap.h"],
        INCLUDE,
        "required libpcap header is absent from configured include paths",
    ) else {
        return;
    };

    let Some(r) = bic_to_gerc(&header, INCLUDE, &["pcap"], &[], "pcap_sys") else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "pcap: expected ≥5 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("pcap_open_live")
            || r.source.contains("pcap_loop")
            || r.source.contains("pcap_close")
    );
    assert_balanced(&r.source, "pcap");
    assert!(r.link_libs >= 1);
}

#[test]
fn pcap_e2e_deterministic() {
    let Some(header) = discover_header(
        &["/usr/include/pcap/pcap.h", "/usr/include/pcap.h"],
        INCLUDE,
        "required libpcap header is absent from configured include paths",
    ) else {
        return;
    };
    assert_deterministic(&header, INCLUDE, &["pcap"], "pcap_sys");
}

#[test]
fn libcurl_e2e() {
    let Some(header) = discover_header(
        &[
            "/usr/include/curl/curl.h",
            "/usr/include/x86_64-linux-gnu/curl/curl.h",
        ],
        INCLUDE,
        "required libcurl header is absent from configured include paths",
    ) else {
        return;
    };

    let Some(r) = bic_to_gerc(
        &header,
        INCLUDE,
        &["curl"],
        &["struct curl_blob"],
        "curl_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 20,
        "libcurl: expected ≥20 items, got {}",
        r.item_count
    );
    assert!(r.source.contains("curl_easy_init"));
    assert!(r.source.contains("curl_easy_setopt"));
    assert_balanced(&r.source, "libcurl");
}

#[test]
fn libcurl_e2e_deterministic() {
    let Some(header) = discover_header(
        &[
            "/usr/include/curl/curl.h",
            "/usr/include/x86_64-linux-gnu/curl/curl.h",
        ],
        INCLUDE,
        "required libcurl header is absent from configured include paths",
    ) else {
        return;
    };

    assert_deterministic(&header, INCLUDE, &["curl"], "curl_sys");
}

#[test]
fn libcurl_e2e_emits_expected_link_directives() {
    let Some(header) = discover_header(
        &[
            "/usr/include/curl/curl.h",
            "/usr/include/x86_64-linux-gnu/curl/curl.h",
        ],
        INCLUDE,
        "required libcurl header is absent from configured include paths",
    ) else {
        return;
    };

    let Some(r) = bic_to_gerc(
        &header,
        INCLUDE,
        &["curl"],
        &["struct curl_blob"],
        "curl_sys",
    ) else {
        return;
    };

    let build_rs = gerc::emit_build_rs(&r.projection);
    let rustc_args = gerc::emit_rustc_args(&r.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=curl"));
    assert!(rustc_args.contains("-ldylib=curl"));
}

// ═══════════════════════════════════════════════════════════
//  GRAPHICS: libpng
// ═══════════════════════════════════════════════════════════

#[test]
fn libpng_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/png.h", INCLUDE, &["png"], &[], "png_sys") else {
        return;
    };

    assert!(
        r.item_count >= 10,
        "libpng: expected ≥10 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("png_create_read_struct")
            || r.source.contains("png_read_png")
            || r.source.contains("png_init_io")
    );
    assert_balanced(&r.source, "libpng");
}

#[test]
fn libpng_e2e_deterministic() {
    assert_deterministic("/usr/include/png.h", INCLUDE, &["png"], "png_sys");
}

#[test]
fn libpng_e2e_emits_expected_link_directives() {
    let Some(r) = bic_to_gerc("/usr/include/png.h", INCLUDE, &["png"], &[], "png_sys") else {
        return;
    };

    let build_rs = gerc::emit_build_rs(&r.projection);
    let rustc_args = gerc::emit_rustc_args(&r.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=png"));
    assert!(rustc_args.contains("-ldylib=png"));
}

// ═══════════════════════════════════════════════════════════
//  GRAPHICS: X11/Xlib
// ═══════════════════════════════════════════════════════════

#[test]
fn xlib_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/X11/Xlib.h", INCLUDE, &["X11"], &[], "x11_sys") else {
        return;
    };

    assert!(
        r.item_count >= 10,
        "xlib: expected ≥10 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("XOpenDisplay")
            || r.source.contains("XCreateWindow")
            || r.source.contains("XNextEvent")
            || r.source.contains("XCloseDisplay")
    );
    assert_balanced(&r.source, "xlib");
}

#[test]
fn xlib_e2e_deterministic() {
    assert_deterministic("/usr/include/X11/Xlib.h", INCLUDE, &["X11"], "x11_sys");
}

// ═══════════════════════════════════════════════════════════
//  AUDIO: ALSA
// ═══════════════════════════════════════════════════════════

#[test]
fn alsa_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/alsa/asoundlib.h",
        INCLUDE,
        &["asound"],
        &[],
        "alsa_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 10,
        "alsa: expected ≥10 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("snd_pcm_open")
            || r.source.contains("snd_pcm_close")
            || r.source.contains("snd_pcm_hw_params")
    );
    assert_balanced(&r.source, "alsa");
    assert!(r.link_libs >= 1);
}

#[test]
fn alsa_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/alsa/asoundlib.h",
        INCLUDE,
        &["asound"],
        "alsa_sys",
    );
}

// ═══════════════════════════════════════════════════════════
//  TEXT UI: ncurses
// ═══════════════════════════════════════════════════════════

#[test]
fn ncurses_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/ncurses.h",
        INCLUDE,
        &["ncurses"],
        &[],
        "ncurses_sys",
    ) else {
        return;
    };

    // ncurses is heavily macro-based; gerc may only see a subset of the API
    assert!(
        r.item_count >= 1,
        "ncurses: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "ncurses");
}

#[test]
fn ncurses_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/ncurses.h",
        INCLUDE,
        &["ncurses"],
        "ncurses_sys",
    );
}

// ═══════════════════════════════════════════════════════════
//  XML: libxml2
// ═══════════════════════════════════════════════════════════

#[test]
fn libxml2_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/libxml2/libxml/parser.h",
        &[
            "/usr/include",
            "/usr/include/x86_64-linux-gnu",
            "/usr/include/libxml2",
        ],
        &["xml2"],
        &[],
        "xml2_sys",
    ) else {
        return;
    };

    // libxml2 has complex macro/typedef layering; check we got something
    assert!(
        r.item_count >= 1,
        "libxml2: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "libxml2");
}

#[test]
fn libxml2_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/libxml2/libxml/parser.h",
        &[
            "/usr/include",
            "/usr/include/x86_64-linux-gnu",
            "/usr/include/libxml2",
        ],
        &["xml2"],
        "xml2_sys",
    );
}

#[test]
fn libxml2_e2e_emits_expected_link_directives() {
    let Some(r) = bic_to_gerc(
        "/usr/include/libxml2/libxml/parser.h",
        &[
            "/usr/include",
            "/usr/include/x86_64-linux-gnu",
            "/usr/include/libxml2",
        ],
        &["xml2"],
        &[],
        "xml2_sys",
    ) else {
        return;
    };

    let build_rs = gerc::emit_build_rs(&r.projection);
    let rustc_args = gerc::emit_rustc_args(&r.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=xml2"));
    assert!(rustc_args.contains("-ldylib=xml2"));
}

// ═══════════════════════════════════════════════════════════
//  LINUX KERNEL UAPI: input subsystem
// ═══════════════════════════════════════════════════════════

#[test]
fn linux_input_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/linux/input.h",
        INCLUDE,
        &[],
        &[],
        "linux_input_sys",
    ) else {
        return;
    };

    // kernel UAPI structs often have bitfields/packed layouts that get gated
    assert!(
        r.item_count >= 1,
        "linux/input: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "linux/input");
}

#[test]
fn linux_input_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/linux/input.h",
        INCLUDE,
        &[],
        "linux_input_sys",
    );
}

// ═══════════════════════════════════════════════════════════
//  LINUX KERNEL UAPI: CAN bus
// ═══════════════════════════════════════════════════════════

#[test]
fn linux_can_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/linux/can.h",
        INCLUDE,
        &[],
        &[],
        "linux_can_sys",
    ) else {
        return;
    };

    // CAN structs often have unions/bitfields that get gated
    assert!(
        r.item_count >= 1,
        "linux/can: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "linux/can");
}

// ═══════════════════════════════════════════════════════════
//  LINUX KERNEL UAPI: netlink
// ═══════════════════════════════════════════════════════════

#[test]
fn linux_netlink_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/linux/netlink.h",
        INCLUDE,
        &[],
        &[],
        "linux_netlink_sys",
    ) else {
        return;
    };

    // netlink structs may have flexible arrays or packed layouts that get gated
    assert!(
        r.item_count >= 1,
        "linux/netlink: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "linux/netlink");
}

// ═══════════════════════════════════════════════════════════
//  LINUX KERNEL UAPI: perf_event
// ═══════════════════════════════════════════════════════════

#[test]
fn linux_perf_event_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/linux/perf_event.h",
        INCLUDE,
        &[],
        &[],
        "linux_perf_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "linux/perf_event: expected ≥1 items, got {}",
        r.item_count
    );
    // perf_event_attr is the primary struct — may or may not survive gating
    // due to bitfields, but the pipeline should not panic
    assert_balanced(&r.source, "linux/perf_event");
}

// ═══════════════════════════════════════════════════════════
//  POSIX/libc: stdio
// ═══════════════════════════════════════════════════════════

#[test]
fn stdio_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/stdio.h", INCLUDE, &["c"], &[], "stdio_sys") else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "stdio: expected ≥5 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("printf")
            || r.source.contains("fprintf")
            || r.source.contains("fopen")
            || r.source.contains("fclose")
    );
    assert_balanced(&r.source, "stdio");
}

#[test]
fn stdio_e2e_deterministic() {
    assert_deterministic("/usr/include/stdio.h", INCLUDE, &["c"], "stdio_sys");
}

// ═══════════════════════════════════════════════════════════
//  POSIX/libc: stdlib
// ═══════════════════════════════════════════════════════════

#[test]
fn stdlib_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/stdlib.h", INCLUDE, &["c"], &[], "stdlib_sys") else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "stdlib: expected ≥5 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("malloc")
            || r.source.contains("free")
            || r.source.contains("realloc")
            || r.source.contains("exit")
    );
    assert_balanced(&r.source, "stdlib");
}

// ═══════════════════════════════════════════════════════════
//  POSIX/libc: string
// ═══════════════════════════════════════════════════════════

#[test]
fn string_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/string.h", INCLUDE, &["c"], &[], "string_sys") else {
        return;
    };

    assert!(
        r.item_count >= 3,
        "string: expected ≥3 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("memcpy")
            || r.source.contains("memset")
            || r.source.contains("strlen")
            || r.source.contains("strcmp")
    );
    assert_balanced(&r.source, "string");
}

// ═══════════════════════════════════════════════════════════
//  POSIX/libc: math
// ═══════════════════════════════════════════════════════════

#[test]
fn math_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/math.h", INCLUDE, &["m"], &[], "math_sys") else {
        return;
    };

    assert_balanced(&r.source, "math");
    assert!(r.link_libs >= 1);
}

// ═══════════════════════════════════════════════════════════
//  POSIX: pthreads
// ═══════════════════════════════════════════════════════════

#[test]
fn pthread_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/pthread.h",
        INCLUDE,
        &["pthread"],
        &[],
        "pthread_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 3,
        "pthread: expected ≥3 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("pthread_create")
            || r.source.contains("pthread_join")
            || r.source.contains("pthread_mutex_lock")
            || r.source.contains("pthread_mutex_init")
    );
    assert_balanced(&r.source, "pthread");
    assert!(r.link_libs >= 1);
}

#[test]
fn pthread_e2e_deterministic() {
    assert_deterministic(
        "/usr/include/pthread.h",
        INCLUDE,
        &["pthread"],
        "pthread_sys",
    );
}

// ═══════════════════════════════════════════════════════════
//  POSIX: dlfcn (dlopen/dlsym)
// ═══════════════════════════════════════════════════════════

#[test]
fn dlfcn_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/dlfcn.h", INCLUDE, &["dl"], &[], "dl_sys") else {
        return;
    };

    assert!(
        r.item_count >= 2,
        "dlfcn: expected ≥2 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("dlopen") || r.source.contains("dlsym") || r.source.contains("dlclose")
    );
    assert_balanced(&r.source, "dlfcn");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: signal
// ═══════════════════════════════════════════════════════════

#[test]
fn signal_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/signal.h", INCLUDE, &["c"], &[], "signal_sys") else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "signal: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "signal");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: unistd (read/write/fork/exec)
// ═══════════════════════════════════════════════════════════

#[test]
fn unistd_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/unistd.h", INCLUDE, &["c"], &[], "unistd_sys") else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "unistd: expected ≥5 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("pub fn read")
            || r.source.contains("pub fn write")
            || r.source.contains("pub fn close")
            || r.source.contains("pub fn fork")
    );
    assert_balanced(&r.source, "unistd");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: fcntl
// ═══════════════════════════════════════════════════════════

#[test]
fn fcntl_e2e() {
    let Some(r) = bic_to_gerc("/usr/include/fcntl.h", INCLUDE, &["c"], &[], "fcntl_sys") else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "fcntl: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "fcntl");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: sys/socket + netinet/in (combined)
// ═══════════════════════════════════════════════════════════

#[test]
fn socket_e2e() {
    let sock = "/usr/include/x86_64-linux-gnu/sys/socket.h";
    let netin = "/usr/include/netinet/in.h";
    let Some(r) = bic_to_gerc_multi(&[sock, netin], INCLUDE, &["c"], &[], "socket_sys") else {
        return;
    };

    assert!(
        r.item_count >= 3,
        "socket: expected ≥3 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("sockaddr")
            || r.source.contains("in_addr")
            || r.source.contains("socket")
            || r.source.contains("bind")
    );
    assert_balanced(&r.source, "socket");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: sys/mman (mmap)
// ═══════════════════════════════════════════════════════════

#[test]
fn mman_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/x86_64-linux-gnu/sys/mman.h",
        INCLUDE,
        &["c"],
        &[],
        "mman_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "mman: expected ≥1 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("mmap") || r.source.contains("munmap") || r.source.contains("mprotect")
    );
    assert_balanced(&r.source, "mman");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: sys/epoll
// ═══════════════════════════════════════════════════════════

#[test]
fn epoll_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        INCLUDE,
        &["c"],
        &[],
        "epoll_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "epoll: expected ≥1 items, got {}",
        r.item_count
    );
    assert!(
        r.source.contains("epoll_create")
            || r.source.contains("epoll_ctl")
            || r.source.contains("epoll_wait")
    );
    assert_balanced(&r.source, "epoll");
}

// ═══════════════════════════════════════════════════════════
//  POSIX: sys/stat
// ═══════════════════════════════════════════════════════════

#[test]
fn stat_e2e() {
    let Some(r) = bic_to_gerc(
        "/usr/include/x86_64-linux-gnu/sys/stat.h",
        INCLUDE,
        &["c"],
        &[],
        "stat_sys",
    ) else {
        return;
    };

    assert!(
        r.item_count >= 1,
        "stat: expected ≥1 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "stat");
}

// ═══════════════════════════════════════════════════════════
//  COMBINED: multi-header real-world surface
//  (unistd + socket + epoll + signal — a typical event loop)
// ═══════════════════════════════════════════════════════════

#[test]
fn combined_event_loop_e2e() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
        "/usr/include/fcntl.h",
        "/usr/include/errno.h",
    ];

    let Some(r) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "combined: expected ≥5 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "combined_event_loop");
    // should have functions from at least one header
    let source = &r.source;
    let mut found = 0;
    if source.contains("pub fn read") || source.contains("pub fn write") {
        found += 1;
    }
    if source.contains("epoll_create") || source.contains("epoll_ctl") {
        found += 1;
    }
    if source.contains("socket") || source.contains("bind") {
        found += 1;
    }
    if source.contains("pub fn signal")
        || source.contains("pub fn kill")
        || source.contains("sigaction")
    {
        found += 1;
    }
    assert!(
        found >= 1,
        "combined surface should contain functions from at least one header"
    );
}

#[test]
fn combined_event_loop_e2e_deterministic() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
    ];
    let Some(first) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };
    let Some(second) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };
    assert_eq!(
        first.source, second.source,
        "non-deterministic combined e2e output"
    );
}

#[test]
fn combined_event_loop_e2e_json_roundtrip() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
    ];
    let Some(r) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };
    let json = projection_to_json(&r.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), r.projection.len());
}

#[test]
fn combined_event_loop_e2e_sidecar() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
    ];
    let Some(r) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };
    let sidecar = build_sidecar("event_loop_sys", &r.projection);
    assert_eq!(sidecar.crate_name, "event_loop_sys");
    assert_eq!(sidecar.items.len(), r.projection.len());
}

#[test]
fn combined_event_loop_e2e_emits_expected_link_directives() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
    ];
    let Some(r) = bic_to_gerc_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };

    let build_rs = gerc::emit_build_rs(&r.projection);
    let rustc_args = gerc::emit_rustc_args(&r.projection);

    assert!(build_rs.contains("cargo:rustc-link-lib=dylib=c"));
    assert!(rustc_args.contains("-ldylib=c"));
}

// ═══════════════════════════════════════════════════════════
//  COMBINED: networking stack
//  (socket + netinet/in + netlink + pcap)
// ═══════════════════════════════════════════════════════════

#[test]
fn combined_networking_e2e() {
    let mut headers = vec![
        PathBuf::from("/usr/include/x86_64-linux-gnu/sys/socket.h"),
        PathBuf::from("/usr/include/netinet/in.h"),
        PathBuf::from("/usr/include/linux/netlink.h"),
    ];
    let Some(pcap_header) = discover_header(
        &["/usr/include/pcap/pcap.h", "/usr/include/pcap.h"],
        INCLUDE,
        "required libpcap header is absent from configured include paths",
    ) else {
        return;
    };
    headers.push(pcap_header);

    let Some(r) = bic_to_gerc_multi(&headers, INCLUDE, &["c", "pcap"], &[], "networking_sys")
    else {
        return;
    };

    assert!(
        r.item_count >= 5,
        "networking: expected ≥5 items, got {}",
        r.item_count
    );
    assert_balanced(&r.source, "combined_networking");
}

// ═══════════════════════════════════════════════════════════
//  COMBINED: full POSIX libc surface
//  (stdio + stdlib + string + unistd + fcntl + signal + math + pthread + dlfcn)
// ═══════════════════════════════════════════════════════════

#[test]
fn combined_full_libc_e2e() {
    let headers = [
        "/usr/include/stdio.h",
        "/usr/include/stdlib.h",
        "/usr/include/string.h",
        "/usr/include/unistd.h",
        "/usr/include/fcntl.h",
        "/usr/include/signal.h",
        "/usr/include/math.h",
        "/usr/include/pthread.h",
        "/usr/include/dlfcn.h",
        "/usr/include/errno.h",
    ];

    let Some(r) = bic_to_gerc_multi(
        &headers,
        INCLUDE,
        &["c", "m", "pthread", "dl"],
        &[],
        "libc_full_sys",
    ) else {
        return;
    };

    assert_balanced(&r.source, "full_libc");
    assert!(r.link_libs >= 1);
}

#[test]
fn combined_full_libc_e2e_deterministic() {
    let headers = [
        "/usr/include/stdio.h",
        "/usr/include/stdlib.h",
        "/usr/include/string.h",
        "/usr/include/unistd.h",
        "/usr/include/pthread.h",
    ];
    let Some(first) = bic_to_gerc_multi(&headers, INCLUDE, &["c", "pthread"], &[], "libc_full_sys")
    else {
        return;
    };
    let Some(second) =
        bic_to_gerc_multi(&headers, INCLUDE, &["c", "pthread"], &[], "libc_full_sys")
    else {
        return;
    };
    assert_eq!(
        first.source, second.source,
        "non-deterministic full libc e2e output"
    );
}

#[test]
fn combined_full_libc_e2e_fol_consumer() {
    let headers = [
        "/usr/include/stdio.h",
        "/usr/include/stdlib.h",
        "/usr/include/string.h",
        "/usr/include/unistd.h",
        "/usr/include/pthread.h",
        "/usr/include/dlfcn.h",
    ];
    let Some(r) = bic_to_gerc_multi(
        &headers,
        INCLUDE,
        &["c", "pthread", "dl"],
        &[],
        "libc_full_sys",
    ) else {
        return;
    };
    let consumer = FolConsumer;
    let report = consumer.inspect(&r.projection);
    assert_eq!(report.consumer_name, "fol-interloop-rust");
    assert!(report.items_inspected >= 20);
}
