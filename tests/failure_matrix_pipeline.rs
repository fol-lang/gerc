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
fn failure_matrix_pipeline_known_anonymous_type_gap_stays_pinned() {
    for (name, crate_name) in [("libpng", "png_sys"), ("zlib", "zlib_sys")] {
        let root = vendored_root(name);
        let include_dir = root.join("include");
        let entry = root.join("main.c");
        let Some(source) = parse_vendored_source(&entry, &[include_dir]) else {
            continue;
        };

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
        let stderr = String::from_utf8_lossy(&checked.stderr);
        assert!(!checked.status.success());
        assert!(stderr.contains("pub type max_align_t = <anonymous>;"));
        assert!(stderr.contains("expected `::`, found `;`"));
    }
}
