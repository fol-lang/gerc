use std::{collections::BTreeSet, path::PathBuf};

use linc::contract::{ArtifactFingerprint, ArtifactSymbolId, ProviderId, SymbolDecoration};
use parc::contract::{
    ChildId, DeclarationId, DeclarationIdentity, DeclarationOccurrence, ExactInteger, FileId,
    Linkage, MacroCategory, MacroForm, MacroId, MacroOccurrence, MacroValue, Nullability,
    RecordCompleteness, RecordKind, SourceAttribute, SourceDeclaration, SourceName,
    SourceProvenance, SourceRange, SupportStatus, TargetFingerprint, TypeQualifiers, Visibility,
};

use crate::{GenerationError, GenerationResult};

/// A Rust projection that can only be constructed by the checked generator.
/// It deliberately implements no deserialization or unchecked constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedRustProjection {
    target_fingerprint: TargetFingerprint,
    items: Vec<RustItem>,
    macros: Vec<RustMacro>,
}

impl ValidatedRustProjection {
    pub(crate) fn try_new(
        target_fingerprint: TargetFingerprint,
        items: Vec<RustItem>,
        macros: Vec<RustMacro>,
        expected_declarations: &[DeclarationId],
    ) -> GenerationResult<Self> {
        let actual: Vec<_> = items.iter().map(RustItem::declaration).collect();
        if actual != expected_declarations {
            return Err(GenerationError::ProjectionInvariant {
                reason: "projected declarations differ from the complete source closure",
            });
        }

        let mut declarations = BTreeSet::new();
        let mut emitted_names = BTreeSet::new();
        for item in &items {
            if !declarations.insert(item.declaration()) {
                return Err(GenerationError::ProjectionInvariant {
                    reason: "projection repeats a DeclarationId",
                });
            }
            for name in item.emitted_names() {
                if !emitted_names.insert(name) {
                    return Err(GenerationError::ProjectionInvariant {
                        reason: "two projection items emit the same Rust identifier",
                    });
                }
            }
        }
        for source_macro in &macros {
            if source_macro.emitted()
                && !emitted_names.insert(source_macro.rust_name().as_str().to_owned())
            {
                return Err(GenerationError::ProjectionInvariant {
                    reason: "a macro and declaration emit the same Rust identifier",
                });
            }
        }

        Ok(Self {
            target_fingerprint,
            items,
            macros,
        })
    }

    pub const fn target_fingerprint(&self) -> TargetFingerprint {
        self.target_fingerprint
    }

    pub fn items(&self) -> &[RustItem] {
        &self.items
    }

    pub fn macros(&self) -> &[RustMacro] {
        &self.macros
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustName(String);

impl RustName {
    pub(crate) fn checked(value: String) -> Option<Self> {
        (!value.is_empty() && !value.bytes().any(|byte| byte == 0)).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustItem {
    Function(RustFunction),
    Record(RustRecord),
    Enum(RustEnum),
    TypeAlias(RustTypeAlias),
    Variable(RustVariable),
}

/// The declaration-level PARC facts retained independently from the emitted
/// Rust identifier. This prevents sanitization from erasing source identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDeclarationMetadata {
    identity: DeclarationIdentity,
    name: Option<SourceName>,
    linkage: Linkage,
    visibility: Visibility,
    support: SupportStatus,
    occurrences: Vec<DeclarationOccurrence>,
}

impl SourceDeclarationMetadata {
    pub(crate) fn from_source(declaration: &SourceDeclaration) -> Self {
        Self {
            identity: declaration.identity.clone(),
            name: declaration.name.clone(),
            linkage: declaration.linkage,
            visibility: declaration.visibility,
            support: declaration.support.clone(),
            occurrences: declaration.occurrences.clone(),
        }
    }

    pub fn identity(&self) -> &DeclarationIdentity {
        &self.identity
    }
    pub fn name(&self) -> Option<&SourceName> {
        self.name.as_ref()
    }
    pub const fn linkage(&self) -> Linkage {
        self.linkage
    }
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        &self.occurrences
    }
}

impl RustItem {
    pub const fn declaration(&self) -> DeclarationId {
        match self {
            Self::Function(item) => item.declaration,
            Self::Record(item) => item.declaration,
            Self::Enum(item) => item.declaration,
            Self::TypeAlias(item) => item.declaration,
            Self::Variable(item) => item.declaration,
        }
    }

    pub fn rust_name(&self) -> &RustName {
        match self {
            Self::Function(item) => &item.rust_name,
            Self::Record(item) => &item.rust_name,
            Self::Enum(item) => &item.rust_name,
            Self::TypeAlias(item) => &item.rust_name,
            Self::Variable(item) => &item.rust_name,
        }
    }

    fn emitted_names(&self) -> Vec<String> {
        let mut names = vec![self.rust_name().as_str().to_owned()];
        if let Self::Enum(item) = self {
            names.extend(
                item.variants
                    .iter()
                    .map(|variant| variant.rust_name.as_str().to_owned()),
            );
        }
        names
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustAbi {
    C,
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall,
    SysV64,
    Win64,
    Aapcs,
}

impl RustAbi {
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cdecl => "cdecl",
            Self::Stdcall => "stdcall",
            Self::Fastcall => "fastcall",
            Self::Thiscall => "thiscall",
            Self::SysV64 => "sysv64",
            Self::Win64 => "win64",
            Self::Aapcs => "aapcs",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustFunction {
    pub(crate) declaration: DeclarationId,
    pub(crate) rust_name: RustName,
    pub(crate) link_name: String,
    pub(crate) abi: RustAbi,
    pub(crate) parameters: Vec<RustParameter>,
    pub(crate) return_type: RustType,
    pub(crate) variadic: bool,
    pub(crate) symbol: NativeSymbolBinding,
    pub(crate) source: SourceDeclarationMetadata,
}

impl RustFunction {
    pub const fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn link_name(&self) -> &str {
        &self.link_name
    }
    pub const fn abi(&self) -> RustAbi {
        self.abi
    }
    pub fn parameters(&self) -> &[RustParameter] {
        &self.parameters
    }
    pub fn return_type(&self) -> &RustType {
        &self.return_type
    }
    pub const fn variadic(&self) -> bool {
        self.variadic
    }
    pub fn symbol(&self) -> &NativeSymbolBinding {
        &self.symbol
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        self.source.occurrences()
    }
    pub fn source(&self) -> &SourceDeclarationMetadata {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustParameter {
    pub(crate) child: ChildId,
    pub(crate) ordinal: u64,
    pub(crate) rust_name: RustName,
    pub(crate) source_name: Option<SourceName>,
    pub(crate) ty: RustType,
    pub(crate) range: SourceRange,
    pub(crate) provenance: SourceProvenance,
    pub(crate) attributes: Vec<SourceAttribute>,
    pub(crate) support: SupportStatus,
}

impl RustParameter {
    pub const fn child(&self) -> ChildId {
        self.child
    }
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn source_name(&self) -> Option<&SourceName> {
        self.source_name.as_ref()
    }
    pub fn ty(&self) -> &RustType {
        &self.ty
    }
    pub const fn range(&self) -> SourceRange {
        self.range
    }
    pub fn provenance(&self) -> &SourceProvenance {
        &self.provenance
    }
    pub fn attributes(&self) -> &[SourceAttribute] {
        &self.attributes
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustRecordKind {
    Struct,
    Union,
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustRecord {
    pub(crate) declaration: DeclarationId,
    pub(crate) rust_name: RustName,
    pub(crate) kind: RustRecordKind,
    pub(crate) source_kind: RecordKind,
    pub(crate) source_completeness: RecordCompleteness,
    pub(crate) fields: Vec<RustField>,
    pub(crate) size_bits: Option<u64>,
    pub(crate) alignment_bits: Option<u32>,
    /// Measured packing cap in bits when the layout cannot be represented by
    /// natural `repr(C)` alignment. `None` means natural layout.
    pub(crate) packing_bits: Option<u32>,
    pub(crate) source: SourceDeclarationMetadata,
}

impl RustRecord {
    pub const fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub const fn kind(&self) -> RustRecordKind {
        self.kind
    }
    pub const fn source_kind(&self) -> RecordKind {
        self.source_kind
    }
    pub const fn source_completeness(&self) -> RecordCompleteness {
        self.source_completeness
    }
    pub fn fields(&self) -> &[RustField] {
        &self.fields
    }
    pub const fn size_bits(&self) -> Option<u64> {
        self.size_bits
    }
    pub const fn alignment_bits(&self) -> Option<u32> {
        self.alignment_bits
    }
    /// The exact `repr(packed(N))` cap required by the measured C layout, in
    /// bits. This is absent for natural `repr(C)` records and opaque records.
    pub const fn packing_bits(&self) -> Option<u32> {
        self.packing_bits
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        self.source.occurrences()
    }
    pub fn source(&self) -> &SourceDeclarationMetadata {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustField {
    pub(crate) child: ChildId,
    pub(crate) rust_name: RustName,
    pub(crate) source_name: Option<SourceName>,
    pub(crate) ty: RustType,
    pub(crate) offset_bits: u64,
    pub(crate) size_bits: u64,
    pub(crate) alignment_bits: Option<u32>,
    pub(crate) range: SourceRange,
    pub(crate) provenance: SourceProvenance,
    pub(crate) attributes: Vec<SourceAttribute>,
    pub(crate) support: SupportStatus,
    pub(crate) identity_tokens: Vec<String>,
    pub(crate) duplicate_ordinal: u64,
}

impl RustField {
    pub const fn child(&self) -> ChildId {
        self.child
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn source_name(&self) -> Option<&SourceName> {
        self.source_name.as_ref()
    }
    pub fn ty(&self) -> &RustType {
        &self.ty
    }
    pub const fn offset_bits(&self) -> u64 {
        self.offset_bits
    }
    pub const fn size_bits(&self) -> u64 {
        self.size_bits
    }
    pub const fn alignment_bits(&self) -> Option<u32> {
        self.alignment_bits
    }
    pub const fn range(&self) -> SourceRange {
        self.range
    }
    pub fn provenance(&self) -> &SourceProvenance {
        &self.provenance
    }
    pub fn attributes(&self) -> &[SourceAttribute] {
        &self.attributes
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
    pub fn identity_tokens(&self) -> &[String] {
        &self.identity_tokens
    }
    pub const fn duplicate_ordinal(&self) -> u64 {
        self.duplicate_ordinal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustEnum {
    pub(crate) declaration: DeclarationId,
    pub(crate) rust_name: RustName,
    pub(crate) storage: RustScalar,
    pub(crate) alignment_bits: u32,
    pub(crate) explicit_underlying_type: Option<RustType>,
    pub(crate) variants: Vec<RustEnumVariant>,
    pub(crate) source: SourceDeclarationMetadata,
}

impl RustEnum {
    pub const fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub const fn storage(&self) -> RustScalar {
        self.storage
    }
    pub const fn alignment_bits(&self) -> u32 {
        self.alignment_bits
    }
    pub fn explicit_underlying_type(&self) -> Option<&RustType> {
        self.explicit_underlying_type.as_ref()
    }
    pub fn variants(&self) -> &[RustEnumVariant] {
        &self.variants
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        self.source.occurrences()
    }
    pub fn source(&self) -> &SourceDeclarationMetadata {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustEnumVariant {
    pub(crate) child: ChildId,
    pub(crate) rust_name: RustName,
    pub(crate) source_name: SourceName,
    pub(crate) value: ExactInteger,
    pub(crate) range: SourceRange,
    pub(crate) provenance: SourceProvenance,
    pub(crate) attributes: Vec<SourceAttribute>,
    pub(crate) support: SupportStatus,
    pub(crate) identity_tokens: Vec<String>,
    pub(crate) duplicate_ordinal: u64,
}

impl RustEnumVariant {
    pub const fn child(&self) -> ChildId {
        self.child
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn source_name(&self) -> &SourceName {
        &self.source_name
    }
    pub const fn value(&self) -> ExactInteger {
        self.value
    }
    pub const fn range(&self) -> SourceRange {
        self.range
    }
    pub fn provenance(&self) -> &SourceProvenance {
        &self.provenance
    }
    pub fn attributes(&self) -> &[SourceAttribute] {
        &self.attributes
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
    pub fn identity_tokens(&self) -> &[String] {
        &self.identity_tokens
    }
    pub const fn duplicate_ordinal(&self) -> u64 {
        self.duplicate_ordinal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTypeAlias {
    pub(crate) declaration: DeclarationId,
    pub(crate) rust_name: RustName,
    pub(crate) target: RustType,
    pub(crate) source: SourceDeclarationMetadata,
}

impl RustTypeAlias {
    pub const fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn target(&self) -> &RustType {
        &self.target
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        self.source.occurrences()
    }
    pub fn source(&self) -> &SourceDeclarationMetadata {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustVariable {
    pub(crate) declaration: DeclarationId,
    pub(crate) rust_name: RustName,
    pub(crate) link_name: String,
    pub(crate) ty: RustType,
    pub(crate) mutability: RustVariableMutability,
    pub(crate) thread_local: bool,
    pub(crate) symbol: NativeSymbolBinding,
    pub(crate) source: SourceDeclarationMetadata,
}

impl RustVariable {
    pub const fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn link_name(&self) -> &str {
        &self.link_name
    }
    pub fn ty(&self) -> &RustType {
        &self.ty
    }
    pub const fn mutability(&self) -> RustVariableMutability {
        self.mutability
    }
    pub const fn thread_local(&self) -> bool {
        self.thread_local
    }
    pub fn symbol(&self) -> &NativeSymbolBinding {
        &self.symbol
    }
    pub fn occurrences(&self) -> &[DeclarationOccurrence] {
        self.source.occurrences()
    }
    pub fn source(&self) -> &SourceDeclarationMetadata {
        &self.source
    }
}

/// Whether a raw extern object may be mutated through its Rust declaration.
/// This is retained independently from the nested type so callers do not have
/// to reinterpret C declaration qualifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustVariableMutability {
    ReadOnly,
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSymbolBinding {
    pub(crate) provider: ProviderId,
    pub(crate) artifact_fingerprint: ArtifactFingerprint,
    pub(crate) artifact_path: PathBuf,
    pub(crate) symbol: ArtifactSymbolId,
    pub(crate) expected_name: String,
    pub(crate) actual_name: String,
    pub(crate) raw_name: Vec<u8>,
    pub(crate) decoration: SymbolDecoration,
}

impl NativeSymbolBinding {
    pub const fn provider(&self) -> ProviderId {
        self.provider
    }
    pub const fn artifact_fingerprint(&self) -> ArtifactFingerprint {
        self.artifact_fingerprint
    }
    pub fn artifact_path(&self) -> &std::path::Path {
        &self.artifact_path
    }
    pub const fn symbol(&self) -> ArtifactSymbolId {
        self.symbol
    }
    pub fn expected_name(&self) -> &str {
        &self.expected_name
    }
    pub fn actual_name(&self) -> &str {
        &self.actual_name
    }
    pub fn raw_name(&self) -> &[u8] {
        &self.raw_name
    }
    pub fn decoration(&self) -> &SymbolDecoration {
        &self.decoration
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustType {
    pub(crate) qualifiers: TypeQualifiers,
    pub(crate) nullability: Nullability,
    pub(crate) support: SupportStatus,
    pub(crate) kind: RustTypeKind,
}

impl RustType {
    pub const fn qualifiers(&self) -> TypeQualifiers {
        self.qualifiers
    }
    pub const fn nullability(&self) -> Nullability {
        self.nullability
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
    pub fn kind(&self) -> &RustTypeKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustTypeKind {
    Void,
    Scalar(RustScalar),
    Pointer(Box<RustType>),
    FixedArray {
        element: Box<RustType>,
        elements: u64,
    },
    /// A C flexible-array member. This is only valid as the final field of a
    /// verified concrete struct and is emitted as a zero-length Rust array.
    FlexibleArray {
        element: Box<RustType>,
    },
    Named {
        declaration: DeclarationId,
        rust_name: RustName,
    },
    FunctionPointer {
        abi: RustAbi,
        parameters: Vec<RustType>,
        return_type: Box<RustType>,
        variadic: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustScalar {
    Bool,
    CChar {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CSignedChar {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CUnsignedChar {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CShort {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CUnsignedShort {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CInt {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CUnsignedInt {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CLong {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CUnsignedLong {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CLongLong {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CUnsignedLongLong {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CFloat {
        storage_bits: u16,
        alignment_bits: u16,
    },
    CDouble {
        storage_bits: u16,
        alignment_bits: u16,
    },
    /// Fixed-width measured integer storage used only for raw C enums.
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl RustScalar {
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::CChar { .. } => "core::ffi::c_char",
            Self::CSignedChar { .. } => "core::ffi::c_schar",
            Self::CUnsignedChar { .. } => "core::ffi::c_uchar",
            Self::CShort { .. } => "core::ffi::c_short",
            Self::CUnsignedShort { .. } => "core::ffi::c_ushort",
            Self::CInt { .. } => "core::ffi::c_int",
            Self::CUnsignedInt { .. } => "core::ffi::c_uint",
            Self::CLong { .. } => "core::ffi::c_long",
            Self::CUnsignedLong { .. } => "core::ffi::c_ulong",
            Self::CLongLong { .. } => "core::ffi::c_longlong",
            Self::CUnsignedLongLong { .. } => "core::ffi::c_ulonglong",
            Self::CFloat { .. } => "core::ffi::c_float",
            Self::CDouble { .. } => "core::ffi::c_double",
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    pub const fn size_bits(self) -> u64 {
        match self {
            Self::Bool | Self::I8 | Self::U8 => 8,
            Self::CChar { storage_bits, .. }
            | Self::CSignedChar { storage_bits, .. }
            | Self::CUnsignedChar { storage_bits, .. }
            | Self::CShort { storage_bits, .. }
            | Self::CUnsignedShort { storage_bits, .. }
            | Self::CInt { storage_bits, .. }
            | Self::CUnsignedInt { storage_bits, .. }
            | Self::CLong { storage_bits, .. }
            | Self::CUnsignedLong { storage_bits, .. }
            | Self::CLongLong { storage_bits, .. }
            | Self::CUnsignedLongLong { storage_bits, .. }
            | Self::CFloat { storage_bits, .. }
            | Self::CDouble { storage_bits, .. } => storage_bits as u64,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 | Self::F32 => 32,
            Self::I64 | Self::U64 | Self::F64 => 64,
        }
    }

    pub const fn alignment_bits(self) -> Option<u16> {
        match self {
            Self::CChar { alignment_bits, .. }
            | Self::CSignedChar { alignment_bits, .. }
            | Self::CUnsignedChar { alignment_bits, .. }
            | Self::CShort { alignment_bits, .. }
            | Self::CUnsignedShort { alignment_bits, .. }
            | Self::CInt { alignment_bits, .. }
            | Self::CUnsignedInt { alignment_bits, .. }
            | Self::CLong { alignment_bits, .. }
            | Self::CUnsignedLong { alignment_bits, .. }
            | Self::CLongLong { alignment_bits, .. }
            | Self::CUnsignedLongLong { alignment_bits, .. }
            | Self::CFloat { alignment_bits, .. }
            | Self::CDouble { alignment_bits, .. } => Some(alignment_bits),
            Self::Bool
            | Self::I8
            | Self::U8
            | Self::I16
            | Self::U16
            | Self::I32
            | Self::U32
            | Self::I64
            | Self::U64
            | Self::F32
            | Self::F64 => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustMacro {
    pub(crate) macro_id: MacroId,
    pub(crate) identity_file: FileId,
    pub(crate) rust_name: RustName,
    pub(crate) source_name: String,
    pub(crate) form: MacroForm,
    pub(crate) category: MacroCategory,
    pub(crate) body: String,
    pub(crate) normalized_tokens: Vec<String>,
    pub(crate) value: Option<MacroValue>,
    pub(crate) occurrences: Vec<MacroOccurrence>,
    pub(crate) support: SupportStatus,
    pub(crate) emitted: bool,
}

impl RustMacro {
    pub const fn macro_id(&self) -> MacroId {
        self.macro_id
    }
    pub const fn identity_file(&self) -> FileId {
        self.identity_file
    }
    pub fn rust_name(&self) -> &RustName {
        &self.rust_name
    }
    pub fn source_name(&self) -> &str {
        &self.source_name
    }
    pub const fn form(&self) -> MacroForm {
        self.form
    }
    pub const fn category(&self) -> MacroCategory {
        self.category
    }
    pub fn body(&self) -> &str {
        &self.body
    }
    pub fn normalized_tokens(&self) -> &[String] {
        &self.normalized_tokens
    }
    pub fn value(&self) -> Option<&MacroValue> {
        self.value.as_ref()
    }
    pub fn occurrences(&self) -> &[MacroOccurrence] {
        &self.occurrences
    }
    pub fn support(&self) -> &SupportStatus {
        &self.support
    }
    pub const fn emitted(&self) -> bool {
        self.emitted
    }
}
