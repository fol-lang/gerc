//! Native link metadata lowering and `build.rs` emission.
//!
//! Turns `linc` link surfaces and resolved link plans into Rust-side
//! `RustLinkRequirement` values and emitted `build.rs` content.

use linc::{
    BindingLinkSurface, BindingPackage, LinkArtifact, LinkArtifactKind, LinkInput, LinkLibrary,
    LinkLibraryKind, LinkResolutionMode, ResolvedLinkPlan,
};

use crate::ir::{RustLinkKind, RustLinkRequirement};

/// Lower a `BindingPackage`'s link surface into `RustLinkRequirement` values.
pub fn lower_link_surface(pkg: &BindingPackage) -> Vec<RustLinkRequirement> {
    lower_declared_link_surface(&pkg.link)
}

/// Lower a declared link surface directly into `RustLinkRequirement` values.
pub fn lower_declared_link_surface(link: &BindingLinkSurface) -> Vec<RustLinkRequirement> {
    lower_binding_link_surface(link)
}

/// Lower from a resolved link plan (when available).
pub fn lower_resolved_plan(plan: &ResolvedLinkPlan) -> Vec<RustLinkRequirement> {
    let mut reqs = Vec::new();

    for input in &plan.inputs {
        if let Some(req) = lower_link_input(input, plan.preferred_mode) {
            reqs.push(req);
        }
    }

    // Add search paths from requirements that have providers
    for resolved in &plan.requirements {
        if let Some(req) = lower_link_input(&resolved.declared, plan.preferred_mode) {
            // Only add if not already present
            if !reqs
                .iter()
                .any(|r| r.name == req.name && r.kind == req.kind)
            {
                reqs.push(req);
            }
        }
    }

    reqs
}

fn lower_binding_link_surface(link: &BindingLinkSurface) -> Vec<RustLinkRequirement> {
    let mut reqs = Vec::new();

    // 9.3: library search paths
    let search_paths: Vec<&str> = link.library_paths.iter().map(|s| s.as_str()).collect();

    // 9.4: named libraries
    for lib in &link.libraries {
        reqs.push(lower_library(lib, &search_paths, link.preferred_mode));
    }

    // 9.6 & 9.7: concrete artifacts
    for artifact in &link.artifacts {
        reqs.push(lower_artifact(artifact));
    }

    // Ordered inputs (may overlap with above, dedup by name+kind)
    for input in &link.ordered_inputs {
        if let Some(req) = lower_link_input(input, link.preferred_mode) {
            if !reqs
                .iter()
                .any(|r| r.name == req.name && r.kind == req.kind)
            {
                reqs.push(req);
            }
        }
    }

    // Frameworks
    for fw in &link.frameworks {
        let req = RustLinkRequirement {
            kind: RustLinkKind::Framework,
            name: fw.name.clone(),
            search_path: None,
        };
        if !reqs
            .iter()
            .any(|r| r.name == req.name && r.kind == req.kind)
        {
            reqs.push(req);
        }
    }

    reqs
}

/// 9.4: Lower a named library.
fn lower_library(
    lib: &LinkLibrary,
    search_paths: &[&str],
    preferred_mode: LinkResolutionMode,
) -> RustLinkRequirement {
    let kind = match lib.kind {
        LinkLibraryKind::Static => RustLinkKind::StaticLibrary,
        LinkLibraryKind::Dynamic => RustLinkKind::DynamicLibrary,
        LinkLibraryKind::Default => match preferred_mode {
            LinkResolutionMode::PreferStatic => RustLinkKind::StaticLibrary,
            LinkResolutionMode::PreferDynamic => RustLinkKind::DynamicLibrary,
            LinkResolutionMode::Default => RustLinkKind::DynamicLibrary,
        },
    };

    RustLinkRequirement {
        kind,
        name: lib.name.clone(),
        search_path: search_paths.first().map(|s| s.to_string()),
    }
}

/// 9.6 & 9.7: Lower concrete artifacts.
fn lower_artifact(artifact: &LinkArtifact) -> RustLinkRequirement {
    let kind = match artifact.kind {
        LinkArtifactKind::StaticLibrary => RustLinkKind::StaticLibrary,
        LinkArtifactKind::SharedLibrary => RustLinkKind::DynamicLibrary,
        LinkArtifactKind::Object => RustLinkKind::StaticLibrary,
    };

    // Extract directory as search path, name from filename
    let path = std::path::Path::new(&artifact.path);
    let search_path = path.parent().map(|p| p.to_string_lossy().into_owned());
    let name = path
        .file_stem()
        .map(|s| {
            let n = s.to_string_lossy();
            // Strip "lib" prefix if present
            n.strip_prefix("lib").unwrap_or(&n).to_string()
        })
        .unwrap_or_else(|| artifact.path.clone());

    RustLinkRequirement {
        kind,
        name,
        search_path,
    }
}

fn lower_link_input(
    input: &LinkInput,
    preferred_mode: LinkResolutionMode,
) -> Option<RustLinkRequirement> {
    match input {
        LinkInput::Library(lib) => Some(lower_library(lib, &[], preferred_mode)),
        LinkInput::Artifact(artifact) => Some(lower_artifact(artifact)),
        LinkInput::Framework(fw) => Some(RustLinkRequirement {
            kind: RustLinkKind::Framework,
            name: fw.name.clone(),
            search_path: None,
        }),
    }
}

/// Emit `build.rs` content from link requirements, with optional
/// platform filtering.
pub fn emit_build_rs_filtered(
    reqs: &[RustLinkRequirement],
    platform_constraints: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("// Generated by GERC — do not edit.\nfn main() {\n");

    // 9.8: platform filtering
    if !platform_constraints.is_empty() {
        let conditions: Vec<String> = platform_constraints
            .iter()
            .map(|c| format!("cfg!(target_os = \"{}\")", c))
            .collect();
        out.push_str(&format!("    if !({}) {{\n", conditions.join(" || ")));
        out.push_str("        return;\n");
        out.push_str("    }\n");
    }

    for req in reqs {
        if let Some(ref path) = req.search_path {
            out.push_str(&format!(
                "    println!(\"cargo:rustc-link-search=native={}\");\n",
                path
            ));
        }
        match req.kind {
            RustLinkKind::DynamicLibrary => {
                out.push_str(&format!(
                    "    println!(\"cargo:rustc-link-lib=dylib={}\");\n",
                    req.name
                ));
            }
            RustLinkKind::StaticLibrary => {
                out.push_str(&format!(
                    "    println!(\"cargo:rustc-link-lib=static={}\");\n",
                    req.name
                ));
            }
            RustLinkKind::Framework => {
                out.push_str(&format!(
                    "    println!(\"cargo:rustc-link-lib=framework={}\");\n",
                    req.name
                ));
            }
        }
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use linc::*;

    fn empty_link_surface() -> BindingLinkSurface {
        BindingLinkSurface::default()
    }

    // 9.1: build-script emission model
    #[test]
    fn empty_surface_produces_empty_reqs() {
        let pkg = BindingPackage::new();
        let reqs = lower_link_surface(&pkg);
        assert!(reqs.is_empty());
    }

    // 9.2: include paths (included as search paths)
    #[test]
    fn library_paths_become_search_paths() {
        let mut link = empty_link_surface();
        link.library_paths.push("/usr/local/lib".into());
        link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].search_path.as_deref(), Some("/usr/local/lib"));
    }

    // 9.3: library search paths into cargo:rustc-link-search
    #[test]
    fn search_path_in_build_rs() {
        let reqs = vec![RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: Some("/usr/local/lib".into()),
        }];
        let content = emit_build_rs_filtered(&reqs, &[]);
        assert!(content.contains("cargo:rustc-link-search=native=/usr/local/lib"));
    }

    // 9.4: named libraries into cargo:rustc-link-lib
    #[test]
    fn named_library_lowered() {
        let mut link = empty_link_surface();
        link.libraries.push(LinkLibrary {
            name: "ssl".into(),
            kind: LinkLibraryKind::Dynamic,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, "ssl");
        assert_eq!(reqs[0].kind, RustLinkKind::DynamicLibrary);
    }

    // 9.5: static vs dynamic preference
    #[test]
    fn prefer_static_mode() {
        let mut link = empty_link_surface();
        link.preferred_mode = LinkResolutionMode::PreferStatic;
        link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs[0].kind, RustLinkKind::StaticLibrary);
    }

    #[test]
    fn prefer_dynamic_mode() {
        let mut link = empty_link_surface();
        link.preferred_mode = LinkResolutionMode::PreferDynamic;
        link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs[0].kind, RustLinkKind::DynamicLibrary);
    }

    #[test]
    fn explicit_kind_overrides_preference() {
        let mut link = empty_link_surface();
        link.preferred_mode = LinkResolutionMode::PreferDynamic;
        link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Static,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs[0].kind, RustLinkKind::StaticLibrary);
    }

    // 9.6: concrete static artifacts
    #[test]
    fn static_artifact_lowered() {
        let mut link = empty_link_surface();
        link.artifacts.push(LinkArtifact {
            path: "/usr/lib/libz.a".into(),
            kind: LinkArtifactKind::StaticLibrary,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].kind, RustLinkKind::StaticLibrary);
        assert_eq!(reqs[0].name, "z");
        assert_eq!(reqs[0].search_path.as_deref(), Some("/usr/lib"));
    }

    // 9.7: concrete shared artifacts
    #[test]
    fn shared_artifact_lowered() {
        let mut link = empty_link_surface();
        link.artifacts.push(LinkArtifact {
            path: "/usr/lib/libcurl.so".into(),
            kind: LinkArtifactKind::SharedLibrary,
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs[0].kind, RustLinkKind::DynamicLibrary);
        assert_eq!(reqs[0].name, "curl");
    }

    // 9.8: platform filtering
    #[test]
    fn platform_filter_in_build_rs() {
        let reqs = vec![RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: None,
        }];
        let content = emit_build_rs_filtered(&reqs, &["linux".into()]);
        assert!(content.contains("cfg!(target_os = \"linux\")"));
        assert!(content.contains("return;"));
    }

    #[test]
    fn no_platform_filter_when_empty() {
        let reqs = vec![RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "z".into(),
            search_path: None,
        }];
        let content = emit_build_rs_filtered(&reqs, &[]);
        assert!(!content.contains("cfg!"));
    }

    // 9.9: build.rs golden test
    #[test]
    fn build_rs_golden() {
        let reqs = vec![
            RustLinkRequirement {
                kind: RustLinkKind::DynamicLibrary,
                name: "z".into(),
                search_path: Some("/usr/lib".into()),
            },
            RustLinkRequirement {
                kind: RustLinkKind::StaticLibrary,
                name: "mylib".into(),
                search_path: Some("/opt/lib".into()),
            },
            RustLinkRequirement {
                kind: RustLinkKind::Framework,
                name: "Security".into(),
                search_path: None,
            },
        ];
        let content = emit_build_rs_filtered(&reqs, &[]);
        assert!(content.starts_with("// Generated by GERC"));
        assert!(content.contains("fn main()"));
        assert!(content.contains("cargo:rustc-link-search=native=/usr/lib"));
        assert!(content.contains("cargo:rustc-link-lib=dylib=z"));
        assert!(content.contains("cargo:rustc-link-search=native=/opt/lib"));
        assert!(content.contains("cargo:rustc-link-lib=static=mylib"));
        assert!(content.contains("cargo:rustc-link-lib=framework=Security"));
        assert!(content.ends_with("}\n"));
    }

    // 9.10: integration test — full pipeline from package to link directives
    #[test]
    fn full_link_pipeline() {
        let mut pkg = BindingPackage::new();
        pkg.link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        pkg.link.library_paths.push("/usr/lib".into());
        pkg.link.artifacts.push(LinkArtifact {
            path: "/opt/lib/libcustom.a".into(),
            kind: LinkArtifactKind::StaticLibrary,
            source: LinkRequirementSource::Declared,
        });

        let reqs = lower_link_surface(&pkg);
        assert_eq!(reqs.len(), 2); // z + custom
        let content = emit_build_rs_filtered(&reqs, &[]);
        assert!(content.contains("cargo:rustc-link-lib=dylib=z"));
        assert!(content.contains("cargo:rustc-link-lib=static=custom"));
    }

    // Resolved link plan lowering
    #[test]
    fn lower_resolved_plan_basic() {
        let plan = ResolvedLinkPlan {
            inputs: vec![LinkInput::Library(LinkLibrary {
                name: "ssl".into(),
                kind: LinkLibraryKind::Dynamic,
                source: LinkRequirementSource::Declared,
            })],
            ..Default::default()
        };
        let reqs = lower_resolved_plan(&plan);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, "ssl");
    }

    // Framework lowering
    #[test]
    fn framework_lowered() {
        let mut link = empty_link_surface();
        link.frameworks.push(LinkFramework {
            name: "CoreFoundation".into(),
            source: LinkRequirementSource::Declared,
        });
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].kind, RustLinkKind::Framework);
        assert_eq!(reqs[0].name, "CoreFoundation");
    }

    // Dedup ordered inputs
    #[test]
    fn dedup_ordered_inputs() {
        let mut link = empty_link_surface();
        link.libraries.push(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });
        link.ordered_inputs.push(LinkInput::Library(LinkLibrary {
            name: "z".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        }));
        let reqs = lower_binding_link_surface(&link);
        assert_eq!(reqs.len(), 1); // deduped
    }
}
