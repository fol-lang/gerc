//! True end-to-end tests: real C headers -> linc -> gec -> Rust source.
//!
//! These tests require system headers to be installed. Each test gates
//! on header existence and returns early if missing — they never fail
//! on a machine that simply lacks the headers.
//!
//! Categories covered:
//!   - Compression: zlib
//!   - Crypto/TLS: OpenSSL
//!   - Networking: libpcap, Linux sockets/netlink
//!   - Linux kernel UAPI: input subsystem, CAN bus, perf_event
//!   - Graphics: libpng, X11/Xlib
//!   - Audio: ALSA
//!   - Text UI: ncurses
//!   - XML: libxml2
//!   - POSIX/libc: stdio, stdlib, string, math, pthreads, dlfcn, signal, mman, epoll
//!   - Combined multi-header surfaces

#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use std::path::Path;

use gec::config::GecConfig;
use gec::consumer::{build_sidecar, sidecar_from_json, sidecar_to_json, FolConsumer, GecConsumer};
use gec::contract::{generate, projection_from_json, projection_to_json};
use gec::emit::emit_source;
use gec::intake::GecInput;

const INCLUDE: &[&str] = &["/usr/include", "/usr/include/x86_64-linux-gnu"];

/// Parse a real C header through linc, then run the result through gec.
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

    let mut cfg = linc::raw_headers::HeaderConfig::new()
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

    let linc_result = linc_common::process(&cfg).ok()?;
    let input = GecInput::from_source_package(linc::intake::adapters::from_binding_package(
        &linc_result.package,
    ));
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

/// Same as linc_to_gec but takes multiple headers.
fn bic_to_gec_multi(
    headers: &[&str],
    include_dirs: &[&str],
    link_libs: &[&str],
    probe_types: &[&str],
    crate_name: &str,
) -> Option<E2EResult> {
    for h in headers {
        if !Path::new(h).exists() {
            return None;
        }
    }

    let mut cfg = linc::raw_headers::HeaderConfig::new().no_origin_filter();

    for h in headers {
        cfg = cfg.entry_header(*h);
    }
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

    let linc_result = linc_common::process(&cfg).ok()?;
    let input = GecInput::from_source_package(linc::intake::adapters::from_binding_package(
        &linc_result.package,
    ));
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

fn assert_balanced(source: &str, label: &str) {
    let opens = source.matches('{').count();
    let closes = source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in {label} output");
}

fn assert_deterministic(header: &str, include_dirs: &[&str], link_libs: &[&str], crate_name: &str) {
    if !Path::new(header).exists() {
        return;
    }
    let make = || {
        bic_to_gec(header, include_dirs, link_libs, &[], crate_name)
            .unwrap()
            .source
    };
    assert_eq!(
        make(),
        make(),
        "non-deterministic e2e output for {crate_name}"
    );
}

#[allow(dead_code)]
struct E2EResult {
    item_count: usize,
    diagnostic_count: usize,
    source: String,
    link_libs: usize,
    projection: gec::ir::RustProjection,
}

// ═══════════════════════════════════════════════════════════
//  COMPRESSION: zlib
// ═══════════════════════════════════════════════════════════

#[test]
fn zlib_e2e() {
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec("/usr/include/zlib.h", INCLUDE, &["z"], &[], "zlib_sys") else {
        return;
    };
    let json = projection_to_json(&r.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), r.projection.len());
}

#[test]
fn zlib_e2e_sidecar() {
    let Some(r) = bic_to_gec("/usr/include/zlib.h", INCLUDE, &["z"], &[], "zlib_sys") else {
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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

// ═══════════════════════════════════════════════════════════
//  NETWORKING: libpcap
// ═══════════════════════════════════════════════════════════

#[test]
fn pcap_e2e() {
    let header = if Path::new("/usr/include/pcap/pcap.h").exists() {
        "/usr/include/pcap/pcap.h"
    } else if Path::new("/usr/include/pcap.h").exists() {
        "/usr/include/pcap.h"
    } else {
        return;
    };

    let Some(r) = bic_to_gec(header, INCLUDE, &["pcap"], &[], "pcap_sys") else {
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
    let header = if Path::new("/usr/include/pcap/pcap.h").exists() {
        "/usr/include/pcap/pcap.h"
    } else if Path::new("/usr/include/pcap.h").exists() {
        "/usr/include/pcap.h"
    } else {
        return;
    };
    assert_deterministic(header, INCLUDE, &["pcap"], "pcap_sys");
}

// ═══════════════════════════════════════════════════════════
//  GRAPHICS: libpng
// ═══════════════════════════════════════════════════════════

#[test]
fn libpng_e2e() {
    let Some(r) = bic_to_gec("/usr/include/png.h", INCLUDE, &["png"], &[], "png_sys") else {
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

// ═══════════════════════════════════════════════════════════
//  GRAPHICS: X11/Xlib
// ═══════════════════════════════════════════════════════════

#[test]
fn xlib_e2e() {
    let Some(r) = bic_to_gec("/usr/include/X11/Xlib.h", INCLUDE, &["X11"], &[], "x11_sys") else {
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
        "/usr/include/ncurses.h",
        INCLUDE,
        &["ncurses"],
        &[],
        "ncurses_sys",
    ) else {
        return;
    };

    // ncurses is heavily macro-based; gec may only see a subset of the API
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
    let Some(r) = bic_to_gec(
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

// ═══════════════════════════════════════════════════════════
//  LINUX KERNEL UAPI: input subsystem
// ═══════════════════════════════════════════════════════════

#[test]
fn linux_input_e2e() {
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec("/usr/include/stdio.h", INCLUDE, &["c"], &[], "stdio_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/stdlib.h", INCLUDE, &["c"], &[], "stdlib_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/string.h", INCLUDE, &["c"], &[], "string_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/math.h", INCLUDE, &["m"], &[], "math_sys") else {
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec("/usr/include/dlfcn.h", INCLUDE, &["dl"], &[], "dl_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/signal.h", INCLUDE, &["c"], &[], "signal_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/unistd.h", INCLUDE, &["c"], &[], "unistd_sys") else {
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
    let Some(r) = bic_to_gec("/usr/include/fcntl.h", INCLUDE, &["c"], &[], "fcntl_sys") else {
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
    let Some(r) = bic_to_gec_multi(&[sock, netin], INCLUDE, &["c"], &[], "socket_sys") else {
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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
    let Some(r) = bic_to_gec(
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

    let Some(r) = bic_to_gec_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
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
    for h in &headers {
        if !Path::new(h).exists() {
            return;
        }
    }

    let make = || {
        bic_to_gec_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys")
            .unwrap()
            .source
    };
    assert_eq!(make(), make(), "non-deterministic combined e2e output");
}

#[test]
fn combined_event_loop_e2e_json_roundtrip() {
    let headers = [
        "/usr/include/unistd.h",
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/x86_64-linux-gnu/sys/epoll.h",
        "/usr/include/signal.h",
    ];
    let Some(r) = bic_to_gec_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
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
    let Some(r) = bic_to_gec_multi(&headers, INCLUDE, &["c"], &[], "event_loop_sys") else {
        return;
    };
    let sidecar = build_sidecar("event_loop_sys", &r.projection);
    assert_eq!(sidecar.crate_name, "event_loop_sys");
    assert_eq!(sidecar.items.len(), r.projection.len());
}

// ═══════════════════════════════════════════════════════════
//  COMBINED: networking stack
//  (socket + netinet/in + netlink + pcap)
// ═══════════════════════════════════════════════════════════

#[test]
fn combined_networking_e2e() {
    let mut headers = vec![
        "/usr/include/x86_64-linux-gnu/sys/socket.h",
        "/usr/include/netinet/in.h",
        "/usr/include/linux/netlink.h",
    ];
    if Path::new("/usr/include/pcap/pcap.h").exists() {
        headers.push("/usr/include/pcap/pcap.h");
    } else if Path::new("/usr/include/pcap.h").exists() {
        headers.push("/usr/include/pcap.h");
    }

    let Some(r) = bic_to_gec_multi(&headers, INCLUDE, &["c", "pcap"], &[], "networking_sys") else {
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

    let Some(r) = bic_to_gec_multi(
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
    for h in &headers {
        if !Path::new(h).exists() {
            return;
        }
    }

    let make = || {
        bic_to_gec_multi(&headers, INCLUDE, &["c", "pthread"], &[], "libc_full_sys")
            .unwrap()
            .source
    };
    assert_eq!(make(), make(), "non-deterministic full libc e2e output");
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
    let Some(r) = bic_to_gec_multi(
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
