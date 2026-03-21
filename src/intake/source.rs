use serde::{Deserialize, Serialize};

use crate::c::{
    self as ir, BindingDefine, BindingInputs, BindingPackage, BindingTarget, CallingConvention,
    LinkFramework, LinkInput, LinkLibrary, LinkLibraryKind, LinkRequirementSource, MacroBinding,
    MacroCategory, MacroForm, MacroKind, RecordKind, TypeQualifiers,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SourcePackage {
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub declarations: Vec<SourceDeclaration>,
    #[serde(default)]
    pub macros: Vec<SourceMacro>,
    #[serde(default)]
    pub link_requirements: Vec<SourceLinkRequirement>,
    #[serde(default)]
    pub include_dirs: Vec<String>,
    #[serde(default)]
    pub entry_headers: Vec<String>,
    #[serde(default)]
    pub defines: Vec<(String, Option<String>)>,
    #[serde(default)]
    pub target_triple: Option<String>,
    #[serde(default)]
    pub compiler_command: Option<String>,
    #[serde(default)]
    pub compiler_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceDeclaration {
    Function(SourceFunction),
    Record(SourceRecord),
    Enum(SourceEnum),
    TypeAlias(SourceTypeAlias),
    Variable(SourceVariable),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceFunction {
    pub name: String,
    #[serde(default)]
    pub parameters: Vec<SourceParameter>,
    pub return_type: SourceType,
    #[serde(default)]
    pub variadic: bool,
    #[serde(default)]
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceParameter {
    pub name: Option<String>,
    pub ty: SourceType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub name: Option<String>,
    #[serde(default)]
    pub is_union: bool,
    #[serde(default)]
    pub fields: Option<Vec<SourceField>>,
    #[serde(default)]
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceField {
    pub name: Option<String>,
    pub ty: SourceType,
    #[serde(default)]
    pub bit_width: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceEnum {
    pub name: Option<String>,
    #[serde(default)]
    pub variants: Vec<SourceEnumVariant>,
    #[serde(default)]
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceEnumVariant {
    pub name: String,
    pub value: Option<i128>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceTypeAlias {
    pub name: String,
    pub target: SourceType,
    #[serde(default)]
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceVariable {
    pub name: String,
    pub ty: SourceType,
    #[serde(default)]
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMacro {
    pub name: String,
    pub body: String,
    #[serde(default)]
    pub function_like: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLinkRequirement {
    pub name: String,
    #[serde(default)]
    pub kind: SourceLinkKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SourceLinkKind {
    #[default]
    Library,
    StaticLibrary,
    DynamicLibrary,
    Framework,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Void,
    Bool,
    Char,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    Float,
    Double,
    LongDouble,
    Pointer(Box<SourceType>),
    ConstPointer(Box<SourceType>),
    Array(Box<SourceType>, Option<u64>),
    FunctionPointer {
        return_type: Box<SourceType>,
        parameters: Vec<SourceType>,
        variadic: bool,
    },
    TypedefRef(String),
    RecordRef(String),
    EnumRef(String),
    Opaque(String),
    Const(Box<SourceType>),
    Volatile(Box<SourceType>),
}

pub(super) fn binding_package_from_source(source: &SourcePackage) -> BindingPackage {
    let mut package = BindingPackage::new();
    package.source_path = source.source_path.clone();
    package.inputs = BindingInputs {
        entry_headers: source.entry_headers.clone(),
        include_dirs: source.include_dirs.clone(),
        defines: source
            .defines
            .iter()
            .map(|(name, value)| BindingDefine {
                name: name.clone(),
                value: value.clone(),
            })
            .collect(),
    };
    package.target = BindingTarget {
        target_triple: source.target_triple.clone(),
        compiler_command: source.compiler_command.clone(),
        compiler_version: source.compiler_version.clone(),
        flavor: None,
    };
    package.items = source
        .declarations
        .iter()
        .map(declaration_to_binding)
        .collect();
    package.macros = source
        .macros
        .iter()
        .map(|m| MacroBinding {
            name: m.name.clone(),
            body: m.body.clone(),
            function_like: m.function_like,
            form: if m.function_like {
                MacroForm::FunctionLike
            } else {
                MacroForm::ObjectLike
            },
            kind: MacroKind::Other,
            category: MacroCategory::Unsupported,
            value: None,
        })
        .collect();

    for requirement in &source.link_requirements {
        match requirement.kind {
            SourceLinkKind::Library => {
                package.link.libraries.push(LinkLibrary {
                    name: requirement.name.clone(),
                    kind: LinkLibraryKind::Default,
                    source: LinkRequirementSource::Declared,
                });
                package
                    .link
                    .ordered_inputs
                    .push(LinkInput::Library(LinkLibrary {
                        name: requirement.name.clone(),
                        kind: LinkLibraryKind::Default,
                        source: LinkRequirementSource::Declared,
                    }));
            }
            SourceLinkKind::StaticLibrary => {
                package.link.libraries.push(LinkLibrary {
                    name: requirement.name.clone(),
                    kind: LinkLibraryKind::Static,
                    source: LinkRequirementSource::Declared,
                });
                package
                    .link
                    .ordered_inputs
                    .push(LinkInput::Library(LinkLibrary {
                        name: requirement.name.clone(),
                        kind: LinkLibraryKind::Static,
                        source: LinkRequirementSource::Declared,
                    }));
            }
            SourceLinkKind::DynamicLibrary => {
                package.link.libraries.push(LinkLibrary {
                    name: requirement.name.clone(),
                    kind: LinkLibraryKind::Dynamic,
                    source: LinkRequirementSource::Declared,
                });
                package
                    .link
                    .ordered_inputs
                    .push(LinkInput::Library(LinkLibrary {
                        name: requirement.name.clone(),
                        kind: LinkLibraryKind::Dynamic,
                        source: LinkRequirementSource::Declared,
                    }));
            }
            SourceLinkKind::Framework => {
                package.link.frameworks.push(LinkFramework {
                    name: requirement.name.clone(),
                    source: LinkRequirementSource::Declared,
                });
                package
                    .link
                    .ordered_inputs
                    .push(LinkInput::Framework(LinkFramework {
                        name: requirement.name.clone(),
                        source: LinkRequirementSource::Declared,
                    }));
            }
        }
    }

    package
}

pub(super) fn source_package_from_json(json: &str) -> Result<SourcePackage, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

pub(crate) fn source_package_from_binding(package: &BindingPackage) -> SourcePackage {
    let declarations = package
        .items
        .iter()
        .filter_map(binding_to_declaration)
        .collect();
    let macros = package
        .macros
        .iter()
        .map(|macro_binding| SourceMacro {
            name: macro_binding.name.clone(),
            body: macro_binding.body.clone(),
            function_like: macro_binding.function_like,
        })
        .collect();
    let mut link_requirements = Vec::new();
    for library in &package.link.libraries {
        let kind = match library.kind {
            LinkLibraryKind::Default => SourceLinkKind::Library,
            LinkLibraryKind::Static => SourceLinkKind::StaticLibrary,
            LinkLibraryKind::Dynamic => SourceLinkKind::DynamicLibrary,
        };
        link_requirements.push(SourceLinkRequirement {
            name: library.name.clone(),
            kind,
        });
    }
    for framework in &package.link.frameworks {
        link_requirements.push(SourceLinkRequirement {
            name: framework.name.clone(),
            kind: SourceLinkKind::Framework,
        });
    }

    SourcePackage {
        source_path: package.source_path.clone(),
        declarations,
        macros,
        link_requirements,
        include_dirs: package.inputs.include_dirs.clone(),
        entry_headers: package.inputs.entry_headers.clone(),
        defines: package
            .inputs
            .defines
            .iter()
            .map(|define| (define.name.clone(), define.value.clone()))
            .collect(),
        target_triple: package.target.target_triple.clone(),
        compiler_command: package.target.compiler_command.clone(),
        compiler_version: package.target.compiler_version.clone(),
    }
}

fn declaration_to_binding(declaration: &SourceDeclaration) -> ir::BindingItem {
    match declaration {
        SourceDeclaration::Function(function) => ir::BindingItem::Function(ir::FunctionBinding {
            name: function.name.clone(),
            calling_convention: CallingConvention::C,
            parameters: function
                .parameters
                .iter()
                .map(|parameter| ir::ParameterBinding {
                    name: parameter.name.clone(),
                    ty: source_type_to_binding(&parameter.ty),
                })
                .collect(),
            return_type: source_type_to_binding(&function.return_type),
            variadic: function.variadic,
            source_offset: function.source_offset,
        }),
        SourceDeclaration::Record(record) => ir::BindingItem::Record(ir::RecordBinding {
            kind: if record.is_union {
                RecordKind::Union
            } else {
                RecordKind::Struct
            },
            name: record.name.clone(),
            fields: record.fields.as_ref().map(|fields| {
                fields
                    .iter()
                    .map(|field| ir::FieldBinding {
                        name: field.name.clone(),
                        ty: source_type_to_binding(&field.ty),
                        bit_width: field.bit_width,
                        layout: None,
                    })
                    .collect()
            }),
            representation: None,
            abi_confidence: None,
            source_offset: record.source_offset,
        }),
        SourceDeclaration::Enum(enumeration) => ir::BindingItem::Enum(ir::EnumBinding {
            name: enumeration.name.clone(),
            variants: enumeration
                .variants
                .iter()
                .map(|variant| ir::EnumVariant {
                    name: variant.name.clone(),
                    value: variant.value,
                })
                .collect(),
            representation: None,
            abi_confidence: None,
            source_offset: enumeration.source_offset,
        }),
        SourceDeclaration::TypeAlias(alias) => {
            ir::BindingItem::TypeAlias(ir::TypeAliasBinding {
                name: alias.name.clone(),
                target: source_type_to_binding(&alias.target),
                canonical_resolution: None,
                abi_confidence: None,
                source_offset: alias.source_offset,
            })
        }
        SourceDeclaration::Variable(variable) => ir::BindingItem::Variable(ir::VariableBinding {
            name: variable.name.clone(),
            ty: source_type_to_binding(&variable.ty),
            source_offset: variable.source_offset,
        }),
    }
}

fn binding_to_declaration(item: &ir::BindingItem) -> Option<SourceDeclaration> {
    match item {
        ir::BindingItem::Function(function) => {
            Some(SourceDeclaration::Function(SourceFunction {
                name: function.name.clone(),
                parameters: function
                    .parameters
                    .iter()
                    .map(|parameter| SourceParameter {
                        name: parameter.name.clone(),
                        ty: binding_type_to_source(&parameter.ty),
                    })
                    .collect(),
                return_type: binding_type_to_source(&function.return_type),
                variadic: function.variadic,
                source_offset: function.source_offset,
            }))
        }
        ir::BindingItem::Record(record) => Some(SourceDeclaration::Record(SourceRecord {
            name: record.name.clone(),
            is_union: record.kind == RecordKind::Union,
            fields: record.fields.as_ref().map(|fields| {
                fields
                    .iter()
                    .map(|field| SourceField {
                        name: field.name.clone(),
                        ty: binding_type_to_source(&field.ty),
                        bit_width: field.bit_width,
                    })
                    .collect()
            }),
            source_offset: record.source_offset,
        })),
        ir::BindingItem::Enum(enumeration) => Some(SourceDeclaration::Enum(SourceEnum {
            name: enumeration.name.clone(),
            variants: enumeration
                .variants
                .iter()
                .map(|variant| SourceEnumVariant {
                    name: variant.name.clone(),
                    value: variant.value,
                })
                .collect(),
            source_offset: enumeration.source_offset,
        })),
        ir::BindingItem::TypeAlias(alias) => Some(SourceDeclaration::TypeAlias(SourceTypeAlias {
            name: alias.name.clone(),
            target: binding_type_to_source(&alias.target),
            source_offset: alias.source_offset,
        })),
        ir::BindingItem::Variable(variable) => Some(SourceDeclaration::Variable(SourceVariable {
            name: variable.name.clone(),
            ty: binding_type_to_source(&variable.ty),
            source_offset: variable.source_offset,
        })),
        ir::BindingItem::Unsupported(_) => None,
    }
}

fn source_type_to_binding(ty: &SourceType) -> ir::BindingType {
    match ty {
        SourceType::Void => ir::BindingType::Void,
        SourceType::Bool => ir::BindingType::Bool,
        SourceType::Char => ir::BindingType::Char,
        SourceType::SChar => ir::BindingType::SChar,
        SourceType::UChar => ir::BindingType::UChar,
        SourceType::Short => ir::BindingType::Short,
        SourceType::UShort => ir::BindingType::UShort,
        SourceType::Int => ir::BindingType::Int,
        SourceType::UInt => ir::BindingType::UInt,
        SourceType::Long => ir::BindingType::Long,
        SourceType::ULong => ir::BindingType::ULong,
        SourceType::LongLong => ir::BindingType::LongLong,
        SourceType::ULongLong => ir::BindingType::ULongLong,
        SourceType::Float => ir::BindingType::Float,
        SourceType::Double => ir::BindingType::Double,
        SourceType::LongDouble => ir::BindingType::LongDouble,
        SourceType::Pointer(inner) => ir::BindingType::ptr(source_type_to_binding(inner)),
        SourceType::ConstPointer(inner) => {
            ir::BindingType::const_ptr(source_type_to_binding(inner))
        }
        SourceType::Array(element, len) => {
            ir::BindingType::Array(Box::new(source_type_to_binding(element)), *len)
        }
        SourceType::FunctionPointer {
            return_type,
            parameters,
            variadic,
        } => ir::BindingType::FunctionPointer {
            return_type: Box::new(source_type_to_binding(return_type)),
            parameters: parameters.iter().map(source_type_to_binding).collect(),
            variadic: *variadic,
        },
        SourceType::TypedefRef(name) => ir::BindingType::TypedefRef(name.clone()),
        SourceType::RecordRef(name) => ir::BindingType::RecordRef(name.clone()),
        SourceType::EnumRef(name) => ir::BindingType::EnumRef(name.clone()),
        SourceType::Opaque(name) => ir::BindingType::Opaque(name.clone()),
        SourceType::Const(inner) => ir::BindingType::qualified(
            source_type_to_binding(inner),
            TypeQualifiers {
                is_const: true,
                ..Default::default()
            },
        ),
        SourceType::Volatile(inner) => ir::BindingType::qualified(
            source_type_to_binding(inner),
            TypeQualifiers {
                is_volatile: true,
                ..Default::default()
            },
        ),
    }
}

fn binding_type_to_source(ty: &ir::BindingType) -> SourceType {
    match ty {
        ir::BindingType::Void => SourceType::Void,
        ir::BindingType::Bool => SourceType::Bool,
        ir::BindingType::Char => SourceType::Char,
        ir::BindingType::SChar => SourceType::SChar,
        ir::BindingType::UChar => SourceType::UChar,
        ir::BindingType::Short => SourceType::Short,
        ir::BindingType::UShort => SourceType::UShort,
        ir::BindingType::Int => SourceType::Int,
        ir::BindingType::UInt => SourceType::UInt,
        ir::BindingType::Long => SourceType::Long,
        ir::BindingType::ULong => SourceType::ULong,
        ir::BindingType::LongLong => SourceType::LongLong,
        ir::BindingType::ULongLong => SourceType::ULongLong,
        ir::BindingType::Float => SourceType::Float,
        ir::BindingType::Double => SourceType::Double,
        ir::BindingType::LongDouble => SourceType::LongDouble,
        ir::BindingType::Pointer {
            pointee,
            const_pointee,
            ..
        } => {
            if *const_pointee {
                SourceType::ConstPointer(Box::new(binding_type_to_source(pointee)))
            } else {
                SourceType::Pointer(Box::new(binding_type_to_source(pointee)))
            }
        }
        ir::BindingType::Array(element, len) => {
            SourceType::Array(Box::new(binding_type_to_source(element)), *len)
        }
        ir::BindingType::FunctionPointer {
            return_type,
            parameters,
            variadic,
        } => SourceType::FunctionPointer {
            return_type: Box::new(binding_type_to_source(return_type)),
            parameters: parameters.iter().map(binding_type_to_source).collect(),
            variadic: *variadic,
        },
        ir::BindingType::TypedefRef(name) => SourceType::TypedefRef(name.clone()),
        ir::BindingType::RecordRef(name) => SourceType::RecordRef(name.clone()),
        ir::BindingType::EnumRef(name) => SourceType::EnumRef(name.clone()),
        ir::BindingType::Opaque(name) => SourceType::Opaque(name.clone()),
        ir::BindingType::Qualified { ty, qualifiers } => {
            let inner = binding_type_to_source(ty);
            if qualifiers.is_const {
                SourceType::Const(Box::new(inner))
            } else if qualifiers.is_volatile {
                SourceType::Volatile(Box::new(inner))
            } else {
                inner
            }
        }
    }
}
