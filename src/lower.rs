//! Item lowering from `gerc`'s C-side declarations to Rust projection IR.
//!
//! This module converts `BindingItem` values into `gerc::ir::RustItem`
//! values, using the type mapping from `typemap`.

use crate::c::{
    BindingItem, BindingPackage, DeclarationProvenance, EnumBinding, FunctionBinding, MacroBinding,
    MacroCategory, MacroValue, RecordBinding, RecordKind, TypeAliasBinding, UnsupportedItem,
    VariableBinding, BindingType,
};

use crate::ir::*;
use crate::output::{GercDiagnostic, GercSeverity};
use crate::typemap::map_type;

/// Lower an entire `BindingPackage` into a `RustProjection`.
///
/// Returns the projection and any diagnostics produced during lowering.
pub fn lower_package(pkg: &BindingPackage) -> (RustProjection, Vec<GercDiagnostic>) {
    let mut proj = RustProjection::new();
    let mut diags = Vec::new();

    // Lower items
    for (index, item) in pkg.items.iter().enumerate() {
        match lower_item(item) {
            Ok(rust_item) => {
                let name = rust_item_name(&rust_item);
                proj.notes.push(ProjectionNote {
                    kind: NoteKind::Projected,
                    message: "lowered from gerc C model".into(),
                    item_name: name.clone(),
                });
                proj.items.push(rust_item);
                if let Some(note) = provenance_note(pkg.provenance.get(index), name) {
                    proj.notes.push(note);
                }
            }
            Err(reason) => {
                let name = binding_item_name(item);
                diags.push(GercDiagnostic {
                    severity: GercSeverity::Warning,
                    message: reason.clone(),
                    item_name: name.clone(),
                });
                proj.items.push(RustItem::Unsupported(RustUnsupported {
                    name: name.clone(),
                    reason,
                }));
                proj.notes.push(ProjectionNote {
                    kind: NoteKind::Unsupported,
                    message: "could not lower".into(),
                    item_name: name.clone(),
                });
                if let Some(note) = provenance_note(pkg.provenance.get(index), name) {
                    proj.notes.push(note);
                }
            }
        }
    }

    // Lower macro constants
    for mac in &pkg.macros {
        if let Some(rust_item) = lower_macro(mac) {
            proj.items.push(rust_item);
        }
    }

    (proj, diags)
}

/// Lower a single declaration item into a `RustItem`.
fn lower_item(item: &BindingItem) -> Result<RustItem, String> {
    match item {
        BindingItem::Function(f) => lower_function(f),
        BindingItem::Record(r) => lower_record(r),
        BindingItem::Enum(e) => lower_enum(e),
        BindingItem::TypeAlias(t) => lower_type_alias(t),
        BindingItem::Variable(v) => lower_variable(v),
        BindingItem::Unsupported(u) => lower_unsupported(u),
    }
}

/// 5.1: Lower function declarations.
fn lower_function(f: &FunctionBinding) -> Result<RustItem, String> {
    let parameters: Vec<RustParameter> = f
        .parameters
        .iter()
        .enumerate()
        .map(|(i, p)| RustParameter {
            name: p.name.clone().unwrap_or_else(|| format!("arg{}", i)),
            ty: map_type(&p.ty),
        })
        .collect();

    Ok(RustItem::Function(RustFunction {
        name: f.name.clone(),
        parameters,
        return_type: map_type(&f.return_type),
        variadic: f.variadic,
        doc: Some(format!("C function: {}", f.name)),
    }))
}

/// 5.2: Lower global variable declarations.
fn lower_variable(v: &VariableBinding) -> Result<RustItem, String> {
    Ok(RustItem::Static(RustStatic {
        name: v.name.clone(),
        ty: map_type(&v.ty),
        mutable: true,
        doc: Some(format!("C global: {}", v.name)),
    }))
}

/// 5.3 & 5.4: Lower struct and union record declarations.
fn lower_record(r: &RecordBinding) -> Result<RustItem, String> {
    let name = r
        .name
        .clone()
        .ok_or_else(|| "anonymous record cannot be projected".to_string())?;

    let kind = match r.kind {
        RecordKind::Struct => RustRecordKind::Struct,
        RecordKind::Union => RustRecordKind::Union,
    };

    let (fields, is_opaque) = match &r.fields {
        Some(bic_fields) => {
            let fields = bic_fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let field_name = f.name.clone().unwrap_or_else(|| format!("__field{}", i));
                    RustField {
                        name: field_name,
                        ty: map_type(&f.ty),
                    }
                })
                .collect();
            (fields, false)
        }
        None => (vec![], true),
    };

    Ok(RustItem::Record(RustRecord {
        name,
        kind,
        fields,
        is_opaque,
        doc: None,
    }))
}

/// 5.5: Lower enum declarations.
fn lower_enum(e: &EnumBinding) -> Result<RustItem, String> {
    let name = e
        .name
        .clone()
        .ok_or_else(|| "anonymous enum cannot be projected".to_string())?;

    let variants: Vec<RustEnumVariant> = e
        .variants
        .iter()
        .map(|v| RustEnumVariant {
            name: v.name.clone(),
            value: v.value,
        })
        .collect();

    // Choose repr based on representation evidence
    let repr = match &e.representation {
        Some(rep) => match (rep.underlying_size, rep.is_signed) {
            (Some(4), Some(false)) => "c_uint".into(),
            (Some(4), _) => "c_int".into(),
            (Some(2), Some(false)) => "c_ushort".into(),
            (Some(2), _) => "c_short".into(),
            (Some(1), Some(false)) => "c_uchar".into(),
            (Some(1), _) => "c_schar".into(),
            (Some(8), Some(false)) => "c_ulonglong".into(),
            (Some(8), _) => "c_longlong".into(),
            _ => "c_int".into(),
        },
        None => "c_int".into(),
    };

    Ok(RustItem::Enum(RustEnum {
        name,
        variants,
        repr,
        doc: None,
    }))
}

/// 5.6: Lower typedef aliases.
fn lower_type_alias(t: &TypeAliasBinding) -> Result<RustItem, String> {
    if let Some(target) = lower_unnameable_alias_target(&t.target) {
        return Ok(RustItem::TypeAlias(RustTypeAlias {
            name: t.name.clone(),
            target,
            doc: None,
        }));
    }

    let mapped = map_type(&t.target);
    if matches!(&mapped, RustType::Named(name) if name == &t.name) {
        return Err(format!(
            "type alias '{}' is a redundant self-alias for an emitted named declaration",
            t.name
        ));
    }

    if binding_type_is_unnameable(&t.target) {
        return Err(format!(
            "type alias '{}' targets an anonymous or unnameable declaration",
            t.name
        ));
    }

    Ok(RustItem::TypeAlias(RustTypeAlias {
        name: t.name.clone(),
        target: mapped,
        doc: None,
    }))
}

fn lower_unnameable_alias_target(ty: &BindingType) -> Option<RustType> {
    match ty {
        BindingType::Pointer {
            pointee,
            const_pointee,
            ..
        } if binding_type_is_unnameable(pointee) => Some(RustType::OpaquePtr {
            is_const: *const_pointee,
        }),
        BindingType::Qualified { ty, .. } => lower_unnameable_alias_target(ty),
        _ => None,
    }
}

fn binding_type_is_unnameable(ty: &BindingType) -> bool {
    match ty {
        BindingType::TypedefRef(name)
        | BindingType::RecordRef(name)
        | BindingType::EnumRef(name)
        | BindingType::Opaque(name) => {
            let trimmed = name.trim();
            trimmed.is_empty() || trimmed == "<anonymous>"
        }
        BindingType::Pointer { pointee, .. } => binding_type_is_unnameable(pointee),
        BindingType::Array(element, _) => binding_type_is_unnameable(element),
        BindingType::Qualified { ty, .. } => binding_type_is_unnameable(ty),
        BindingType::FunctionPointer {
            return_type,
            parameters,
            ..
        } => {
            binding_type_is_unnameable(return_type)
                || parameters.iter().any(binding_type_is_unnameable)
        }
        _ => false,
    }
}

/// 5.7: Lower macro constants into Rust constants where supported.
fn lower_macro(mac: &MacroBinding) -> Option<RustItem> {
    if mac.category != MacroCategory::BindableConstant {
        return None;
    }

    match &mac.value {
        Some(MacroValue::Integer(val)) => Some(RustItem::Constant(RustConstant {
            name: mac.name.clone(),
            ty: RustType::CInt,
            value: val.to_string(),
            doc: Some(format!("C macro: {}", mac.name)),
        })),
        Some(MacroValue::String(val)) => Some(RustItem::Constant(RustConstant {
            name: mac.name.clone(),
            ty: RustType::Pointer {
                pointee: Box::new(RustType::CChar),
                is_const: true,
            },
            value: format!("\"{}\"", val),
            doc: Some(format!("C macro: {}", mac.name)),
        })),
        None => None,
    }
}

/// 5.8: Mark unsupported declarations explicitly.
fn lower_unsupported(u: &UnsupportedItem) -> Result<RustItem, String> {
    Err(format!(
        "unsupported: {}",
        u.name.as_deref().unwrap_or("<anonymous>")
    ))
}

fn rust_item_name(item: &RustItem) -> Option<String> {
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

fn binding_item_name(item: &BindingItem) -> Option<String> {
    match item {
        BindingItem::Function(f) => Some(f.name.clone()),
        BindingItem::Record(r) => r.name.clone(),
        BindingItem::Enum(e) => e.name.clone(),
        BindingItem::TypeAlias(t) => Some(t.name.clone()),
        BindingItem::Variable(v) => Some(v.name.clone()),
        BindingItem::Unsupported(u) => u.name.clone(),
    }
}

fn provenance_note(
    provenance: Option<&DeclarationProvenance>,
    item_name: Option<String>,
) -> Option<ProjectionNote> {
    let provenance = provenance?;
    let message = format_provenance_message(provenance)?;
    Some(ProjectionNote {
        kind: NoteKind::Provenance,
        message,
        item_name,
    })
}

fn format_provenance_message(provenance: &DeclarationProvenance) -> Option<String> {
    let mut details = Vec::new();

    if let Some(location) = &provenance.source_location {
        let mut location_str = location.file.clone();
        if let Some(line) = location.line {
            location_str.push(':');
            location_str.push_str(&line.to_string());
            if let Some(column) = location.column {
                location_str.push(':');
                location_str.push_str(&column.to_string());
            }
        }
        details.push(format!("declared at {location_str}"));
    }

    if let Some(origin) = &provenance.source_origin {
        details.push(format!("origin {:?}", origin));
    }

    if let Some(offset) = provenance.source_offset {
        details.push(format!("offset {offset}"));
    }

    if details.is_empty() {
        return None;
    }

    Some(details.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c::*;

    fn make_package(items: Vec<BindingItem>) -> BindingPackage {
        let mut pkg = BindingPackage::new();
        pkg.items = items;
        pkg
    }

    // 5.1: function lowering
    #[test]
    fn lower_simple_function() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "foo".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: Some("x".into()),
                ty: BindingType::Int,
            }],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert!(diags.is_empty());
        assert_eq!(proj.len(), 1);
        match &proj.items[0] {
            RustItem::Function(f) => {
                assert_eq!(f.name, "foo");
                assert_eq!(f.parameters.len(), 1);
                assert_eq!(f.parameters[0].name, "x");
            }
            other => panic!("expected Function, got {:?}", other),
        }
    }

    #[test]
    fn lower_function_unnamed_params() {
        let pkg = make_package(vec![BindingItem::Function(FunctionBinding {
            name: "bar".into(),
            calling_convention: CallingConvention::C,
            parameters: vec![ParameterBinding {
                name: None,
                ty: BindingType::Int,
            }],
            return_type: BindingType::Int,
            variadic: false,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Function(f) => assert_eq!(f.parameters[0].name, "arg0"),
            other => panic!("expected Function, got {:?}", other),
        }
    }

    // 5.2: variable lowering
    #[test]
    fn lower_variable() {
        let pkg = make_package(vec![BindingItem::Variable(VariableBinding {
            name: "errno".into(),
            ty: BindingType::Int,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Static(s) => {
                assert_eq!(s.name, "errno");
                assert!(s.mutable);
            }
            other => panic!("expected Static, got {:?}", other),
        }
    }

    // 5.3: struct lowering
    #[test]
    fn lower_struct() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("point".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("x".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("y".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Record(r) => {
                assert_eq!(r.name, "point");
                assert_eq!(r.kind, RustRecordKind::Struct);
                assert!(!r.is_opaque);
                assert_eq!(r.fields.len(), 2);
            }
            other => panic!("expected Record, got {:?}", other),
        }
    }

    // 5.4: union lowering
    #[test]
    fn lower_union() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Union,
            name: Some("data".into()),
            fields: Some(vec![
                FieldBinding {
                    name: Some("i".into()),
                    ty: BindingType::Int,
                    bit_width: None,
                    layout: None,
                },
                FieldBinding {
                    name: Some("f".into()),
                    ty: BindingType::Float,
                    bit_width: None,
                    layout: None,
                },
            ]),
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Record(r) => {
                assert_eq!(r.kind, RustRecordKind::Union);
            }
            other => panic!("expected Record, got {:?}", other),
        }
    }

    // Opaque struct
    #[test]
    fn lower_opaque_struct() {
        let pkg = make_package(vec![BindingItem::Record(RecordBinding {
            kind: RecordKind::Struct,
            name: Some("FILE".into()),
            fields: None,
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Record(r) => {
                assert!(r.is_opaque);
                assert!(r.fields.is_empty());
            }
            other => panic!("expected Record, got {:?}", other),
        }
    }

    // 5.5: enum lowering
    #[test]
    fn lower_enum() {
        let pkg = make_package(vec![BindingItem::Enum(EnumBinding {
            name: Some("color".into()),
            variants: vec![
                EnumVariant {
                    name: "RED".into(),
                    value: Some(0),
                },
                EnumVariant {
                    name: "GREEN".into(),
                    value: Some(1),
                },
            ],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::Enum(e) => {
                assert_eq!(e.name, "color");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.repr, "c_int");
            }
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    // Anonymous enum rejected
    #[test]
    fn lower_anonymous_enum_rejected() {
        let pkg = make_package(vec![BindingItem::Enum(EnumBinding {
            name: None,
            variants: vec![],
            representation: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert_eq!(diags.len(), 1);
        match &proj.items[0] {
            RustItem::Unsupported(_) => {}
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    // 5.6: typedef alias lowering
    #[test]
    fn lower_typedef() {
        let pkg = make_package(vec![BindingItem::TypeAlias(TypeAliasBinding {
            name: "size_t".into(),
            target: BindingType::ULong,
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, _) = lower_package(&pkg);
        match &proj.items[0] {
            RustItem::TypeAlias(t) => {
                assert_eq!(t.name, "size_t");
                assert_eq!(t.target, RustType::CULong);
            }
            other => panic!("expected TypeAlias, got {:?}", other),
        }
    }

    // 5.7: macro constant lowering
    #[test]
    fn lower_integer_macro() {
        let mut pkg = BindingPackage::new();
        pkg.macros.push(MacroBinding {
            name: "API_LEVEL".into(),
            body: "3".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Integer,
            category: MacroCategory::BindableConstant,
            value: Some(MacroValue::Integer(3)),
        });
        let (proj, _) = lower_package(&pkg);
        assert_eq!(proj.len(), 1);
        match &proj.items[0] {
            RustItem::Constant(c) => {
                assert_eq!(c.name, "API_LEVEL");
                assert_eq!(c.value, "3");
            }
            other => panic!("expected Constant, got {:?}", other),
        }
    }

    #[test]
    fn skip_non_bindable_macro() {
        let mut pkg = BindingPackage::new();
        pkg.macros.push(MacroBinding {
            name: "DEBUG".into(),
            body: "".into(),
            function_like: false,
            form: MacroForm::ObjectLike,
            kind: MacroKind::Other,
            category: MacroCategory::ConfigurationFlag,
            value: None,
        });
        let (proj, _) = lower_package(&pkg);
        assert!(proj.is_empty());
    }

    // 5.8: unsupported items
    #[test]
    fn lower_unsupported_item() {
        let pkg = make_package(vec![BindingItem::Unsupported(UnsupportedItem {
            name: Some("bad_thing".into()),
            reason: "not supported".into(),
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert_eq!(diags.len(), 1);
        match &proj.items[0] {
            RustItem::Unsupported(u) => assert!(u.reason.contains("bad_thing")),
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    #[test]
    fn lower_anonymous_alias_target_becomes_unsupported() {
        let pkg = make_package(vec![BindingItem::TypeAlias(TypeAliasBinding {
            name: "max_align_t".into(),
            target: BindingType::RecordRef("<anonymous>".into()),
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert_eq!(diags.len(), 1);
        match &proj.items[0] {
            RustItem::Unsupported(u) => assert!(u.reason.contains("anonymous")),
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    #[test]
    fn lower_pointer_to_anonymous_alias_becomes_opaque_ptr() {
        let pkg = make_package(vec![BindingItem::TypeAlias(TypeAliasBinding {
            name: "png_imagep".into(),
            target: BindingType::ptr(BindingType::RecordRef("<anonymous>".into())),
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert!(diags.is_empty());
        match &proj.items[0] {
            RustItem::TypeAlias(t) => assert_eq!(t.target, RustType::OpaquePtr { is_const: false }),
            other => panic!("expected TypeAlias, got {:?}", other),
        }
    }

    #[test]
    fn lower_self_alias_becomes_unsupported() {
        let pkg = make_package(vec![BindingItem::TypeAlias(TypeAliasBinding {
            name: "pthread_attr_t".into(),
            target: BindingType::RecordRef("pthread_attr_t".into()),
            canonical_resolution: None,
            abi_confidence: None,
            source_offset: None,
        })]);
        let (proj, diags) = lower_package(&pkg);
        assert_eq!(diags.len(), 1);
        match &proj.items[0] {
            RustItem::Unsupported(u) => assert!(u.reason.contains("self-alias")),
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    // 5.9: provenance comments
    #[test]
    fn provenance_notes_generated() {
        let mut pkg = make_package(vec![
            BindingItem::Function(FunctionBinding {
                name: "foo".into(),
                calling_convention: CallingConvention::C,
                parameters: vec![],
                return_type: BindingType::Void,
                variadic: false,
                source_offset: None,
            }),
            BindingItem::Unsupported(UnsupportedItem {
                name: Some("bad".into()),
                reason: "nope".into(),
                source_offset: None,
            }),
        ]);
        pkg.provenance.push(DeclarationProvenance {
            item_name: Some("foo".into()),
            item_kind: Some(BindingItemKind::Function),
            source_offset: Some(12),
            source_origin: Some(SourceOrigin::Entry),
            source_location: Some(SourceLocation {
                file: "demo.h".into(),
                line: Some(7),
                column: Some(3),
            }),
        });
        pkg.provenance.push(DeclarationProvenance {
            item_name: Some("bad".into()),
            item_kind: Some(BindingItemKind::Unsupported),
            source_offset: Some(18),
            source_origin: Some(SourceOrigin::UserInclude),
            source_location: Some(SourceLocation {
                file: "detail.h".into(),
                line: Some(9),
                column: Some(1),
            }),
        });

        let (proj, _) = lower_package(&pkg);
        assert_eq!(proj.notes.len(), 4);
        assert_eq!(proj.notes[0].kind, NoteKind::Projected);
        assert_eq!(proj.notes[1].kind, NoteKind::Provenance);
        assert!(proj.notes[1].message.contains("demo.h:7:3"));
        assert!(proj.notes[1].message.contains("origin Entry"));
        assert_eq!(proj.notes[2].kind, NoteKind::Unsupported);
        assert_eq!(proj.notes[3].kind, NoteKind::Provenance);
        assert!(proj.notes[3].message.contains("detail.h:9:1"));
    }

    // 5.10: mixed header surface fixture
    #[test]
    fn lower_mixed_surface() {
        let pkg = make_package(vec![
            BindingItem::TypeAlias(TypeAliasBinding {
                name: "uint32_t".into(),
                target: BindingType::UInt,
                canonical_resolution: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::Record(RecordBinding {
                kind: RecordKind::Struct,
                name: Some("config".into()),
                fields: Some(vec![FieldBinding {
                    name: Some("flags".into()),
                    ty: BindingType::UInt,
                    bit_width: None,
                    layout: None,
                }]),
                representation: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::Function(FunctionBinding {
                name: "init".into(),
                calling_convention: CallingConvention::C,
                parameters: vec![ParameterBinding {
                    name: Some("cfg".into()),
                    ty: BindingType::ptr(BindingType::RecordRef("config".into())),
                }],
                return_type: BindingType::Int,
                variadic: false,
                source_offset: None,
            }),
            BindingItem::Enum(EnumBinding {
                name: Some("status".into()),
                variants: vec![
                    EnumVariant {
                        name: "OK".into(),
                        value: Some(0),
                    },
                    EnumVariant {
                        name: "ERR".into(),
                        value: Some(1),
                    },
                ],
                representation: None,
                abi_confidence: None,
                source_offset: None,
            }),
            BindingItem::Variable(VariableBinding {
                name: "errno".into(),
                ty: BindingType::Int,
                source_offset: None,
            }),
        ]);
        let (proj, diags) = lower_package(&pkg);
        assert!(diags.is_empty());
        assert_eq!(proj.len(), 5);
    }
}
