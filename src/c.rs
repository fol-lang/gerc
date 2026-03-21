use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BindingTarget {
    #[serde(default)]
    pub target_triple: Option<String>,
    #[serde(default)]
    pub compiler_command: Option<String>,
    #[serde(default)]
    pub compiler_version: Option<String>,
    #[serde(default)]
    pub flavor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingDefine {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BindingInputs {
    #[serde(default)]
    pub entry_headers: Vec<String>,
    #[serde(default)]
    pub include_dirs: Vec<String>,
    #[serde(default)]
    pub defines: Vec<BindingDefine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingItem {
    Function(FunctionBinding),
    Record(RecordBinding),
    Enum(EnumBinding),
    TypeAlias(TypeAliasBinding),
    Variable(VariableBinding),
    Unsupported(UnsupportedItem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingItemKind {
    Function,
    Record,
    Enum,
    TypeAlias,
    Variable,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingType {
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
    Pointer {
        pointee: Box<BindingType>,
        const_pointee: bool,
        #[serde(default)]
        qualifiers: TypeQualifiers,
    },
    Array(Box<BindingType>, Option<u64>),
    Qualified {
        ty: Box<BindingType>,
        #[serde(default)]
        qualifiers: TypeQualifiers,
    },
    FunctionPointer {
        return_type: Box<BindingType>,
        parameters: Vec<BindingType>,
        variadic: bool,
    },
    TypedefRef(String),
    RecordRef(String),
    EnumRef(String),
    Opaque(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TypeQualifiers {
    #[serde(default)]
    pub is_const: bool,
    #[serde(default)]
    pub is_volatile: bool,
    #[serde(default)]
    pub is_restrict: bool,
    #[serde(default)]
    pub is_atomic: bool,
}

impl BindingType {
    pub fn ptr(pointee: BindingType) -> Self {
        Self::Pointer {
            pointee: Box::new(pointee),
            const_pointee: false,
            qualifiers: TypeQualifiers::default(),
        }
    }

    pub fn const_ptr(pointee: BindingType) -> Self {
        Self::Pointer {
            pointee: Box::new(pointee),
            const_pointee: true,
            qualifiers: TypeQualifiers::default(),
        }
    }

    pub fn qualified(ty: BindingType, qualifiers: TypeQualifiers) -> Self {
        if qualifiers == TypeQualifiers::default() {
            ty
        } else {
            Self::Qualified {
                ty: Box::new(ty),
                qualifiers,
            }
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            Self::Void => true,
            Self::Qualified { ty, .. } => ty.is_void(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallingConvention {
    C,
    Cdecl,
    Stdcall,
    Fastcall,
    Vectorcall,
    Thiscall,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionBinding {
    pub name: String,
    pub calling_convention: CallingConvention,
    pub parameters: Vec<ParameterBinding>,
    pub return_type: BindingType,
    pub variadic: bool,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterBinding {
    pub name: Option<String>,
    pub ty: BindingType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordKind {
    Struct,
    Union,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldBinding {
    pub name: Option<String>,
    pub ty: BindingType,
    #[serde(default)]
    pub bit_width: Option<u64>,
    #[serde(default)]
    pub layout: Option<FieldLayout>,
}

impl FieldBinding {
    pub fn is_bitfield(&self) -> bool {
        self.bit_width.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldLayout {
    #[serde(default)]
    pub offset_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordRepresentation {
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub align: Option<u64>,
    #[serde(default)]
    pub completeness: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbiConfidence {
    DeclaredOnly,
    LayoutProbed,
    FieldOffsetsProbed,
    RepresentationProbed,
    PartialBitfieldLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordBinding {
    pub kind: RecordKind,
    pub name: Option<String>,
    pub fields: Option<Vec<FieldBinding>>,
    #[serde(default)]
    pub representation: Option<RecordRepresentation>,
    #[serde(default)]
    pub abi_confidence: Option<AbiConfidence>,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumRepresentation {
    #[serde(default)]
    pub underlying_size: Option<u64>,
    #[serde(default)]
    pub is_signed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumBinding {
    pub name: Option<String>,
    pub variants: Vec<EnumVariant>,
    #[serde(default)]
    pub representation: Option<EnumRepresentation>,
    #[serde(default)]
    pub abi_confidence: Option<AbiConfidence>,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AliasResolution {
    #[serde(default)]
    pub alias_chain: Vec<String>,
    pub terminal_target: BindingType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAliasBinding {
    pub name: String,
    pub target: BindingType,
    #[serde(default)]
    pub canonical_resolution: Option<AliasResolution>,
    #[serde(default)]
    pub abi_confidence: Option<AbiConfidence>,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableBinding {
    pub name: String,
    pub ty: BindingType,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedItem {
    pub name: Option<String>,
    pub reason: String,
    pub source_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroKind {
    Integer,
    String,
    Expression,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MacroForm {
    #[default]
    ObjectLike,
    FunctionLike,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MacroCategory {
    BindableConstant,
    ConfigurationFlag,
    AbiAffecting,
    #[default]
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroValue {
    Integer(i128),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroBinding {
    pub name: String,
    pub body: String,
    pub function_like: bool,
    #[serde(default)]
    pub form: MacroForm,
    pub kind: MacroKind,
    #[serde(default)]
    pub category: MacroCategory,
    #[serde(default)]
    pub value: Option<MacroValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkRequirementSource {
    #[default]
    Declared,
    Inferred,
    Discovered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkLibraryKind {
    Default,
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkResolutionMode {
    #[default]
    Default,
    PreferStatic,
    PreferDynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NativeSurfaceKind {
    #[default]
    HeaderOnly,
    LibraryNames,
    ConcreteArtifacts,
    Mixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkLibrary {
    pub name: String,
    pub kind: LinkLibraryKind,
    #[serde(default)]
    pub source: LinkRequirementSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkArtifactKind {
    Object,
    StaticLibrary,
    SharedLibrary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkArtifact {
    pub path: String,
    pub kind: LinkArtifactKind,
    #[serde(default)]
    pub source: LinkRequirementSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkFramework {
    pub name: String,
    #[serde(default)]
    pub source: LinkRequirementSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkInput {
    Library(LinkLibrary),
    Artifact(LinkArtifact),
    Framework(LinkFramework),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BindingLinkSurface {
    #[serde(default)]
    pub preferred_mode: LinkResolutionMode,
    #[serde(default)]
    pub native_surface_kind: NativeSurfaceKind,
    #[serde(default)]
    pub platform_constraints: Vec<String>,
    #[serde(default)]
    pub include_paths: Vec<String>,
    #[serde(default)]
    pub framework_paths: Vec<String>,
    #[serde(default)]
    pub library_paths: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<LinkLibrary>,
    #[serde(default)]
    pub frameworks: Vec<LinkFramework>,
    #[serde(default)]
    pub artifacts: Vec<LinkArtifact>,
    #[serde(default)]
    pub ordered_inputs: Vec<LinkInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceOrigin {
    Entry,
    UserInclude,
    System,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    #[serde(default)]
    pub line: Option<usize>,
    #[serde(default)]
    pub column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeclarationProvenance {
    #[serde(default)]
    pub item_name: Option<String>,
    #[serde(default)]
    pub item_kind: Option<BindingItemKind>,
    #[serde(default)]
    pub source_offset: Option<usize>,
    #[serde(default)]
    pub source_origin: Option<SourceOrigin>,
    #[serde(default)]
    pub source_location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingPackage {
    #[serde(default)]
    pub target: BindingTarget,
    #[serde(default)]
    pub inputs: BindingInputs,
    #[serde(default)]
    pub macros: Vec<MacroBinding>,
    #[serde(default)]
    pub link: BindingLinkSurface,
    #[serde(default)]
    pub provenance: Vec<DeclarationProvenance>,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub items: Vec<BindingItem>,
}

impl BindingPackage {
    pub fn new() -> Self {
        Self {
            target: BindingTarget::default(),
            inputs: BindingInputs::default(),
            macros: Vec::new(),
            link: BindingLinkSurface::default(),
            provenance: Vec::new(),
            source_path: None,
            items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.macros.is_empty()
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl Default for BindingPackage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolVisibility {
    Default,
    Hidden,
    Protected,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Function,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchStatus {
    Matched,
    AbiShapeMismatch,
    Missing,
    UnresolvedDeclaredLinkInputs,
    DecorationMismatch,
    NotAFunction,
    NotAVariable,
    Hidden,
    WeakMatch,
    DuplicateProviders,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ValidationSummary {
    pub total: usize,
    pub matched: usize,
    #[serde(default)]
    pub abi_shape_mismatches: usize,
    pub missing: usize,
    pub unresolved_declared_link_inputs: usize,
    pub hidden: usize,
    pub weak_matches: usize,
    pub duplicate_providers: usize,
    pub decoration_mismatches: usize,
    pub kind_mismatches: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchConfidence {
    High,
    Medium,
    Low,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    ExactExported,
    AbiShapeVerified,
    WeakExported,
    HiddenProvider,
    DecoratedCandidate,
    ReexportedCandidate,
    DuplicateVisibleProviders,
    DeclaredLinkInputsWithoutProvider,
    MissingProvider,
    AbiShapeMismatch,
    KindMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMatch {
    pub name: String,
    pub item_kind: ItemKind,
    pub status: MatchStatus,
    pub visibility: Option<SymbolVisibility>,
    #[serde(default)]
    pub provider_artifacts: Vec<String>,
    pub confidence: MatchConfidence,
    pub evidence_kind: EvidenceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationPhaseReport;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEntry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    #[serde(default)]
    pub phases: Vec<ValidationPhaseReport>,
    #[serde(default)]
    pub entries: Vec<ValidationEntry>,
    #[serde(default)]
    pub summary: ValidationSummary,
    #[serde(default)]
    pub matches: Vec<SymbolMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderMatchKind {
    ExactArtifact,
    LibraryName,
    FrameworkName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderProvenance {
    DeclaredArtifact,
    DiscoveredInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProvider {
    pub artifact_path: String,
    pub match_kind: ProviderMatchKind,
    pub provenance: ProviderProvenance,
    #[serde(default)]
    pub dependency_edges: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RequirementResolution {
    #[default]
    Unresolved,
    Resolved,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedLinkRequirement {
    pub declared: LinkInput,
    #[serde(default)]
    pub source: LinkRequirementSource,
    #[serde(default)]
    pub resolution: RequirementResolution,
    #[serde(default)]
    pub providers: Vec<ResolvedProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResolvedLinkPlan {
    #[serde(default)]
    pub preferred_mode: LinkResolutionMode,
    #[serde(default)]
    pub native_surface_kind: NativeSurfaceKind,
    #[serde(default)]
    pub platform_constraints: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<LinkInput>,
    #[serde(default)]
    pub requirements: Vec<ResolvedLinkRequirement>,
    #[serde(default)]
    pub transitive_dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LinkAnalysisPackage {
    #[serde(default)]
    pub target: BindingTarget,
    #[serde(default)]
    pub inputs: BindingInputs,
    #[serde(default)]
    pub declared_link_surface: BindingLinkSurface,
    #[serde(default)]
    pub resolved_link_plan: Option<ResolvedLinkPlan>,
    #[serde(default)]
    pub validation: Option<ValidationReport>,
}
