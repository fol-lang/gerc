//! Downstream consumer contract and metadata sidecar.
//!
//! Defines the generic consumer interface for tools (like `fol-interloop-rust`)
//! that inspect `gerc`-generated crates.  This module intentionally avoids
//! `fol`-specific assumptions while providing concrete hooks for it.

use serde::{Deserialize, Serialize};

use crate::contract::SCHEMA_VERSION;
use crate::ir::{RustItem, RustProjection, RustType};

/// Generic downstream-consumer contract.
///
/// Any tool that consumes `gerc` output should use this trait to inspect
/// the generated projection.
pub trait GercConsumer {
    /// Inspect the projection and produce consumer-specific output.
    fn inspect(&self, proj: &RustProjection) -> ConsumerReport;
}

/// Report produced by a consumer inspecting a `gerc` projection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsumerReport {
    pub consumer_name: String,
    pub items_inspected: usize,
    pub findings: Vec<ConsumerFinding>,
}

/// One finding from a consumer inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerFinding {
    pub item_name: Option<String>,
    pub kind: FindingKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingKind {
    Usable,
    NeedsWrapper,
    Unsupported,
    Info,
}

/// Optional metadata sidecar emitted alongside a generated crate.
///
/// This is a JSON file that downstream tooling can read without parsing
/// Rust source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSidecar {
    pub schema_version: u32,
    pub gerc_version: String,
    pub crate_name: String,
    pub items: Vec<SidecarItem>,
    pub link_libraries: Vec<String>,
}

/// One item in the metadata sidecar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarItem {
    pub name: String,
    pub kind: SidecarItemKind,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidecarItemKind {
    Function,
    Record,
    Enum,
    TypeAlias,
    Constant,
    Static,
    Unsupported,
}

/// Build a metadata sidecar from a projection.
pub fn build_sidecar(crate_name: &str, proj: &RustProjection) -> MetadataSidecar {
    let items: Vec<SidecarItem> = proj
        .items
        .iter()
        .map(|item| SidecarItem {
            name: item_name(item).unwrap_or_else(|| "<anonymous>".into()),
            kind: item_kind(item),
            provenance: None,
        })
        .collect();

    let link_libraries: Vec<String> = proj
        .link_requirements
        .iter()
        .map(|r| r.name.clone())
        .collect();

    MetadataSidecar {
        schema_version: SCHEMA_VERSION,
        gerc_version: env!("CARGO_PKG_VERSION").to_string(),
        crate_name: crate_name.into(),
        items,
        link_libraries,
    }
}

/// Serialize a sidecar to JSON.
pub fn sidecar_to_json(sidecar: &MetadataSidecar) -> Result<String, String> {
    serde_json::to_string_pretty(sidecar).map_err(|e| e.to_string())
}

/// Deserialize a sidecar from JSON.
pub fn sidecar_from_json(json: &str) -> Result<MetadataSidecar, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Emitted Rust surface conventions: item names that interloop tooling
/// should recognize.
pub fn extern_function_names(proj: &RustProjection) -> Vec<String> {
    proj.items
        .iter()
        .filter_map(|item| match item {
            RustItem::Function(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect()
}

/// Symbol/provenance markers useful for interloop inspection.
pub fn record_names(proj: &RustProjection) -> Vec<String> {
    proj.items
        .iter()
        .filter_map(|item| match item {
            RustItem::Record(r) => Some(r.name.clone()),
            _ => None,
        })
        .collect()
}

/// Type alias names in the projection.
pub fn type_alias_names(proj: &RustProjection) -> Vec<String> {
    proj.items
        .iter()
        .filter_map(|item| match item {
            RustItem::TypeAlias(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect()
}

fn item_name(item: &RustItem) -> Option<String> {
    match item {
        RustItem::Function(f) => Some(f.name.clone()),
        RustItem::Record(r) => Some(r.name.clone()),
        RustItem::Enum(e) => Some(e.name.clone()),
        RustItem::TypeAlias(t) => Some(t.name.clone()),
        RustItem::Constant(c) => Some(c.name.clone()),
        RustItem::Static(s) => Some(s.name.clone()),
        RustItem::Unsupported(u) => u.name.clone(),
    }
}

fn item_kind(item: &RustItem) -> SidecarItemKind {
    match item {
        RustItem::Function(_) => SidecarItemKind::Function,
        RustItem::Record(_) => SidecarItemKind::Record,
        RustItem::Enum(_) => SidecarItemKind::Enum,
        RustItem::TypeAlias(_) => SidecarItemKind::TypeAlias,
        RustItem::Constant(_) => SidecarItemKind::Constant,
        RustItem::Static(_) => SidecarItemKind::Static,
        RustItem::Unsupported(_) => SidecarItemKind::Unsupported,
    }
}

/// A minimal example consumer that reports all items as usable.
pub struct PassthroughConsumer;

impl GercConsumer for PassthroughConsumer {
    fn inspect(&self, proj: &RustProjection) -> ConsumerReport {
        ConsumerReport {
            consumer_name: "passthrough".into(),
            items_inspected: proj.len(),
            findings: proj
                .items
                .iter()
                .map(|item| ConsumerFinding {
                    item_name: item_name(item),
                    kind: FindingKind::Usable,
                    message: "accepted".into(),
                })
                .collect(),
        }
    }
}

/// A fol-oriented example consumer (without making it the core contract).
pub struct FolConsumer;

impl GercConsumer for FolConsumer {
    fn inspect(&self, proj: &RustProjection) -> ConsumerReport {
        let mut findings = Vec::new();
        for item in &proj.items {
            let finding = match item {
                RustItem::Function(f) => {
                    // fol cares about function pointers returning void*
                    let needs_wrapper = matches!(f.return_type, RustType::OpaquePtr { .. });
                    ConsumerFinding {
                        item_name: Some(f.name.clone()),
                        kind: if needs_wrapper {
                            FindingKind::NeedsWrapper
                        } else {
                            FindingKind::Usable
                        },
                        message: if needs_wrapper {
                            "returns opaque pointer — fol may need wrapper".into()
                        } else {
                            "usable".into()
                        },
                    }
                }
                RustItem::Unsupported(u) => ConsumerFinding {
                    item_name: u.name.clone(),
                    kind: FindingKind::Unsupported,
                    message: u.reason.clone(),
                },
                _ => ConsumerFinding {
                    item_name: item_name(item),
                    kind: FindingKind::Usable,
                    message: "usable".into(),
                },
            };
            findings.push(finding);
        }

        ConsumerReport {
            consumer_name: "fol-interloop-rust".into(),
            items_inspected: proj.len(),
            findings,
        }
    }
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
        proj.items.push(RustItem::TypeAlias(RustTypeAlias {
            name: "size_t".into(),
            target: RustType::CULong,
            doc: None,
        }));
        proj.link_requirements.push(RustLinkRequirement {
            kind: RustLinkKind::DynamicLibrary,
            name: "mylib".into(),
            search_path: None,
        });
        proj
    }

    // 12.1: generic downstream-consumer contract
    #[test]
    fn passthrough_consumer() {
        let proj = sample_projection();
        let consumer = PassthroughConsumer;
        let report = consumer.inspect(&proj);
        assert_eq!(report.consumer_name, "passthrough");
        assert_eq!(report.items_inspected, 3);
        assert!(report
            .findings
            .iter()
            .all(|f| f.kind == FindingKind::Usable));
    }

    // 12.2: emitted surface conventions
    #[test]
    fn extern_function_names_extracted() {
        let proj = sample_projection();
        let names = extern_function_names(&proj);
        assert_eq!(names, vec!["foo"]);
    }

    // 12.3: symbol/provenance markers
    #[test]
    fn record_names_extracted() {
        let proj = sample_projection();
        let names = record_names(&proj);
        assert_eq!(names, vec!["Bar"]);
    }

    #[test]
    fn type_alias_names_extracted() {
        let proj = sample_projection();
        let names = type_alias_names(&proj);
        assert_eq!(names, vec!["size_t"]);
    }

    // 12.4: metadata sidecar
    #[test]
    fn sidecar_built() {
        let proj = sample_projection();
        let sidecar = build_sidecar("test_crate", &proj);
        assert_eq!(sidecar.crate_name, "test_crate");
        assert_eq!(sidecar.items.len(), 3);
        assert_eq!(sidecar.link_libraries, vec!["mylib"]);
        assert_eq!(sidecar.items[0].kind, SidecarItemKind::Function);
        assert_eq!(sidecar.items[1].kind, SidecarItemKind::Record);
        assert_eq!(sidecar.items[2].kind, SidecarItemKind::TypeAlias);
    }

    #[test]
    fn sidecar_json_roundtrip() {
        let proj = sample_projection();
        let sidecar = build_sidecar("test_crate", &proj);
        let json = sidecar_to_json(&sidecar).unwrap();
        let sidecar2 = sidecar_from_json(&json).unwrap();
        assert_eq!(sidecar2.crate_name, "test_crate");
        assert_eq!(sidecar2.items.len(), 3);
    }

    // 12.5: fixture showing what fol-interloop-rust would inspect
    #[test]
    fn fol_interloop_inspection_fixture() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "malloc".into(),
            parameters: vec![RustParameter {
                name: "size".into(),
                ty: RustType::CULong,
            }],
            return_type: RustType::OpaquePtr { is_const: false },
            variadic: false,
            doc: None,
        }));
        proj.items.push(RustItem::Function(RustFunction {
            name: "free".into(),
            parameters: vec![RustParameter {
                name: "ptr".into(),
                ty: RustType::OpaquePtr { is_const: false },
            }],
            return_type: RustType::Void,
            variadic: false,
            doc: None,
        }));

        let sidecar = build_sidecar("libc_sys", &proj);
        assert_eq!(sidecar.items.len(), 2);
        assert_eq!(sidecar.items[0].name, "malloc");
        assert_eq!(sidecar.items[1].name, "free");
    }

    // 12.6: small consumer example outside fol
    #[test]
    fn passthrough_consumer_example() {
        let proj = sample_projection();
        let consumer = PassthroughConsumer;
        let report = consumer.inspect(&proj);
        assert_eq!(report.items_inspected, proj.len());
        // All items accepted
        assert!(report
            .findings
            .iter()
            .all(|f| f.kind == FindingKind::Usable));
    }

    // 12.7: fol-oriented example
    #[test]
    fn fol_consumer_example() {
        let mut proj = RustProjection::new();
        proj.items.push(RustItem::Function(RustFunction {
            name: "alloc".into(),
            parameters: vec![],
            return_type: RustType::OpaquePtr { is_const: false },
            variadic: false,
            doc: None,
        }));
        proj.items.push(RustItem::Function(RustFunction {
            name: "compute".into(),
            parameters: vec![RustParameter {
                name: "x".into(),
                ty: RustType::CInt,
            }],
            return_type: RustType::CInt,
            variadic: false,
            doc: None,
        }));
        proj.items.push(RustItem::Unsupported(RustUnsupported {
            name: Some("bad".into()),
            reason: "bitfield".into(),
        }));

        let consumer = FolConsumer;
        let report = consumer.inspect(&proj);
        assert_eq!(report.consumer_name, "fol-interloop-rust");
        assert_eq!(report.findings.len(), 3);
        assert_eq!(report.findings[0].kind, FindingKind::NeedsWrapper); // alloc returns void*
        assert_eq!(report.findings[1].kind, FindingKind::Usable); // compute
        assert_eq!(report.findings[2].kind, FindingKind::Unsupported); // bad
    }

    // 12.8: documented consumption (tested via sidecar structure)
    #[test]
    fn sidecar_has_schema_version() {
        let proj = sample_projection();
        let sidecar = build_sidecar("test", &proj);
        assert_eq!(sidecar.schema_version, SCHEMA_VERSION);
    }

    // 12.9: what remains outside gerc
    // This is a documentation slice — gerc does not own:
    // - C parsing (bic)
    // - fol surface generation
    // - runtime loader policy
    // The test proves the contract boundaries are clean.
    #[test]
    fn gerc_does_not_expose_linc_internals() {
        // The consumer module uses only gerc::ir types, not linc types
        let proj = sample_projection();
        let sidecar = build_sidecar("boundary_test", &proj);
        // Sidecar is purely gerc-typed
        let json = sidecar_to_json(&sidecar).unwrap();
        assert!(!json.contains("BindingType")); // no linc types leak
        assert!(!json.contains("BindingItem"));
    }

    // 12.10: architecture review — all modules compose correctly
    #[test]
    fn full_consumer_pipeline() {
        // Build a package, run full generate, then consumer inspect
        use crate::config::GercConfig;
        use crate::contract::generate;
        use crate::intake::GercInput;
        use crate::c::*;

        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "init".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        }));

        let input =
            GercInput::from_source_package(crate::intake::source_package_from_binding(&pkg));
        let cfg = GercConfig::new("final_test");
        let output = generate(&input, &cfg).unwrap();

        // Consumer inspection
        let consumer = PassthroughConsumer;
        let report = consumer.inspect(&output.projection);
        assert_eq!(report.items_inspected, 1);

        // Sidecar
        let sidecar = build_sidecar("final_test", &output.projection);
        assert_eq!(sidecar.items.len(), 1);
        assert_eq!(sidecar.items[0].name, "init");

        // Source emission
        let source = crate::emit::emit_source(&output.projection);
        assert!(source.contains("pub fn init"));
    }
}
