use std::{ffi::OsStr, path::Path};

use linc::contract::SymbolDecoration;
use parc::contract::{
    AttributeDisposition, DeclarationIdentity, EntityNamespace, EntityScope, ExactInteger, Linkage,
    MacroCategory, MacroForm, MacroValue, Nullability, ObjectFormat, RecordCompleteness,
    RecordKind, SourceAttribute, SourceName, SourceOrigin, SourceProvenance, SourceRange,
    StorageClass, SupportStatus, Visibility,
};

use crate::{
    GeneratedFileSet, GenerationContext, GenerationDiagnostic, GenerationFingerprint,
    ItemSelection, RustItem, RustLinkAtom, RustLinkPlan, RustRecordKind, RustType, RustTypeKind,
    ValidatedRustProjection, GENERATION_ALGORITHM_ID, GENERATION_SCHEMA_ID,
    GENERATION_SCHEMA_VERSION, GENERATOR_IDENTITY,
};

pub(crate) fn generation_fingerprint(
    context: GenerationContext,
    selection: &ItemSelection,
    projection: &ValidatedRustProjection,
    diagnostics: &[GenerationDiagnostic],
    files: &GeneratedFileSet,
    link_plan: &RustLinkPlan,
) -> GenerationFingerprint {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, b"follang.gerc.generation-fingerprint.v1");
    hash_field(&mut hasher, GENERATION_SCHEMA_ID.as_bytes());
    hash_field(&mut hasher, &GENERATION_SCHEMA_VERSION.to_le_bytes());
    hash_field(&mut hasher, GENERATION_ALGORITHM_ID.as_bytes());
    hash_field(&mut hasher, GENERATOR_IDENTITY.as_bytes());
    hash_field(&mut hasher, context.source_fingerprint().as_bytes());
    hash_field(&mut hasher, context.target_fingerprint().as_bytes());
    hash_field(&mut hasher, context.evidence_fingerprint().as_bytes());
    hash_field(&mut hasher, b"selection");
    hash_count(&mut hasher, selection.declarations().len());
    for declaration in selection.declarations() {
        hash_field(&mut hasher, declaration.as_bytes());
    }
    hash_projection(&mut hasher, projection);
    hash_field(&mut hasher, b"diagnostics");
    hash_count(&mut hasher, diagnostics.len());
    for diagnostic in diagnostics {
        hash_field(&mut hasher, diagnostic.code().as_str().as_bytes());
        if let Some(declaration) = diagnostic.declaration() {
            hash_field(&mut hasher, b"diagnostic-declaration");
            hash_field(&mut hasher, declaration.as_bytes());
        } else {
            hash_field(&mut hasher, b"no-diagnostic-declaration");
        }
        if let Some(macro_id) = diagnostic.macro_id() {
            hash_field(&mut hasher, b"diagnostic-macro");
            hash_field(&mut hasher, macro_id.as_bytes());
        } else {
            hash_field(&mut hasher, b"no-diagnostic-macro");
        }
    }
    hash_field(&mut hasher, b"generated-files");
    hash_count(&mut hasher, files.files().len());
    for file in files.files() {
        hash_native(&mut hasher, file.path().as_path().as_os_str());
        hash_field(&mut hasher, file.contents());
    }
    hash_field(&mut hasher, b"link-plan");
    hash_field(&mut hasher, link_plan.target_fingerprint().as_bytes());
    hash_field(
        &mut hasher,
        match link_plan.object_format() {
            ObjectFormat::Elf => b"object-format-elf",
            ObjectFormat::MachO => b"object-format-mach-o",
            ObjectFormat::Coff => b"object-format-coff",
            ObjectFormat::Wasm => b"object-format-wasm",
            ObjectFormat::Xcoff => b"object-format-xcoff",
        },
    );
    hash_count(&mut hasher, link_plan.atoms().len());
    for atom in link_plan.atoms() {
        match atom {
            RustLinkAtom::SearchNative(path) => {
                hash_field(&mut hasher, b"search-native");
                hash_path(&mut hasher, path);
            }
            RustLinkAtom::Artifact(artifact) => {
                hash_field(&mut hasher, artifact.kind().fingerprint_tag());
                hash_field(&mut hasher, artifact.provider().as_bytes());
                hash_field(&mut hasher, artifact.artifact_fingerprint().as_bytes());
                hash_path(&mut hasher, artifact.canonical_path());
            }
            RustLinkAtom::Framework {
                name,
                search_path,
                artifact,
            } => {
                hash_field(&mut hasher, b"framework");
                hash_native(&mut hasher, name);
                hash_path(&mut hasher, search_path);
                hash_field(&mut hasher, artifact.provider().as_bytes());
                hash_field(&mut hasher, artifact.artifact_fingerprint().as_bytes());
                hash_path(&mut hasher, artifact.canonical_path());
            }
            RustLinkAtom::GroupStart => hash_field(&mut hasher, b"group-start"),
            RustLinkAtom::GroupEnd => hash_field(&mut hasher, b"group-end"),
        }
    }
    GenerationFingerprint(*hasher.finalize().as_bytes())
}

fn hash_projection(hasher: &mut blake3::Hasher, projection: &ValidatedRustProjection) {
    hash_field(hasher, b"validated-rust-projection");
    hash_field(hasher, projection.target_fingerprint().as_bytes());
    hash_count(hasher, projection.items().len());
    for item in projection.items() {
        match item {
            RustItem::Function(function) => {
                hash_field(hasher, b"function");
                hash_field(hasher, function.declaration().as_bytes());
                hash_declaration_metadata(hasher, function.source());
                hash_field(hasher, function.rust_name().as_str().as_bytes());
                hash_field(hasher, function.link_name().as_bytes());
                hash_field(hasher, function.abi().spelling().as_bytes());
                hash_field(hasher, &[u8::from(function.variadic())]);
                hash_count(hasher, function.parameters().len());
                for parameter in function.parameters() {
                    hash_field(hasher, parameter.child().as_bytes());
                    hash_field(hasher, &parameter.ordinal().to_le_bytes());
                    hash_field(hasher, parameter.rust_name().as_str().as_bytes());
                    hash_optional_source_name(hasher, parameter.source_name());
                    hash_type(hasher, parameter.ty());
                    hash_range(hasher, parameter.range());
                    hash_provenance(hasher, parameter.provenance());
                    hash_attributes(hasher, parameter.attributes());
                    hash_support(hasher, parameter.support());
                }
                hash_type(hasher, function.return_type());
                hash_symbol(hasher, function.symbol());
            }
            RustItem::Record(record) => {
                hash_field(hasher, b"record");
                hash_field(hasher, record.declaration().as_bytes());
                hash_declaration_metadata(hasher, record.source());
                hash_field(hasher, record.rust_name().as_str().as_bytes());
                hash_field(
                    hasher,
                    match record.kind() {
                        RustRecordKind::Struct => b"struct",
                        RustRecordKind::Union => b"union",
                        RustRecordKind::Opaque => b"opaque",
                    },
                );
                hash_field(
                    hasher,
                    match record.source_kind() {
                        RecordKind::Struct => b"source-record-struct",
                        RecordKind::Union => b"source-record-union",
                    },
                );
                hash_field(
                    hasher,
                    match record.source_completeness() {
                        RecordCompleteness::Complete => b"source-record-complete",
                        RecordCompleteness::Incomplete => b"source-record-incomplete",
                    },
                );
                hash_field(
                    hasher,
                    &record.size_bits().unwrap_or(u64::MAX).to_le_bytes(),
                );
                hash_field(
                    hasher,
                    &record.alignment_bits().unwrap_or(u32::MAX).to_le_bytes(),
                );
                hash_optional_u32(hasher, record.packing_bits());
                hash_count(hasher, record.fields().len());
                for field in record.fields() {
                    hash_field(hasher, field.child().as_bytes());
                    hash_field(hasher, field.rust_name().as_str().as_bytes());
                    hash_optional_source_name(hasher, field.source_name());
                    hash_type(hasher, field.ty());
                    hash_field(hasher, &field.offset_bits().to_le_bytes());
                    hash_field(hasher, &field.size_bits().to_le_bytes());
                    hash_optional_u32(hasher, field.alignment_bits());
                    hash_range(hasher, field.range());
                    hash_provenance(hasher, field.provenance());
                    hash_attributes(hasher, field.attributes());
                    hash_support(hasher, field.support());
                    hash_strings(hasher, field.identity_tokens());
                    hash_field(hasher, &field.duplicate_ordinal().to_le_bytes());
                }
            }
            RustItem::Enum(enumeration) => {
                hash_field(hasher, b"enum");
                hash_field(hasher, enumeration.declaration().as_bytes());
                hash_declaration_metadata(hasher, enumeration.source());
                hash_field(hasher, enumeration.rust_name().as_str().as_bytes());
                hash_scalar(hasher, enumeration.storage());
                hash_field(hasher, &enumeration.alignment_bits().to_le_bytes());
                match enumeration.explicit_underlying_type() {
                    Some(ty) => {
                        hash_field(hasher, b"explicit-underlying-type");
                        hash_type(hasher, ty);
                    }
                    None => hash_field(hasher, b"no-explicit-underlying-type"),
                }
                hash_count(hasher, enumeration.variants().len());
                for variant in enumeration.variants() {
                    hash_field(hasher, variant.child().as_bytes());
                    hash_field(hasher, variant.rust_name().as_str().as_bytes());
                    hash_source_name(hasher, variant.source_name());
                    hash_exact_integer(hasher, variant.value());
                    hash_range(hasher, variant.range());
                    hash_provenance(hasher, variant.provenance());
                    hash_attributes(hasher, variant.attributes());
                    hash_support(hasher, variant.support());
                    hash_strings(hasher, variant.identity_tokens());
                    hash_field(hasher, &variant.duplicate_ordinal().to_le_bytes());
                }
            }
            RustItem::TypeAlias(alias) => {
                hash_field(hasher, b"type-alias");
                hash_field(hasher, alias.declaration().as_bytes());
                hash_declaration_metadata(hasher, alias.source());
                hash_field(hasher, alias.rust_name().as_str().as_bytes());
                hash_type(hasher, alias.target());
            }
            RustItem::Variable(variable) => {
                hash_field(hasher, b"variable");
                hash_field(hasher, variable.declaration().as_bytes());
                hash_declaration_metadata(hasher, variable.source());
                hash_field(hasher, variable.rust_name().as_str().as_bytes());
                hash_field(hasher, variable.link_name().as_bytes());
                hash_field(
                    hasher,
                    match variable.mutability() {
                        crate::RustVariableMutability::ReadOnly => b"read-only",
                        crate::RustVariableMutability::Mutable => b"mutable",
                    },
                );
                hash_field(hasher, &[u8::from(variable.thread_local())]);
                hash_type(hasher, variable.ty());
                hash_symbol(hasher, variable.symbol());
            }
        }
    }
    hash_field(hasher, b"projection-macros");
    hash_count(hasher, projection.macros().len());
    for source_macro in projection.macros() {
        hash_field(hasher, b"macro");
        hash_field(hasher, source_macro.macro_id().as_bytes());
        hash_field(hasher, source_macro.identity_file().as_bytes());
        hash_field(hasher, source_macro.rust_name().as_str().as_bytes());
        hash_field(hasher, source_macro.source_name().as_bytes());
        hash_macro_form(hasher, source_macro.form());
        hash_macro_category(hasher, source_macro.category());
        hash_field(hasher, source_macro.body().as_bytes());
        hash_strings(hasher, source_macro.normalized_tokens());
        hash_macro_value(hasher, source_macro.value());
        hash_count(hasher, source_macro.occurrences().len());
        for occurrence in source_macro.occurrences() {
            hash_field(hasher, occurrence.id.as_bytes());
            hash_range(hasher, occurrence.range);
            hash_strings(hasher, &occurrence.normalized_tokens);
            hash_field(hasher, &occurrence.duplicate_ordinal.to_le_bytes());
            hash_provenance(hasher, &occurrence.provenance);
        }
        hash_support(hasher, source_macro.support());
        hash_field(hasher, &[u8::from(source_macro.emitted())]);
    }
}

fn hash_symbol(hasher: &mut blake3::Hasher, symbol: &crate::NativeSymbolBinding) {
    hash_field(hasher, symbol.provider().as_bytes());
    hash_field(hasher, symbol.artifact_fingerprint().as_bytes());
    hash_native(hasher, symbol.artifact_path().as_os_str());
    hash_field(hasher, &symbol.symbol().table_index().to_le_bytes());
    hash_field(hasher, &symbol.symbol().symbol_index().to_le_bytes());
    hash_field(hasher, symbol.expected_name().as_bytes());
    hash_field(hasher, symbol.actual_name().as_bytes());
    hash_field(hasher, symbol.raw_name());
    match symbol.decoration() {
        SymbolDecoration::None => hash_field(hasher, b"decoration-none"),
        SymbolDecoration::LeadingUnderscore => hash_field(hasher, b"decoration-leading-underscore"),
        SymbolDecoration::Stdcall { stack_bytes } => {
            hash_field(hasher, b"decoration-stdcall");
            hash_field(hasher, &stack_bytes.to_le_bytes());
        }
        SymbolDecoration::Versioned {
            version,
            is_default,
        } => {
            hash_field(hasher, b"decoration-versioned");
            hash_field(hasher, version);
            hash_field(hasher, &[u8::from(*is_default)]);
        }
        SymbolDecoration::Other { spelling } => {
            hash_field(hasher, b"decoration-other");
            hash_field(hasher, spelling);
        }
    }
}

fn hash_type(hasher: &mut blake3::Hasher, ty: &RustType) {
    let qualifiers = ty.qualifiers();
    hash_field(
        hasher,
        &[
            u8::from(qualifiers.is_const),
            u8::from(qualifiers.is_volatile),
            u8::from(qualifiers.is_restrict),
            u8::from(qualifiers.is_atomic),
        ],
    );
    hash_support(hasher, ty.support());
    hash_field(
        hasher,
        match ty.nullability() {
            Nullability::Unspecified => b"unspecified",
            Nullability::Nonnull => b"nonnull",
            Nullability::Nullable => b"nullable",
            Nullability::NullUnspecified => b"null-unspecified",
        },
    );
    match ty.kind() {
        RustTypeKind::Void => hash_field(hasher, b"void"),
        RustTypeKind::Scalar(scalar) => hash_scalar(hasher, *scalar),
        RustTypeKind::Pointer(pointee) => {
            hash_field(hasher, b"pointer");
            hash_type(hasher, pointee);
        }
        RustTypeKind::FixedArray { element, elements } => {
            hash_field(hasher, b"fixed-array");
            hash_field(hasher, &elements.to_le_bytes());
            hash_type(hasher, element);
        }
        RustTypeKind::FlexibleArray { element } => {
            hash_field(hasher, b"flexible-array");
            hash_type(hasher, element);
        }
        RustTypeKind::Named {
            declaration,
            rust_name,
        } => {
            hash_field(hasher, b"named");
            hash_field(hasher, declaration.as_bytes());
            hash_field(hasher, rust_name.as_str().as_bytes());
        }
        RustTypeKind::FunctionPointer {
            abi,
            parameters,
            return_type,
            variadic,
        } => {
            hash_field(hasher, b"function-pointer");
            hash_field(hasher, abi.spelling().as_bytes());
            hash_field(hasher, &[u8::from(*variadic)]);
            hash_count(hasher, parameters.len());
            for parameter in parameters {
                hash_type(hasher, parameter);
            }
            hash_type(hasher, return_type);
        }
    }
}

fn hash_scalar(hasher: &mut blake3::Hasher, scalar: crate::RustScalar) {
    hash_field(hasher, scalar.spelling().as_bytes());
    hash_field(hasher, &scalar.size_bits().to_le_bytes());
    hash_field(
        hasher,
        &scalar.alignment_bits().unwrap_or(u16::MAX).to_le_bytes(),
    );
}

fn hash_exact_integer(hasher: &mut blake3::Hasher, value: ExactInteger) {
    match value {
        ExactInteger::Signed { value } => {
            hash_field(hasher, b"signed");
            hash_field(hasher, &value.to_le_bytes());
        }
        ExactInteger::Unsigned { value } => {
            hash_field(hasher, b"unsigned");
            hash_field(hasher, &value.to_le_bytes());
        }
    }
}

fn hash_declaration_metadata(
    hasher: &mut blake3::Hasher,
    metadata: &crate::SourceDeclarationMetadata,
) {
    match metadata.identity() {
        DeclarationIdentity::Named {
            namespace,
            scope,
            normalized_name,
        } => {
            hash_field(hasher, b"identity-named");
            hash_entity_namespace(hasher, *namespace);
            hash_entity_scope(hasher, *scope);
            hash_field(hasher, normalized_name.as_bytes());
        }
        DeclarationIdentity::Anonymous {
            scope,
            token_fingerprint,
            duplicate_ordinal,
        } => {
            hash_field(hasher, b"identity-anonymous");
            hash_entity_scope(hasher, *scope);
            hash_field(hasher, token_fingerprint.as_bytes());
            hash_field(hasher, &duplicate_ordinal.to_le_bytes());
        }
    }
    hash_optional_source_name(hasher, metadata.name());
    hash_field(
        hasher,
        match metadata.linkage() {
            Linkage::External => b"linkage-external",
            Linkage::Internal => b"linkage-internal",
            Linkage::None => b"linkage-none",
        },
    );
    hash_field(
        hasher,
        match metadata.visibility() {
            Visibility::Unspecified => b"visibility-unspecified",
            Visibility::ExplicitDefault => b"visibility-explicit-default",
            Visibility::TargetDefault => b"visibility-target-default",
            Visibility::Hidden => b"visibility-hidden",
            Visibility::Protected => b"visibility-protected",
            Visibility::Internal => b"visibility-internal",
        },
    );
    hash_support(hasher, metadata.support());
    hash_count(hasher, metadata.occurrences().len());
    for occurrence in metadata.occurrences() {
        hash_field(hasher, occurrence.id.as_bytes());
        hash_range(hasher, occurrence.range);
        match occurrence.name_range {
            Some(range) => {
                hash_field(hasher, b"name-range");
                hash_range(hasher, range);
            }
            None => hash_field(hasher, b"no-name-range"),
        }
        hash_field(hasher, occurrence.spelling.as_bytes());
        hash_strings(hasher, &occurrence.normalized_tokens);
        hash_field(hasher, &occurrence.duplicate_ordinal.to_le_bytes());
        hash_storage_class(hasher, occurrence.storage);
        hash_field(hasher, &[u8::from(occurrence.is_definition)]);
        hash_attributes(hasher, &occurrence.attributes);
        hash_provenance(hasher, &occurrence.provenance);
    }
}

fn hash_entity_namespace(hasher: &mut blake3::Hasher, namespace: EntityNamespace) {
    hash_field(
        hasher,
        match namespace {
            EntityNamespace::Ordinary => b"namespace-ordinary",
            EntityNamespace::Tag => b"namespace-tag",
        },
    );
}

fn hash_entity_scope(hasher: &mut blake3::Hasher, scope: EntityScope) {
    match scope {
        EntityScope::TranslationUnit => hash_field(hasher, b"scope-translation-unit"),
        EntityScope::File(file) => {
            hash_field(hasher, b"scope-file");
            hash_field(hasher, file.as_bytes());
        }
        EntityScope::Owner(declaration) => {
            hash_field(hasher, b"scope-owner");
            hash_field(hasher, declaration.as_bytes());
        }
    }
}

fn hash_source_name(hasher: &mut blake3::Hasher, name: &SourceName) {
    hash_field(hasher, name.normalized.as_bytes());
    hash_field(hasher, name.original.as_bytes());
}

fn hash_optional_source_name(hasher: &mut blake3::Hasher, name: Option<&SourceName>) {
    match name {
        Some(name) => {
            hash_field(hasher, b"source-name");
            hash_source_name(hasher, name);
        }
        None => hash_field(hasher, b"no-source-name"),
    }
}

fn hash_support(hasher: &mut blake3::Hasher, support: &SupportStatus) {
    match support {
        SupportStatus::Supported => hash_field(hasher, b"support-supported"),
        SupportStatus::Partial { code, reason } => {
            hash_field(hasher, b"support-partial");
            hash_field(hasher, code.as_str().as_bytes());
            hash_field(hasher, reason.as_bytes());
        }
        SupportStatus::Unsupported { code, reason } => {
            hash_field(hasher, b"support-unsupported");
            hash_field(hasher, code.as_str().as_bytes());
            hash_field(hasher, reason.as_bytes());
        }
    }
}

fn hash_storage_class(hasher: &mut blake3::Hasher, storage: StorageClass) {
    hash_field(
        hasher,
        match storage {
            StorageClass::None => b"storage-none",
            StorageClass::Typedef => b"storage-typedef",
            StorageClass::Extern => b"storage-extern",
            StorageClass::Static => b"storage-static",
            StorageClass::ThreadLocal => b"storage-thread-local",
            StorageClass::Auto => b"storage-auto",
            StorageClass::Register => b"storage-register",
        },
    );
}

fn hash_attributes(hasher: &mut blake3::Hasher, attributes: &[SourceAttribute]) {
    hash_count(hasher, attributes.len());
    for attribute in attributes {
        match &attribute.namespace {
            Some(namespace) => {
                hash_field(hasher, b"attribute-namespace");
                hash_field(hasher, namespace.as_bytes());
            }
            None => hash_field(hasher, b"no-attribute-namespace"),
        }
        hash_field(hasher, attribute.name.as_bytes());
        hash_strings(hasher, &attribute.arguments);
        hash_field(hasher, attribute.spelling.as_bytes());
        hash_range(hasher, attribute.range);
        hash_field(
            hasher,
            match attribute.disposition {
                AttributeDisposition::Modeled => b"attribute-modeled",
                AttributeDisposition::Preserved => b"attribute-preserved",
                AttributeDisposition::UnsupportedAbiRelevant => b"attribute-unsupported-abi",
            },
        );
    }
}

fn hash_range(hasher: &mut blake3::Hasher, range: SourceRange) {
    hash_field(hasher, range.file.as_bytes());
    hash_field(hasher, &range.start.to_le_bytes());
    hash_field(hasher, &range.end.to_le_bytes());
}

fn hash_provenance(hasher: &mut blake3::Hasher, provenance: &SourceProvenance) {
    hash_field(
        hasher,
        match provenance.origin {
            SourceOrigin::Entry => b"origin-entry",
            SourceOrigin::UserInclude => b"origin-user-include",
            SourceOrigin::SystemInclude => b"origin-system-include",
            SourceOrigin::Builtin => b"origin-builtin",
            SourceOrigin::Generated => b"origin-generated",
        },
    );
    hash_count(hasher, provenance.include_chain.len());
    for include in &provenance.include_chain {
        hash_range(hasher, include.directive);
        hash_field(hasher, include.included.as_bytes());
    }
    hash_count(hasher, provenance.macro_expansions.len());
    for expansion in &provenance.macro_expansions {
        hash_field(hasher, expansion.macro_name.as_bytes());
        hash_range(hasher, expansion.invocation);
        match expansion.definition {
            Some(range) => {
                hash_field(hasher, b"macro-definition");
                hash_range(hasher, range);
            }
            None => hash_field(hasher, b"no-macro-definition"),
        }
    }
}

fn hash_macro_form(hasher: &mut blake3::Hasher, form: MacroForm) {
    hash_field(
        hasher,
        match form {
            MacroForm::ObjectLike => b"macro-object-like",
            MacroForm::FunctionLike => b"macro-function-like",
        },
    );
}

fn hash_macro_category(hasher: &mut blake3::Hasher, category: MacroCategory) {
    hash_field(
        hasher,
        match category {
            MacroCategory::BindableConstant => b"macro-bindable-constant",
            MacroCategory::ConfigurationFlag => b"macro-configuration-flag",
            MacroCategory::AbiAffecting => b"macro-abi-affecting",
            MacroCategory::Unsupported => b"macro-unsupported",
        },
    );
}

fn hash_macro_value(hasher: &mut blake3::Hasher, value: Option<&MacroValue>) {
    match value {
        Some(MacroValue::Integer { value }) => {
            hash_field(hasher, b"macro-value-integer");
            hash_exact_integer(hasher, *value);
        }
        Some(MacroValue::String { value }) => {
            hash_field(hasher, b"macro-value-string");
            hash_field(hasher, value.as_bytes());
        }
        Some(MacroValue::Tokens { tokens }) => {
            hash_field(hasher, b"macro-value-tokens");
            hash_strings(hasher, tokens);
        }
        None => hash_field(hasher, b"no-macro-value"),
    }
}

fn hash_strings(hasher: &mut blake3::Hasher, values: &[String]) {
    hash_count(hasher, values.len());
    for value in values {
        hash_field(hasher, value.as_bytes());
    }
}

fn hash_optional_u32(hasher: &mut blake3::Hasher, value: Option<u32>) {
    match value {
        Some(value) => {
            hash_field(hasher, b"some-u32");
            hash_field(hasher, &value.to_le_bytes());
        }
        None => hash_field(hasher, b"no-u32"),
    }
}

fn hash_count(hasher: &mut blake3::Hasher, count: usize) {
    hash_field(hasher, &(count as u64).to_le_bytes());
}

fn hash_field(hasher: &mut blake3::Hasher, field: &[u8]) {
    hasher.update(&(field.len() as u64).to_le_bytes());
    hasher.update(field);
}

fn hash_path(hasher: &mut blake3::Hasher, path: &Path) {
    hash_native(hasher, path.as_os_str());
}

#[cfg(unix)]
fn hash_native(hasher: &mut blake3::Hasher, value: &OsStr) {
    use std::os::unix::ffi::OsStrExt;
    hash_field(hasher, b"unix-bytes");
    hash_field(hasher, value.as_bytes());
}

#[cfg(windows)]
fn hash_native(hasher: &mut blake3::Hasher, value: &OsStr) {
    use std::os::windows::ffi::OsStrExt;
    hash_field(hasher, b"windows-utf16");
    let units: Vec<_> = value.encode_wide().flat_map(u16::to_le_bytes).collect();
    hash_field(hasher, &units);
}
