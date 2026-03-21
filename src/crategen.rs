//! Crate emission — writes a full Cargo-compatible Rust crate to disk.
//!
//! This module takes a `RustProjection` and a `GecConfig` and writes out a
//! complete crate directory: `Cargo.toml`, `src/lib.rs`, and optionally
//! `build.rs`.

use std::path::{Path, PathBuf};

use crate::config::GecConfig;
use crate::emit::emit_source;
use crate::error::{GecError, GecResult};
use crate::ir::RustProjection;
use crate::linkgen::emit_build_rs_filtered;

/// Model for the emitted crate manifest.
#[derive(Debug, Clone)]
pub struct CrateManifest {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub description: String,
}

impl CrateManifest {
    pub fn from_config(cfg: &GecConfig) -> Self {
        Self {
            name: cfg.crate_name.clone(),
            version: cfg.crate_version.clone(),
            edition: "2021".into(),
            description: format!("Generated FFI bindings crate (by gec)"),
        }
    }

    /// Render `Cargo.toml` content.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"{}\"\ndescription = \"{}\"\n",
            self.name, self.version, self.edition, self.description
        ));
        out
    }
}

/// Output mode: emit a full crate directory or just a source bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Full crate with `Cargo.toml`, `src/lib.rs`, optional `build.rs`.
    Crate,
    /// Just `lib.rs` (and optional `build.rs`) without `Cargo.toml`.
    SourceBundle,
}

/// Policy for what to do if the output directory already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    /// Fail if the directory exists and is non-empty.
    Fail,
    /// Remove existing contents first.
    Clean,
    /// Overwrite files in place, leave others.
    Overwrite,
}

/// Validates and normalizes a crate name.
pub fn normalize_crate_name(name: &str) -> GecResult<String> {
    let normalized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if normalized.is_empty() {
        return Err(GecError::InvalidConfig {
            reason: "crate name must not be empty".into(),
        });
    }
    if normalized.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(GecError::InvalidConfig {
            reason: "crate name must not start with a digit".into(),
        });
    }
    Ok(normalized)
}

/// Emit a full crate directory from a projection and config.
pub fn emit_crate(
    proj: &RustProjection,
    cfg: &GecConfig,
    output_dir: &Path,
    mode: OutputMode,
    policy: OverwritePolicy,
) -> GecResult<EmittedCrate> {
    let crate_name = normalize_crate_name(&cfg.crate_name)?;

    // Handle output directory
    if output_dir.exists() {
        match policy {
            OverwritePolicy::Fail => {
                let has_content = std::fs::read_dir(output_dir)
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false);
                if has_content {
                    return Err(GecError::InvalidConfig {
                        reason: format!(
                            "output directory '{}' is non-empty (use Clean or Overwrite policy)",
                            output_dir.display()
                        ),
                    });
                }
            }
            OverwritePolicy::Clean => {
                std::fs::remove_dir_all(output_dir)?;
            }
            OverwritePolicy::Overwrite => {}
        }
    }

    // Create directory structure
    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let mut emitted = EmittedCrate {
        root: output_dir.to_path_buf(),
        files: Vec::new(),
    };

    // Emit Cargo.toml (crate mode only)
    if mode == OutputMode::Crate {
        let manifest = CrateManifest::from_config(&GecConfig {
            crate_name: crate_name.clone(),
            ..cfg.clone()
        });
        let cargo_toml = output_dir.join("Cargo.toml");
        std::fs::write(&cargo_toml, manifest.render())?;
        emitted.files.push(cargo_toml);
    }

    // Emit src/lib.rs
    let lib_rs_content = emit_lib_rs(proj, &crate_name);
    let lib_rs = src_dir.join("lib.rs");
    std::fs::write(&lib_rs, &lib_rs_content)?;
    emitted.files.push(lib_rs);

    // Emit build.rs if link requirements exist and build script enabled
    if cfg.emit_build_script && !proj.link_requirements.is_empty() {
        let build_rs_content = emit_build_rs(proj);
        let build_rs = output_dir.join("build.rs");
        std::fs::write(&build_rs, &build_rs_content)?;
        emitted.files.push(build_rs);
    }

    Ok(emitted)
}

/// Info about an emitted crate on disk.
#[derive(Debug, Clone)]
pub struct EmittedCrate {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Emit `src/lib.rs` content with crate-level docs and provenance marker.
fn emit_lib_rs(proj: &RustProjection, crate_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "//! `{}` — generated FFI bindings crate.\n//!\n//! This crate was generated by `gec`.\n\n",
        crate_name
    ));
    out.push_str(&emit_source(proj));
    out
}

/// Emit `build.rs` content from link requirements.
pub fn emit_build_rs(proj: &RustProjection) -> String {
    emit_build_rs_filtered(&proj.link_requirements, &[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn sample_projection() -> RustProjection {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "foo".into(),
            parameters: vec![],
            return_type: RustType::Void,
            variadic: false,
            doc: None,
        }));
        proj.items.push(RustItem::Record(RustRecord {
            name: "Bar".into(),
            kind: RustRecordKind::Struct,
            fields: vec![RustField {
                name: "x".into(),
                ty: RustType::CInt,
            }],
            is_opaque: false,
            doc: None,
        }));
        proj
    }

    fn sample_config() -> GecConfig {
        GecConfig::new("test_bindings")
    }

    // 8.1: manifest model
    #[test]
    fn manifest_from_config() {
        let cfg = sample_config();
        let m = CrateManifest::from_config(&cfg);
        assert_eq!(m.name, "test_bindings");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.edition, "2021");
    }

    // 8.2: emit Cargo.toml
    #[test]
    fn manifest_render() {
        let m = CrateManifest {
            name: "my_ffi".into(),
            version: "1.0.0".into(),
            edition: "2021".into(),
            description: "FFI bindings".into(),
        };
        let toml = m.render();
        assert!(toml.contains("name = \"my_ffi\""));
        assert!(toml.contains("version = \"1.0.0\""));
        assert!(toml.contains("edition = \"2021\""));
    }

    // 8.3: emit src/lib.rs
    #[test]
    fn emit_crate_creates_lib_rs() {
        let dir = tempdir("emit_crate_lib");
        let proj = sample_projection();
        let cfg = sample_config();
        let result = emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        let lib_rs = dir.join("src/lib.rs");
        assert!(lib_rs.exists());
        let content = std::fs::read_to_string(&lib_rs).unwrap();
        assert!(content.contains("test_bindings"));
        assert!(content.contains("pub fn foo"));
        assert!(result.files.iter().any(|f| f.ends_with("lib.rs")));
    }

    // 8.4: emit module tree (single file for now)
    #[test]
    fn emit_crate_creates_cargo_toml() {
        let dir = tempdir("emit_crate_toml");
        let proj = sample_projection();
        let cfg = sample_config();
        emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        let toml = dir.join("Cargo.toml");
        assert!(toml.exists());
        let content = std::fs::read_to_string(&toml).unwrap();
        assert!(content.contains("name = \"test_bindings\""));
    }

    // 8.5: crate-level docs and provenance
    #[test]
    fn lib_rs_has_provenance_marker() {
        let dir = tempdir("emit_crate_prov");
        let proj = sample_projection();
        let cfg = sample_config();
        emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        let content = std::fs::read_to_string(dir.join("src/lib.rs")).unwrap();
        assert!(content.contains("generated by `gec`"));
    }

    // 8.6: source bundle mode (no Cargo.toml)
    #[test]
    fn emit_source_bundle_no_cargo_toml() {
        let dir = tempdir("emit_bundle");
        let proj = sample_projection();
        let cfg = sample_config();
        emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::SourceBundle,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        assert!(!dir.join("Cargo.toml").exists());
        assert!(dir.join("src/lib.rs").exists());
    }

    // 8.7: crate naming policy
    #[test]
    fn normalize_valid_name() {
        assert_eq!(normalize_crate_name("my_ffi").unwrap(), "my_ffi");
    }

    #[test]
    fn normalize_replaces_hyphens() {
        assert_eq!(normalize_crate_name("my-ffi").unwrap(), "my_ffi");
    }

    #[test]
    fn normalize_rejects_empty() {
        assert!(normalize_crate_name("").is_err());
    }

    #[test]
    fn normalize_rejects_leading_digit() {
        assert!(normalize_crate_name("3abc").is_err());
    }

    // 8.8: overwrite/clean policy
    #[test]
    fn fail_policy_on_nonempty_dir() {
        let dir = tempdir("emit_fail");
        std::fs::write(dir.join("existing.txt"), "x").unwrap();
        let proj = sample_projection();
        let cfg = sample_config();
        let result = emit_crate(&proj, &cfg, &dir, OutputMode::Crate, OverwritePolicy::Fail);
        assert!(result.is_err());
    }

    #[test]
    fn clean_policy_removes_existing() {
        let dir = tempdir("emit_clean");
        std::fs::write(dir.join("old.txt"), "x").unwrap();
        let proj = sample_projection();
        let cfg = sample_config();
        emit_crate(&proj, &cfg, &dir, OutputMode::Crate, OverwritePolicy::Clean).unwrap();
        assert!(!dir.join("old.txt").exists());
        assert!(dir.join("Cargo.toml").exists());
    }

    // 8.9: emitted crate structure integration
    #[test]
    fn emitted_crate_structure() {
        let dir = tempdir("emit_structure");
        let mut proj = sample_projection();
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: None,
        });
        let cfg = sample_config();
        let result = emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        assert!(dir.join("Cargo.toml").exists());
        assert!(dir.join("src/lib.rs").exists());
        assert!(dir.join("build.rs").exists());
        assert!(result.files.len() >= 3);
    }

    #[test]
    fn no_build_rs_without_link_requirements() {
        let dir = tempdir("emit_no_build");
        let proj = sample_projection(); // no link requirements
        let cfg = sample_config();
        emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        assert!(!dir.join("build.rs").exists());
    }

    #[test]
    fn no_build_rs_when_disabled() {
        let dir = tempdir("emit_no_build_disabled");
        let mut proj = sample_projection();
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: None,
        });
        let mut cfg = sample_config();
        cfg.emit_build_script = false;
        emit_crate(
            &proj,
            &cfg,
            &dir,
            OutputMode::Crate,
            OverwritePolicy::Overwrite,
        )
        .unwrap();
        assert!(!dir.join("build.rs").exists());
    }

    // 8.10: build.rs content
    #[test]
    fn build_rs_content() {
        let mut proj = RustProjection::new();
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: Some("/usr/lib".into()),
        });
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::StaticLibrary,
            name: "mylib".into(),
            search_path: None,
        });
        let content = emit_build_rs(&proj);
        assert!(content.contains("cargo:rustc-link-search=native=/usr/lib"));
        assert!(content.contains("cargo:rustc-link-lib=dylib=z"));
        assert!(content.contains("cargo:rustc-link-lib=static=mylib"));
    }

    #[test]
    fn build_rs_matches_linkgen_output() {
        let mut proj = RustProjection::new();
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::Framework,
            name: "CoreFoundation".into(),
            search_path: Some("/System/Library/Frameworks".into()),
        });

        assert_eq!(
            emit_build_rs(&proj),
            crate::linkgen::emit_build_rs_filtered(&proj.link_requirements, &[])
        );
    }

    fn tempdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gec_test_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
