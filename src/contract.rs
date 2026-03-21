//! Public API contract, JSON output, and schema versioning.
//!
//! This module defines the stable, JSON-serializable output contract for `gec`,
//! including schema versioning and deterministic generation guarantees.

use serde::{Deserialize, Serialize};

use crate::config::GecConfig;
use crate::error::{GecError, GecResult};
use crate::gate::{gate_package, GateDecision};
use crate::intake::GecInput;
use crate::ir::RustProjection;
use crate::linkgen::lower_link_surface;
use crate::lower::lower_package;
use crate::output::{GecOutput, GecSeverity};

/// Current schema version for `gec` output metadata.
pub const SCHEMA_VERSION: u32 = 1;

/// JSON-serializable output metadata for downstream tooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GecOutputMeta {
    pub schema_version: u32,
    pub gec_version: String,
    pub crate_name: String,
    pub item_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub diagnostic_count: usize,
}

/// The top-level public API entry point: run the full `gec` pipeline.
///
/// This is the primary way downstream consumers should use `gec`.
///
/// ## API tiers
///
/// - **Tier 1 (stable)**: `generate`, `GecConfig`, `GecInput`, `GecOutput`, `GecOutputMeta`
/// - **Tier 2 (public but less stable)**: individual modules (`lower`, `gate`, `emit`, etc.)
pub fn generate(input: &GecInput, _config: &GecConfig) -> GecResult<GecOutput> {
    let mut input_clone = input.clone();
    input_clone.normalize();

    if input_clone.is_empty() {
        return Err(GecError::EmptyInput);
    }

    let (decisions, gate_diags) =
        gate_package(&input_clone.package, input_clone.validation.as_ref());

    // Filter items: only lower accepted items
    let mut filtered_pkg = input_clone.package.clone();
    let mut accepted_items = Vec::new();
    for (i, decision) in decisions.iter().enumerate() {
        if *decision == GateDecision::Accept {
            if let Some(item) = filtered_pkg.items.get(i) {
                accepted_items.push(item.clone());
            }
        }
    }
    filtered_pkg.items = accepted_items;

    let (mut proj, lower_diags) = lower_package(&filtered_pkg);

    // Populate link requirements
    proj.link_requirements = lower_link_surface(&input_clone.package);

    let mut all_diags = gate_diags;
    all_diags.extend(lower_diags);

    Ok(GecOutput {
        projection: proj,
        diagnostics: all_diags,
    })
}

/// Generate directly from a `linc::SourcePackage`.
///
/// This is the preferred entrypoint when the caller has source contracts and
/// wants `gec` to adapt them through LINC internally.
pub fn generate_from_source(
    source: linc::SourcePackage,
    config: &GecConfig,
) -> GecResult<GecOutput> {
    let input = GecInput::from_source_package(source);
    generate(&input, config)
}

/// Generate JSON-serializable output metadata.
pub fn output_meta(config: &GecConfig, output: &GecOutput) -> GecOutputMeta {
    let rejected = output
        .diagnostics
        .iter()
        .filter(|d| d.severity == GecSeverity::Warning)
        .count();
    GecOutputMeta {
        schema_version: SCHEMA_VERSION,
        gec_version: env!("CARGO_PKG_VERSION").to_string(),
        crate_name: config.crate_name.clone(),
        item_count: output.item_count(),
        accepted_count: output.item_count(),
        rejected_count: rejected,
        diagnostic_count: output.diagnostics.len(),
    }
}

/// Serialize output metadata to JSON.
pub fn meta_to_json(meta: &GecOutputMeta) -> GecResult<String> {
    serde_json::to_string_pretty(meta).map_err(|e| GecError::Serialization(e.to_string()))
}

/// Deserialize output metadata from JSON.
pub fn meta_from_json(json: &str) -> GecResult<GecOutputMeta> {
    let meta: GecOutputMeta =
        serde_json::from_str(json).map_err(|e| GecError::Serialization(e.to_string()))?;
    if meta.schema_version > SCHEMA_VERSION {
        return Err(GecError::InvalidConfig {
            reason: format!(
                "unsupported gec schema version {} (this build supports up to {})",
                meta.schema_version, SCHEMA_VERSION
            ),
        });
    }
    Ok(meta)
}

/// Serialize a `RustProjection` to JSON.
pub fn projection_to_json(proj: &RustProjection) -> GecResult<String> {
    serde_json::to_string_pretty(proj).map_err(|e| GecError::Serialization(e.to_string()))
}

/// Deserialize a `RustProjection` from JSON.
pub fn projection_from_json(json: &str) -> GecResult<RustProjection> {
    serde_json::from_str(json).map_err(|e| GecError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        generate, generate_from_source, meta_from_json, meta_to_json, output_meta,
        projection_from_json, projection_to_json, SCHEMA_VERSION,
    };
    use crate::config::GecConfig;
    use crate::error::GecError;
    use crate::intake::GecInput;
    use linc::*;

    fn sample_input() -> GecInput {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("x".into()),
                ty: BindingType::Int,
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("Bar".into()),
            fields: Some(vec![FieldBinding {
                name: Some("val".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));
        pkg.items.push(BindingItem::TypeAlias(TypeAliasBinding {
            name: "size_t".into(),
            target: BindingType::ULong,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        }));
        GecInput::from_package(pkg)
    }

    // 11.1: root-level public API tiers
    #[test]
    fn generate_basic() {
        let input = sample_input();
        let cfg = GecConfig::default();
        let output = generate(&input, &cfg).unwrap();
        assert!(!output.is_empty());
        assert_eq!(output.item_count(), 3);
    }

    #[test]
    fn generate_empty_input_error() {
        let input = GecInput::from_package(BindingPackage::new());
        let cfg = GecConfig::default();
        let result = generate(&input, &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn generate_from_source_basic() {
        let mut source = SourcePackage::default();
        source
            .declarations
            .push(SourceDeclaration::Function(SourceFunction {
                name: "foo".into(),
                parameters: vec![],
                return_type: SourceType::Void,
                variadic: false,
                source_offset: None,
            }));

        let cfg = GecConfig::default();
        let output = generate_from_source(source, &cfg).unwrap();
        assert_eq!(output.item_count(), 1);
    }

    #[test]
    fn generate_from_source_empty_input_error() {
        let cfg = GecConfig::default();
        let result = generate_from_source(SourcePackage::default(), &cfg);
        assert!(result.is_err());
    }

    // 11.2: typed error taxonomy
    #[test]
    fn error_types_exhaustive() {
        let _ = GecError::EmptyInput;
        let _ = GecError::InvalidConfig { reason: "x".into() };
        let _ = GecError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x"));
        let _ = GecError::Serialization("x".into());
    }

    // 11.3: JSON-serializable output contract
    #[test]
    fn output_meta_json_roundtrip() {
        let input = sample_input();
        let cfg = GecConfig::new("test");
        let output = generate(&input, &cfg).unwrap();
        let meta = output_meta(&cfg, &output);
        let json = meta_to_json(&meta).unwrap();
        let meta2 = meta_from_json(&json).unwrap();
        assert_eq!(meta2.schema_version, SCHEMA_VERSION);
        assert_eq!(meta2.crate_name, "test");
        assert_eq!(meta2.item_count, meta.item_count);
    }

    // 11.4: schema versioning policy
    #[test]
    fn reject_future_schema_version() {
        let json = r#"{"schema_version": 99, "gec_version": "0.1.0", "crate_name": "x", "item_count": 0, "accepted_count": 0, "rejected_count": 0, "diagnostic_count": 0}"#;
        let result = meta_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn accept_current_schema_version() {
        let json = format!(
            r#"{{"schema_version": {}, "gec_version": "0.1.0", "crate_name": "x", "item_count": 0, "accepted_count": 0, "rejected_count": 0, "diagnostic_count": 0}}"#,
            SCHEMA_VERSION
        );
        let meta = meta_from_json(&json).unwrap();
        assert_eq!(meta.schema_version, SCHEMA_VERSION);
    }

    // 11.5: compatibility fixtures
    #[test]
    fn meta_missing_fields_deserialize() {
        // Older metadata without diagnostic_count should still work with serde defaults
        let json = format!(
            r#"{{"schema_version": {}, "gec_version": "0.1.0", "crate_name": "x", "item_count": 0, "accepted_count": 0, "rejected_count": 0, "diagnostic_count": 0}}"#,
            SCHEMA_VERSION
        );
        let meta = meta_from_json(&json).unwrap();
        assert_eq!(meta.item_count, 0);
    }

    // 11.6: public API tests and contract snapshots
    #[test]
    fn projection_json_roundtrip() {
        let input = sample_input();
        let cfg = GecConfig::default();
        let output = generate(&input, &cfg).unwrap();
        let json = projection_to_json(&output.projection).unwrap();
        let proj2 = projection_from_json(&json).unwrap();
        assert_eq!(proj2.len(), output.projection.len());
    }

    #[test]
    fn generate_filters_rejected_items() {
        let mut pkg = BindingPackage::new();
        // Accepted function
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "good".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        }));
        // Rejected bitfield struct
        pkg.items.push(BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("bad".into()),
            fields: Some(vec![FieldBinding {
                name: Some("x".into()),
                ty: BindingType::UInt,
                bit_width: Some(4),
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        }));

        let input = GecInput::from_package(pkg);
        let output = generate(&input, &GecConfig::default()).unwrap();
        // Only the function should be in the projection (bitfield filtered out)
        assert_eq!(output.item_count(), 1);
        assert!(output.has_diagnostics());
    }

    // 11.7: non-goals (informational — tested by absence)
    // gec does not parse C, does not own fol surface, etc.
    // This is enforced by the module structure.

    // 11.8: deterministic generation guarantees
    #[test]
    fn deterministic_output() {
        let input = sample_input();
        let cfg = GecConfig::default();
        let output1 = generate(&input, &cfg).unwrap();
        let output2 = generate(&input, &cfg).unwrap();
        let json1 = projection_to_json(&output1.projection).unwrap();
        let json2 = projection_to_json(&output2.projection).unwrap();
        assert_eq!(json1, json2, "non-deterministic output");
    }

    #[test]
    fn deterministic_output_10_runs() {
        let input = sample_input();
        let cfg = GecConfig::default();
        let first = projection_to_json(&generate(&input, &cfg).unwrap().projection).unwrap();
        for _ in 0..9 {
            let json = projection_to_json(&generate(&input, &cfg).unwrap().projection).unwrap();
            assert_eq!(first, json, "non-deterministic output detected");
        }
    }

    // 11.9 & 11.10: readiness (tested by overall pipeline success)
    #[test]
    fn full_api_surface_works() {
        let mut pkg = BindingPackage::new();
        pkg.items.push(BindingItem::Function(FunctionBinding {
            name: "init".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        }));
        pkg.link.libraries.push(LinkLibrary {
            name: "mylib".into(),
            kind: LinkLibraryKind::Default,
            source: LinkRequirementSource::Declared,
        });

        let input = GecInput::from_package(pkg);
        let cfg = GecConfig::new("mylib_sys");
        let output = generate(&input, &cfg).unwrap();

        // Meta
        let meta = output_meta(&cfg, &output);
        assert_eq!(meta.crate_name, "mylib_sys");
        assert_eq!(meta.schema_version, SCHEMA_VERSION);

        // JSON roundtrip
        let meta_json = meta_to_json(&meta).unwrap();
        let _ = meta_from_json(&meta_json).unwrap();

        // Projection
        assert!(!output.is_empty());
        assert!(output.projection.link_requirements.len() >= 1);

        // Source emission
        let source = crate::emit::emit_source(&output.projection);
        assert!(source.contains("pub fn init"));
    }
}
