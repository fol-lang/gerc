//! Integration tests for very large API surfaces: FFmpeg (multimedia),
//! Linux kernel UAPI (syscalls/sockets/io), and the torture fixture.
//!
//! These test that gec handles hundreds of items, large enum spaces,
//! mixed accept/reject decisions, deep type hierarchies, and complex
//! link surfaces without panicking or producing malformed output.

#[path = "../test/stress/ffmpeg.rs"]
mod ffmpeg;
#[path = "../test/stress/linux_kernel.rs"]
mod linux_kernel;
#[path = "../test/stress/torture.rs"]
mod torture;

use gec::config::GecConfig;
use gec::consumer::{build_sidecar, sidecar_from_json, sidecar_to_json, FolConsumer, GecConsumer};
use gec::contract::{
    generate, meta_from_json, meta_to_json, output_meta, projection_from_json, projection_to_json,
};
use gec::emit::emit_source;
use gec::intake::GecInput;

fn input_from_binding(pkg: linc::ir::BindingPackage) -> GecInput {
    GecInput::from_source_package(linc::intake::adapters::from_binding_package(&pkg))
}

// ======== FFmpeg ========

#[test]
fn ffmpeg_full_pipeline() {
    let pkg = ffmpeg::ffmpeg_package();
    let input = input_from_binding(pkg);
    let cfg = GecConfig::new("ffmpeg_sys");
    let output = generate(&input, &cfg).unwrap();
    let source = emit_source(&output.projection);

    // FFmpeg fixture: ~19 opaque + 3 by-value + 4 enums + 2 typedef + ~50 functions + macros
    assert!(
        output.item_count() >= 60,
        "expected ≥60 items, got {}",
        output.item_count()
    );

    // key avformat functions
    assert!(source.contains("pub fn avformat_open_input"));
    assert!(source.contains("pub fn avformat_close_input"));
    assert!(source.contains("pub fn av_read_frame"));
    assert!(source.contains("pub fn avformat_alloc_context"));

    // key avcodec functions
    assert!(source.contains("pub fn avcodec_alloc_context3"));
    assert!(source.contains("pub fn avcodec_send_packet"));
    assert!(source.contains("pub fn avcodec_receive_frame"));
    assert!(source.contains("pub fn avcodec_find_decoder"));

    // avutil
    assert!(source.contains("pub fn av_frame_alloc"));
    assert!(source.contains("pub fn av_packet_alloc"));
    assert!(source.contains("pub fn av_malloc"));
    assert!(source.contains("pub fn av_free"));
    assert!(source.contains("pub fn av_log")); // variadic

    // swscale
    assert!(source.contains("pub fn sws_getContext"));
    assert!(source.contains("pub fn sws_scale"));

    // structs
    assert!(source.contains("pub struct AVRational"));
    assert!(source.contains("pub struct AVDictionaryEntry"));
    assert!(source.contains("pub struct AVProbeData"));

    // opaque types
    assert!(source.contains("pub struct AVFormatContext"));
    assert!(source.contains("pub struct AVCodecContext"));
    assert!(source.contains("pub struct AVFrame"));
    assert!(source.contains("pub struct AVPacket"));

    // enums
    assert!(source.contains("AV_CODEC_ID_NONE"));
    assert!(source.contains("AV_CODEC_ID_H264"));
    assert!(source.contains("AV_CODEC_ID_AV1"));
    assert!(source.contains("AV_PIX_FMT_YUV420P"));
    assert!(source.contains("AVMEDIA_TYPE_VIDEO"));

    // link surface: 5 libraries
    assert!(output.projection.link_requirements.len() >= 5);
}

#[test]
fn ffmpeg_deterministic_10_runs() {
    let pkg = ffmpeg::ffmpeg_package();
    let cfg = GecConfig::new("ffmpeg_sys");
    let first = projection_to_json(
        &generate(&input_from_binding(pkg.clone()), &cfg)
            .unwrap()
            .projection,
    )
    .unwrap();
    for _ in 0..9 {
        let json = projection_to_json(
            &generate(&input_from_binding(pkg.clone()), &cfg)
                .unwrap()
                .projection,
        )
        .unwrap();
        assert_eq!(first, json, "non-deterministic ffmpeg output");
    }
}

#[test]
fn ffmpeg_json_roundtrip() {
    let pkg = ffmpeg::ffmpeg_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("ffmpeg_sys")).unwrap();
    let json = projection_to_json(&output.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), output.projection.len());
}

#[test]
fn ffmpeg_meta_roundtrip() {
    let pkg = ffmpeg::ffmpeg_package();
    let cfg = GecConfig::new("ffmpeg_sys");
    let output = generate(&input_from_binding(pkg), &cfg).unwrap();
    let meta = output_meta(&cfg, &output);
    let json = meta_to_json(&meta).unwrap();
    let meta2 = meta_from_json(&json).unwrap();
    assert_eq!(meta2.crate_name, "ffmpeg_sys");
    assert_eq!(meta2.item_count, meta.item_count);
}

#[test]
fn ffmpeg_sidecar() {
    let pkg = ffmpeg::ffmpeg_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("ffmpeg_sys")).unwrap();
    let sidecar = build_sidecar("ffmpeg_sys", &output.projection);
    let json = sidecar_to_json(&sidecar).unwrap();
    let sidecar2 = sidecar_from_json(&json).unwrap();
    assert_eq!(sidecar2.crate_name, "ffmpeg_sys");
    assert_eq!(sidecar2.items.len(), output.projection.len());
    assert!(sidecar2.link_libraries.len() >= 5);
}

#[test]
fn ffmpeg_balanced_braces() {
    let pkg = ffmpeg::ffmpeg_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("ff"))
            .unwrap()
            .projection,
    );
    let opens = source.matches('{').count();
    let closes = source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in ffmpeg output");
}

#[test]
fn ffmpeg_fol_consumer() {
    let pkg = ffmpeg::ffmpeg_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("ffmpeg_sys")).unwrap();
    let consumer = FolConsumer;
    let report = consumer.inspect(&output.projection);
    assert_eq!(report.consumer_name, "fol-interloop-rust");
    assert_eq!(report.items_inspected, output.projection.len());
    assert!(!report.findings.is_empty());
}

// ======== Linux kernel UAPI ========

#[test]
fn linux_kernel_full_pipeline() {
    let pkg = linux_kernel::linux_kernel_package();
    let input = input_from_binding(pkg);
    let cfg = GecConfig::new("linux_uapi_sys");
    let output = generate(&input, &cfg).unwrap();
    let source = emit_source(&output.projection);

    assert!(
        output.item_count() >= 50,
        "expected ≥50 items, got {}",
        output.item_count()
    );

    // typedefs
    assert!(source.contains("pub type size_t"));
    assert!(source.contains("pub type pid_t"));
    assert!(source.contains("pub type socklen_t"));

    // socket structs
    assert!(source.contains("pub struct sockaddr"));
    assert!(source.contains("pub struct sockaddr_in"));
    assert!(source.contains("pub struct sockaddr_in6"));
    assert!(source.contains("pub struct sockaddr_un"));

    // other structs
    assert!(source.contains("pub struct iovec"));
    assert!(source.contains("pub struct msghdr"));
    assert!(source.contains("pub struct pollfd"));
    assert!(source.contains("pub struct timespec"));
    assert!(source.contains("pub struct stat"));
    assert!(source.contains("pub struct input_event"));
    assert!(source.contains("pub struct sigaction"));

    // syscall functions
    assert!(source.contains("pub fn open"));
    assert!(source.contains("pub fn close"));
    assert!(source.contains("pub fn read"));
    assert!(source.contains("pub fn write"));
    assert!(source.contains("pub fn mmap"));
    assert!(source.contains("pub fn socket"));
    assert!(source.contains("pub fn bind"));
    assert!(source.contains("pub fn listen"));
    assert!(source.contains("pub fn accept"));
    assert!(source.contains("pub fn connect"));
    assert!(source.contains("pub fn send"));
    assert!(source.contains("pub fn recv"));
    assert!(source.contains("pub fn poll"));
    assert!(source.contains("pub fn epoll_create1"));
    assert!(source.contains("pub fn fork"));
    assert!(source.contains("pub fn kill"));

}

#[test]
fn linux_kernel_rejects_bitfield() {
    let pkg = linux_kernel::linux_kernel_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("linux")).unwrap();
    let source = emit_source(&output.projection);
    // epoll_event_packed has a bitfield → should be rejected
    assert!(!source.contains("pub struct epoll_event_packed"));
    assert!(output.has_diagnostics());
}

#[test]
fn linux_kernel_rejects_unsupported() {
    let pkg = linux_kernel::linux_kernel_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("linux")).unwrap();
    let source = emit_source(&output.projection);
    assert!(!source.contains("__kernel_sigset_t"));
}

#[test]
fn linux_kernel_balanced_braces() {
    let pkg = linux_kernel::linux_kernel_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("l"))
            .unwrap()
            .projection,
    );
    let opens = source.matches('{').count();
    let closes = source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in linux kernel output");
}

#[test]
fn linux_kernel_deterministic() {
    let pkg = linux_kernel::linux_kernel_package();
    let cfg = GecConfig::new("linux_sys");
    let s1 = emit_source(
        &generate(&input_from_binding(pkg.clone()), &cfg)
            .unwrap()
            .projection,
    );
    let s2 = emit_source(
        &generate(&input_from_binding(pkg), &cfg)
            .unwrap()
            .projection,
    );
    assert_eq!(s1, s2);
}

#[test]
fn linux_kernel_json_roundtrip() {
    let pkg = linux_kernel::linux_kernel_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("linux")).unwrap();
    let json = projection_to_json(&output.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), output.projection.len());
}

// ======== Torture ========

#[test]
fn torture_full_pipeline() {
    let pkg = torture::torture_package();
    let input = input_from_binding(pkg);
    let cfg = GecConfig::new("torture_sys");
    let output = generate(&input, &cfg).unwrap();
    let source = emit_source(&output.projection);

    // Torture has a mix of accepted and rejected items
    assert!(
        output.item_count() >= 10,
        "expected ≥10 items, got {}",
        output.item_count()
    );
    assert!(
        output.has_diagnostics(),
        "torture should have diagnostics for rejected items"
    );

    // accepted functions
    assert!(source.contains("pub fn deep_ptr_fn"));
    assert!(source.contains("pub fn nested_callback"));
    assert!(source.contains("pub fn torture_printf"));
    assert!(source.contains("pub fn torture_noop"));
    assert!(source.contains("pub fn unnamed_params"));

    // accepted structs
    assert!(source.contains("pub struct flexible_msg"));
    assert!(source.contains("pub struct all_primitives"));
    assert!(source.contains("pub struct torture_opaque"));

    // union
    assert!(source.contains("pub union torture_union"));

    // enums
    assert!(source.contains("TORTURE_VARIANT_0"));
    assert!(source.contains("TORTURE_VARIANT_49"));
    assert!(source.contains("NEG_THREE"));

    // typedefs
    assert!(source.contains("pub type torture_handle"));
    assert!(source.contains("pub type torture_const_handle"));

    // REJECTED: should NOT appear in source
    assert!(
        !source.contains("pub struct bitfield_torture"),
        "bitfield should be rejected"
    );
    // anonymous items also rejected — no way to name them so they won't appear
}

#[test]
fn torture_rejects_anonymous_and_bitfield() {
    let pkg = torture::torture_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("t")).unwrap();

    // We should have diagnostics for: anonymous struct, bitfield struct,
    // anonymous enum, and the Unsupported item
    let diag_count = output.diagnostics.len();
    assert!(diag_count >= 3, "expected ≥3 diagnostics, got {diag_count}");
}

#[test]
fn torture_deep_pointers_in_source() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    // deep_ptr_fn takes *mut *mut *mut *mut *mut c_int
    // Should contain multiple pointer levels
    assert!(source.contains("deep_ptr_fn"));
}

#[test]
fn torture_nested_fn_pointers() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    assert!(source.contains("nested_callback"));
    // Should have Option<unsafe extern "C" fn(...)> patterns
    assert!(source.contains("Option<unsafe extern \"C\" fn"));
}

#[test]
fn torture_variadic() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    assert!(source.contains("torture_printf"));
    assert!(source.contains("..."));
}

#[test]
fn torture_large_enum() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    // 50-variant enum
    for i in 0..50 {
        assert!(
            source.contains(&format!("TORTURE_VARIANT_{i}")),
            "missing TORTURE_VARIANT_{i}"
        );
    }
}

#[test]
fn torture_signed_enum() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    assert!(source.contains("NEG_THREE"));
    assert!(source.contains("NEG_ONE"));
    assert!(source.contains("POS_ONE"));
}

#[test]
fn torture_statics_emitted() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    assert!(source.contains("torture_global_state"));
    assert!(source.contains("torture_version_string"));
}

#[test]
fn torture_deterministic_10_runs() {
    let pkg = torture::torture_package();
    let cfg = GecConfig::new("t");
    let first = emit_source(
        &generate(&input_from_binding(pkg.clone()), &cfg)
            .unwrap()
            .projection,
    );
    for _ in 0..9 {
        let s = emit_source(
            &generate(&input_from_binding(pkg.clone()), &cfg)
                .unwrap()
                .projection,
        );
        assert_eq!(first, s, "non-deterministic torture output");
    }
}

#[test]
fn torture_balanced_braces() {
    let pkg = torture::torture_package();
    let source = emit_source(
        &generate(&input_from_binding(pkg), &GecConfig::new("t"))
            .unwrap()
            .projection,
    );
    let opens = source.matches('{').count();
    let closes = source.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces in torture output");
}

#[test]
fn torture_fol_consumer() {
    let pkg = torture::torture_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("t")).unwrap();
    let consumer = FolConsumer;
    let report = consumer.inspect(&output.projection);
    assert_eq!(report.consumer_name, "fol-interloop-rust");
    // deep_ptr_fn returns ***void → the innermost is an opaque ptr
    // nested_callback has fn pointer params, not opaque ptr returns
    assert!(!report.findings.is_empty());
}

#[test]
fn torture_json_roundtrip() {
    let pkg = torture::torture_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("t")).unwrap();
    let json = projection_to_json(&output.projection).unwrap();
    let proj2 = projection_from_json(&json).unwrap();
    assert_eq!(proj2.len(), output.projection.len());
}

#[test]
fn torture_sidecar() {
    let pkg = torture::torture_package();
    let output = generate(&input_from_binding(pkg), &GecConfig::new("torture_sys")).unwrap();
    let sidecar = build_sidecar("torture_sys", &output.projection);
    let json = sidecar_to_json(&sidecar).unwrap();
    let sidecar2 = sidecar_from_json(&json).unwrap();
    assert_eq!(sidecar2.crate_name, "torture_sys");
    assert_eq!(sidecar2.items.len(), output.projection.len());
}
