#![cfg(feature = "system-tests")]

mod common;

#[path = "../../linc/tests/common/mod.rs"]
mod linc_common;

use std::path::{Path, PathBuf};

use gerc::{emit_crate, generate_from_source, GercConfig, OutputMode, OverwritePolicy};

fn vendored_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../linc/tests/full_apps/external")
        .join(name)
        .join("header")
}

fn parse_vendored_source(
    entry: &Path,
    include_dirs: &[PathBuf],
) -> Result<gerc::SourcePackage, String> {
    let mut cpp_options = vec!["-E".to_string()];
    for dir in include_dirs {
        cpp_options.push(format!("-I{}", dir.display()));
    }

    let cpp_command = std::env::var("CC").unwrap_or_else(|_| "gcc".into());
    let compiler_name = Path::new(&cpp_command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&cpp_command);
    let flavor = if compiler_name.contains("clang") {
        parc::driver::Flavor::ClangC11
    } else {
        parc::driver::Flavor::GnuC11
    };
    let config = parc::driver::Config {
        cpp_command,
        cpp_options,
        flavor,
    };

    let parsed = parc::driver::parse(&config, entry).map_err(|error| {
        format!(
            "failed to parse vendored fixture '{}': {error:?}",
            entry.display()
        )
    })?;
    let extracted = parc::extract::extract_from_translation_unit(&parsed.unit, None);
    let binding = linc_common::from_parc_package(&extracted);
    Ok(common::from_binding_package(&binding))
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gerc_failure_matrix_{name}"));
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

#[test]
fn failure_matrix_pipeline_anonymous_alias_gap_is_closed() {
    for (name, crate_name) in [("libpng", "png_sys"), ("zlib", "zlib_sys")] {
        let root = vendored_root(name);
        let include_dir = root.join("include");
        let entry = root.join("main.c");
        let source = parse_vendored_source(&entry, &[include_dir])
            .unwrap_or_else(|error| panic!("{name} fixture must parse: {error}"));

        let cfg = GercConfig::new(crate_name);
        let output = generate_from_source(source, &cfg).unwrap();
        let dir = tempdir(name);
        let emitted = emit_crate(
            &output.projection,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();

        let checked = cargo_check(&emitted.root);
        let lib_rs = std::fs::read_to_string(emitted.root.join("src/lib.rs")).unwrap();
        assert!(checked.status.success());
        assert!(!lib_rs.contains("pub type max_align_t = <anonymous>;"));
    }
}
