//! ABI and safety gating for Rust projection.
//!
//! This module defines rules for when `gec` should refuse to generate Rust
//! code because the `linc` evidence is insufficient for safe FFI.

use linc::{
    BindingItem, BindingPackage, EnumBinding, FieldBinding, FunctionBinding, RecordBinding,
    ValidationReport,
};

use crate::output::{GecDiagnostic, GecSeverity};

/// Result of gating a single item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    /// The item is accepted for generation.
    Accept,
    /// The item is rejected with a reason.
    Reject(String),
}

/// Gate all items in a package, returning per-item decisions.
pub fn gate_package(
    pkg: &BindingPackage,
    validation: Option<&ValidationReport>,
) -> (Vec<GateDecision>, Vec<GecDiagnostic>) {
    let mut decisions = Vec::new();
    let mut diags = Vec::new();

    for item in &pkg.items {
        let decision = gate_item(item, validation);
        if let GateDecision::Reject(ref reason) = decision {
            diags.push(GecDiagnostic {
                severity: GecSeverity::Warning,
                message: reason.clone(),
                item_name: item_name(item),
            });
        }
        decisions.push(decision);
    }

    (decisions, diags)
}

fn gate_item(item: &BindingItem, _validation: Option<&ValidationReport>) -> GateDecision {
    match item {
        BindingItem::Function(f) => gate_function(f, _validation),
        BindingItem::Record(r) => gate_record(r),
        BindingItem::Enum(e) => gate_enum(e),
        BindingItem::TypeAlias(_) => GateDecision::Accept,
        BindingItem::Variable(v) => gate_variable(v, _validation),
        BindingItem::Unsupported(u) => {
            GateDecision::Reject(format!("linc marked as unsupported: {}", u.reason))
        }
    }
}

/// 6.3: Required evidence rules for function signatures.
fn gate_function(f: &FunctionBinding, validation: Option<&ValidationReport>) -> GateDecision {
    if let Some(report) = validation {
        let Some(symbol_match) = validation_match_for_function(report, &f.name) else {
            return GateDecision::Reject(format!(
                "function '{}' lacks validation evidence",
                f.name
            ));
        };
        if symbol_match.status == linc::MatchStatus::AbiShapeMismatch {
            return GateDecision::Reject(format!(
                "function '{}' has ABI mismatch validation evidence",
                f.name
            ));
        }
    }

    // Check for unsupported parameter types
    for param in &f.parameters {
        if has_bitfield_in_signature(&param.ty) {
            return GateDecision::Reject(format!(
                "function '{}' has bitfield in parameter — by-value bitfield passing is unsound",
                f.name
            ));
        }
    }
    GateDecision::Accept
}

fn validation_match_for_function<'a>(
    report: &'a ValidationReport,
    name: &str,
) -> Option<&'a linc::SymbolMatch> {
    report
        .matches
        .iter()
        .find(|m| m.item_kind == linc::ItemKind::Function && m.name == name)
}

fn gate_variable(v: &linc::VariableBinding, validation: Option<&ValidationReport>) -> GateDecision {
    if let Some(report) = validation {
        let Some(symbol_match) = validation_match_for_variable(report, &v.name) else {
            return GateDecision::Reject(format!(
                "variable '{}' lacks validation evidence",
                v.name
            ));
        };
        if symbol_match.status == linc::MatchStatus::AbiShapeMismatch {
            return GateDecision::Reject(format!(
                "variable '{}' has ABI mismatch validation evidence",
                v.name
            ));
        }
    }

    GateDecision::Accept
}

fn validation_match_for_variable<'a>(
    report: &'a ValidationReport,
    name: &str,
) -> Option<&'a linc::SymbolMatch> {
    report
        .matches
        .iter()
        .find(|m| m.item_kind == linc::ItemKind::Variable && m.name == name)
}

/// 6.1: Required evidence rules for by-value structs.
/// 6.5: Policy for partial bitfields and packed records.
fn gate_record(r: &RecordBinding) -> GateDecision {
    // Anonymous records cannot be projected
    if r.name.is_none() {
        return GateDecision::Reject("anonymous record cannot be projected".into());
    }

    // Opaque records are always accepted (as opaque types)
    if r.fields.is_none() {
        return GateDecision::Accept;
    }

    let fields = r.fields.as_ref().unwrap();

    // 6.5: Reject records with bitfields (conservative)
    if fields.iter().any(|f| f.is_bitfield()) {
        return GateDecision::Reject(format!(
            "record '{}' contains bitfields — not projected",
            r.name.as_deref().unwrap_or("<anon>")
        ));
    }

    // 6.4: Opaque/incomplete fields
    for field in fields {
        if field_is_incomplete(field) {
            return GateDecision::Reject(format!(
                "record '{}' has incomplete field types",
                r.name.as_deref().unwrap_or("<anon>")
            ));
        }
    }

    GateDecision::Accept
}

/// 6.2: Required evidence rules for by-value enums.
fn gate_enum(e: &EnumBinding) -> GateDecision {
    if e.name.is_none() {
        return GateDecision::Reject("anonymous enum cannot be projected".into());
    }
    GateDecision::Accept
}

fn has_bitfield_in_signature(_ty: &linc::BindingType) -> bool {
    // Bitfields cannot appear directly in function signatures in C,
    // but structs containing bitfields can be passed by value.
    // This is a conservative check — we let the record gate handle it.
    false
}

fn field_is_incomplete(field: &FieldBinding) -> bool {
    matches!(&field.ty, linc::BindingType::Opaque(_))
}

fn item_name(item: &BindingItem) -> Option<String> {
    match item {
        BindingItem::Function(f) => Some(f.name.clone()),
        BindingItem::Record(r) => r.name.clone(),
        BindingItem::Enum(e) => e.name.clone(),
        BindingItem::TypeAlias(t) => Some(t.name.clone()),
        BindingItem::Variable(v) => Some(v.name.clone()),
        BindingItem::Unsupported(u) => u.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linc::*;

    fn make_package(items: Vec<BindingItem>) -> BindingPackage {
        let mut pkg = BindingPackage::new();
        pkg.items = items;
        pkg
    }

    // 6.1: by-value struct evidence
    #[test]
    fn accept_normal_struct() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("point".into()),
            fields: Some(vec![FieldBinding {
                name: Some("x".into()),
                ty: BindingType::Int,
                bit_width: None,
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        assert_eq!(decisions[0], GateDecision::Accept);
    }

    // 6.1/6.5: reject bitfield struct
    #[test]
    fn reject_bitfield_struct() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("flags".into()),
            fields: Some(vec![FieldBinding {
                name: Some("bits".into()),
                ty: BindingType::UInt,
                bit_width: Some(3),
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, diags) = gate_package(&pkg, None);
        assert_eq!(
            decisions[0],
            GateDecision::Reject("record 'flags' contains bitfields — not projected".into())
        );
        assert_eq!(diags.len(), 1);
    }

    // 6.2: enum acceptance
    #[test]
    fn accept_named_enum() {
        let pkg = make_package(vec![BindingItem::Enum(EnumBinding {
            name: Some("color".into()),
            variants: vec![],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        assert_eq!(decisions[0], GateDecision::Accept);
    }

    // 6.2: reject anonymous enum
    #[test]
    fn reject_anonymous_enum() {
        let pkg = make_package(vec![BindingItem::Enum(EnumBinding {
            name: None,
            variants: vec![],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        match &decisions[0] {
            GateDecision::Reject(r) => assert!(r.contains("anonymous")),
            GateDecision::Accept => panic!("should reject anonymous enum"),
        }
    }

    // 6.3: function acceptance
    #[test]
    fn accept_simple_function() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        assert_eq!(decisions[0], GateDecision::Accept);
    }

    #[test]
    fn reject_function_without_validation_match() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: Vec::new(),
        };

        let (decisions, diags) = gate_package(&pkg, Some(&report));
        assert_eq!(
            decisions[0],
            GateDecision::Reject("function 'foo' lacks validation evidence".into())
        );
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn accept_function_with_validation_match() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![SymbolMatch {
                name: "foo".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::Matched,
                visibility: None,
                provider_artifacts: Vec::new(),
                confidence: MatchConfidence::High,
                evidence_kind: EvidenceKind::ExactExported,
            }],
        };

        let (decisions, diags) = gate_package(&pkg, Some(&report));
        assert_eq!(decisions[0], GateDecision::Accept);
        assert!(diags.is_empty());
    }

    #[test]
    fn reject_variable_without_validation_match() {
        let pkg = make_package(vec![BindingItem::Variable(VariableBinding {
            name: "GLOBAL".into(),
            ty: BindingType::Int,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: Vec::new(),
        };

        let (decisions, diags) = gate_package(&pkg, Some(&report));
        assert_eq!(
            decisions[0],
            GateDecision::Reject("variable 'GLOBAL' lacks validation evidence".into())
        );
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn accept_variable_with_validation_match() {
        let pkg = make_package(vec![BindingItem::Variable(VariableBinding {
            name: "GLOBAL".into(),
            ty: BindingType::Int,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![SymbolMatch {
                name: "GLOBAL".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::Matched,
                visibility: None,
                provider_artifacts: Vec::new(),
                confidence: MatchConfidence::High,
                evidence_kind: EvidenceKind::ExactExported,
            }],
        };

        let (decisions, diags) = gate_package(&pkg, Some(&report));
        assert_eq!(decisions[0], GateDecision::Accept);
        assert!(diags.is_empty());
    }

    #[test]
    fn reject_function_with_abi_mismatch_validation() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![SymbolMatch {
                name: "foo".into(),
                item_kind: ItemKind::Function,
                status: MatchStatus::AbiShapeMismatch,
                visibility: None,
                provider_artifacts: Vec::new(),
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::AbiShapeMismatch,
            }],
        };

        let (decisions, _) = gate_package(&pkg, Some(&report));
        assert_eq!(
            decisions[0],
            GateDecision::Reject("function 'foo' has ABI mismatch validation evidence".into())
        );
    }

    #[test]
    fn reject_variable_with_abi_mismatch_validation() {
        let pkg = make_package(vec![BindingItem::Variable(VariableBinding {
            name: "GLOBAL".into(),
            ty: BindingType::Int,
            source_offset: None,
        })]);
        let report = ValidationReport {
            phases: Vec::new(),
            entries: Vec::new(),
            summary: ValidationSummary::default(),
            matches: vec![SymbolMatch {
                name: "GLOBAL".into(),
                item_kind: ItemKind::Variable,
                status: MatchStatus::AbiShapeMismatch,
                visibility: None,
                provider_artifacts: Vec::new(),
                confidence: MatchConfidence::Low,
                evidence_kind: EvidenceKind::AbiShapeMismatch,
            }],
        };

        let (decisions, _) = gate_package(&pkg, Some(&report));
        assert_eq!(
            decisions[0],
            GateDecision::Reject("variable 'GLOBAL' has ABI mismatch validation evidence".into())
        );
    }

    // 6.4: opaque record accepted
    #[test]
    fn accept_opaque_record() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("FILE".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        assert_eq!(decisions[0], GateDecision::Accept);
    }

    // 6.4: reject incomplete field types
    #[test]
    fn reject_incomplete_field() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("wrapper".into()),
            fields: Some(vec![FieldBinding {
                name: Some("inner".into()),
                ty: BindingType::Opaque("unknown_type".into()),
                bit_width: None,
                layout: None,
            }]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        match &decisions[0] {
            GateDecision::Reject(r) => assert!(r.contains("incomplete")),
            GateDecision::Accept => panic!("should reject incomplete field"),
        }
    }

    // 6.6: unsupported items rejected
    #[test]
    fn reject_unsupported() {
        let pkg = make_package(vec![BindingItem::Unsupported(UnsupportedItem {
            name: Some("bad".into()),
            reason: "not supported by bic".into(),
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        match &decisions[0] {
            GateDecision::Reject(r) => assert!(r.contains("unsupported")),
            GateDecision::Accept => panic!("should reject unsupported"),
        }
    }

    // 6.8: typed refusal diagnostics
    #[test]
    fn refusal_diagnostics_generated() {
        let pkg = make_package(vec![
            BindingItem::Record(RecordBinding {
                kind: RecordKind::Struct,
                name: Some("bf".into()),
                fields: Some(vec![FieldBinding {
                    name: Some("x".into()),
                    ty: BindingType::UInt,
                    bit_width: Some(4),
                    layout: None,
                }]),
                representation: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::Function(FunctionBinding {
                name: "ok".into(),
                calling_convention: CallingConvention::C,
                parameters: vec![],
                return_type: BindingType::Void,
                variadic: false,
                source_offset: None,
            }),
        ]);
        let (decisions, diags) = gate_package(&pkg, None);
        assert!(matches!(decisions[0], GateDecision::Reject(_)));
        assert_eq!(decisions[1], GateDecision::Accept);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, GecSeverity::Warning);
    }

    // 6.9: accepted vs rejected generation tests
    #[test]
    fn mixed_accept_reject() {
        let pkg = make_package(vec![
            BindingItem::Function(FunctionBinding {
                name: "good".into(),
                calling_convention: CallingConvention::C,
                parameters: vec![],
                return_type: BindingType::Void,
                variadic: false,
                source_offset: None,
            }),
            BindingItem::Record(RecordBinding {
                kind: RecordKind::Struct,
                name: None,
                fields: Some(vec![]),
                representation: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::TypeAlias(TypeAliasBinding {
                name: "size_t".into(),
                target: BindingType::ULong,
                canonical_resolution: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::Variable(VariableBinding {
                name: "errno".into(),
                ty: BindingType::Int,
                source_offset: None,
            }),
        ]);
        let (decisions, _) = gate_package(&pkg, None);
        assert_eq!(decisions[0], GateDecision::Accept);
        assert!(matches!(decisions[1], GateDecision::Reject(_)));
        assert_eq!(decisions[2], GateDecision::Accept);
        assert_eq!(decisions[3], GateDecision::Accept);
    }

    // Anonymous record rejected
    #[test]
    fn reject_anonymous_record() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: None,
            fields: Some(vec![]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (decisions, _) = gate_package(&pkg, None);
        match &decisions[0] {
            GateDecision::Reject(r) => assert!(r.contains("anonymous")),
            GateDecision::Accept => panic!("should reject"),
        }
    }
}
