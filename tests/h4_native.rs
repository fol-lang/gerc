mod support;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use gerc::{generate, GenerationRequest, ItemSelection};

static NEXT_RUN: AtomicU64 = AtomicU64::new(0);

#[test]
fn generated_no_std_types_cross_an_explicit_gcc_abi_boundary() {
    let gcc = command("gcc");
    let target = checked_output(
        Command::new(&gcc).arg("-dumpmachine"),
        "query explicit GCC target",
    );
    assert_eq!(target.trim(), "x86_64-unknown-linux-gnu");

    let (source, evidence) = support::preservation_pair();
    let packet = support::declaration_id(source.source(), "parc_packet");
    let mode = support::declaration_id(source.source(), "parc_mode");
    let selection = ItemSelection::try_new([packet, mode]).expect("ABI type roots");
    let bundle = generate(
        GenerationRequest::try_new(&source, &evidence, &selection).expect("typed H4 request"),
    )
    .expect("strict H4 generation");
    let generated = bundle
        .files()
        .get("src/lib.rs")
        .and_then(|file| file.utf8_contents())
        .expect("generated Rust translation unit");

    let directory = unique_temp("gerc-h4-native");
    fs::create_dir(&directory).expect("create unique native fixture directory");
    let generated_source = directory.join("bindings.rs");
    let generated_rlib = directory.join("libh4_bindings.rlib");
    fs::write(&generated_source, generated).expect("write generated source");
    checked(
        Command::new(command("rustc"))
            .arg("--crate-name=h4_bindings")
            .arg("--crate-type=rlib")
            .arg("--edition=2021")
            .arg("-o")
            .arg(&generated_rlib)
            .arg(&generated_source),
        "build generated no_std crate",
    );

    let c_object = directory.join("h4_abi.o");
    checked(
        Command::new(&gcc)
            .arg("-std=c17")
            .arg("-m64")
            .arg("-Wall")
            .arg("-Wextra")
            .arg("-Werror")
            .arg("-c")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/native-fixtures/h4_abi.c"))
            .arg("-o")
            .arg(&c_object),
        "compile explicit GCC ABI provider",
    );

    let consumer_source = directory.join("consumer.rs");
    fs::write(
        &consumer_source,
        r#"
extern crate h4_bindings as bindings;

unsafe extern "C" {
    fn h4_packet_roundtrip(packet: bindings::parc_packet) -> bindings::parc_packet;
    fn h4_mode_roundtrip(mode: bindings::parc_mode) -> bindings::parc_mode;
}

fn main() {
    let input = bindings::parc_packet { value: 5 };
    let packet = unsafe { h4_packet_roundtrip(input) };
    assert_eq!(packet.value, 42);
    let mode = unsafe { h4_mode_roundtrip(bindings::PARC_MODE_FAST) };
    assert_eq!(mode, 7_u32 ^ 0x5a5a_a5a5);
}
"#,
    )
    .expect("write ABI consumer");
    let executable = directory.join("consumer");
    checked(
        Command::new(command("rustc"))
            .arg("--crate-name=h4_native_consumer")
            .arg("--edition=2021")
            .arg("--extern")
            .arg(format!("h4_bindings={}", generated_rlib.display()))
            .arg("-C")
            .arg(native_link_arg(&c_object))
            .arg("-o")
            .arg(&executable)
            .arg(&consumer_source),
        "link Rust consumer to exact GCC object",
    );
    checked(
        &mut Command::new(&executable),
        "run cross-language ABI consumer",
    );
    fs::remove_dir_all(&directory).expect("remove owned native fixture directory");
}

fn native_link_arg(path: &Path) -> std::ffi::OsString {
    let mut argument = std::ffi::OsString::from("link-arg=");
    argument.push(path.as_os_str());
    argument
}

fn unique_temp(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        NEXT_RUN.fetch_add(1, Ordering::Relaxed)
    ))
}

fn command(name: &str) -> PathBuf {
    std::env::var_os(match name {
        "gcc" => "GERC_H4_GCC",
        "rustc" => "RUSTC",
        _ => unreachable!("fixed native tools"),
    })
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(name))
}

fn checked(command: &mut Command, action: &str) {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{action}: {error}"));
    assert!(
        output.status.success(),
        "{action} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn checked_output(command: &mut Command, action: &str) -> String {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{action}: {error}"));
    assert!(
        output.status.success(),
        "{action} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("tool output is UTF-8")
}
