mod evidence;
mod source;

use crate::c::{
    BindingItem, BindingPackage, DeclarationProvenance, LinkAnalysisPackage, ResolvedLinkPlan,
    ValidationReport,
};

pub use evidence::EvidenceInputs;
pub use source::{
    SourceDeclaration, SourceEnum, SourceEnumVariant, SourceField, SourceFunction, SourceLinkKind,
    SourceLinkRequirement, SourceMacro, SourcePackage, SourceParameter, SourceRecord, SourceType,
    SourceTypeAlias, SourceVariable,
};
#[allow(dead_code)]
pub(crate) fn source_package_from_binding(package: &BindingPackage) -> SourcePackage {
    source::source_package_from_binding(package)
}

/// Primary input container for a `gerc` generation run.
///
/// Wraps a required source contract plus optional link/binary evidence.
/// Source meaning comes from `SourcePackage`; binary/link evidence comes
/// separately so `gerc` does not have to receive source meaning through `linc`.
///
/// ## Required vs optional `linc` evidence
///
/// - **Required**: `SourcePackage` — always needed; contains declarations,
///   macros, and source-side link declarations.
/// - **Optional**: `LinkAnalysisPackage` — link/binary evidence.
/// - **Optional**: `ValidationReport` — symbol-level validation evidence.
///   Useful for gating generation on verified symbols but not required for
///   basic projection.
/// - **Optional**: `ResolvedLinkPlan` — resolved native link requirements.
///   When absent, `gerc` uses the source-derived raw link surface.
#[derive(Debug, Clone)]
pub struct GercInput {
    /// Required source contract.
    pub source: SourcePackage,
    /// Optional validation and link evidence.
    pub evidence: EvidenceInputs,
}

impl GercInput {
    /// Create a new input from a `SourcePackage`.
    pub fn from_source_package(source: SourcePackage) -> Self {
        Self {
            source,
            evidence: EvidenceInputs::default(),
        }
    }

    /// Attach a validation report.
    pub fn with_validation(mut self, report: ValidationReport) -> Self {
        self.evidence.validation = Some(report);
        self
    }

    /// Attach a resolved link plan.
    pub fn with_link_plan(mut self, plan: ResolvedLinkPlan) -> Self {
        self.evidence.link_plan = Some(plan);
        self
    }

    /// Attach a full link-analysis package.
    pub fn with_analysis(mut self, analysis: LinkAnalysisPackage) -> Self {
        self.evidence.analysis = Some(analysis);
        self
    }

    /// Attach evidence in one step.
    pub fn with_evidence(mut self, evidence: EvidenceInputs) -> Self {
        self.evidence = evidence;
        self
    }

    /// Normalize the intake: remove duplicate items and ensure provenance
    /// alignment with items. This is idempotent.
    pub fn normalize(&mut self) {
        let mut package = self.binding_package();
        dedup_items(&mut package);
        align_provenance(&mut package);
        self.source = source::source_package_from_binding(&package);
    }

    /// Returns `true` if the package contains no items and no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.binding_package().is_empty()
    }

    /// Number of declaration items in the underlying package.
    pub fn item_count(&self) -> usize {
        self.binding_package().item_count()
    }

    /// Whether link-analysis evidence is attached.
    pub fn has_analysis(&self) -> bool {
        self.evidence.analysis.is_some()
    }

    /// Whether validation evidence is attached.
    pub fn has_validation(&self) -> bool {
        self.evidence.validation.is_some()
    }

    /// Whether a resolved link plan is attached.
    pub fn has_link_plan(&self) -> bool {
        self.evidence.link_plan.is_some()
    }

    /// Whether the package has provenance entries aligned with items.
    pub fn has_aligned_provenance(&self) -> bool {
        let package = self.binding_package();
        package.provenance.is_empty() || package.provenance.len() == package.items.len()
    }

    /// Construct from a JSON string containing a `SourcePackage`.
    pub fn from_source_json(json: &str) -> Result<Self, String> {
        let source = source::source_package_from_json(json)?;
        Ok(Self::from_source_package(source))
    }

    pub(crate) fn binding_package(&self) -> BindingPackage {
        source::binding_package_from_source(&self.source)
    }
}

impl From<SourcePackage> for GercInput {
    fn from(source: SourcePackage) -> Self {
        Self::from_source_package(source)
    }
}

/// Remove duplicate items by (kind, name) identity. Keeps first occurrence.
fn dedup_items(pkg: &mut BindingPackage) {
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();
    let mut deduped_prov = Vec::new();

    for (i, item) in pkg.items.drain(..).enumerate() {
        let key = item_identity(&item);
        if seen.insert(key) {
            deduped.push(item);
            if let Some(prov) = pkg.provenance.get(i).cloned() {
                deduped_prov.push(prov);
            }
        }
    }
    pkg.items = deduped;
    if !pkg.provenance.is_empty() {
        pkg.provenance = deduped_prov;
    }
}

fn item_identity(item: &BindingItem) -> String {
    match item {
        BindingItem::Function(f) => format!("fn:{}", f.name),
        BindingItem::Record(r) => {
            format!("rec:{}", r.name.as_deref().unwrap_or("<anon>"))
        }
        BindingItem::Enum(e) => {
            format!("enum:{}", e.name.as_deref().unwrap_or("<anon>"))
        }
        BindingItem::TypeAlias(t) => format!("alias:{}", t.name),
        BindingItem::Variable(v) => format!("var:{}", v.name),
        BindingItem::Unsupported(u) => {
            format!("unsup:{}", u.name.as_deref().unwrap_or("<anon>"))
        }
    }
}

/// Ensure provenance vec is aligned with items (pad with defaults if short).
fn align_provenance(pkg: &mut BindingPackage) {
    if pkg.provenance.is_empty() && !pkg.items.is_empty() {
        return;
    }
    while pkg.provenance.len() < pkg.items.len() {
        pkg.provenance.push(DeclarationProvenance::default());
    }
    pkg.provenance.truncate(pkg.items.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c::*;
    use serde_json::to_string_pretty;

    fn empty_package() -> BindingPackage {
        BindingPackage::new()
    }

    fn sample_function(name: &str) -> BindingItem {
        BindingItem::Function(FunctionBinding {
            name: name.into(),
            calling_convention: CallingConvention::C,
            parameters: vec![],
            return_type: BindingType::Void,
            variadic: false,
            source_offset: None,
        })
    }

    fn fixture_source_package() -> SourcePackage {
        let mut source = SourcePackage {
            source_path: Some("fixtures/demo.h".into()),
            ..SourcePackage::default()
        };
        source
            .declarations
            .push(SourceDeclaration::Function(SourceFunction {
                name: "demo_init".into(),
                parameters: vec![SourceParameter {
                    name: Some("flags".into()),
                    ty: SourceType::UInt,
                }],
                return_type: SourceType::Int,
                variadic: false,
                source_offset: Some(12),
            }));
        source
            .declarations
            .push(SourceDeclaration::Variable(SourceVariable {
                name: "demo_errno".into(),
                ty: SourceType::Int,
                source_offset: Some(27),
            }));
        source.macros.push(SourceMacro {
            name: "DEMO_API_VERSION".into(),
            body: "3".into(),
            function_like: false,
        });
        source
    }

    fn input_from_binding(pkg: BindingPackage) -> GercInput {
        GercInput::from_source_package(source::source_package_from_binding(&pkg))
    }

    #[test]
    fn from_source_package_basic_empty() {
        let input = GercInput::from_source_package(SourcePackage::default());
        assert!(input.is_empty());
        assert_eq!(input.item_count(), 0);
        assert!(!input.has_analysis());
        assert!(!input.has_validation());
        assert!(!input.has_link_plan());
    }

    #[test]
    fn from_source_package_basic() {
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
        source.macros.push(SourceMacro {
            name: "FOO".into(),
            body: "1".into(),
            function_like: false,
        });

        let input = GercInput::from_source_package(source);
        assert_eq!(input.item_count(), 1);
        assert_eq!(input.source.macros.len(), 1);
        assert_eq!(input.source.macros[0].name, "FOO");
    }

    #[test]
    fn from_source_trait() {
        let mut source = SourcePackage::default();
        source
            .declarations
            .push(SourceDeclaration::TypeAlias(SourceTypeAlias {
                name: "size_t".into(),
                target: SourceType::ULong,
                source_offset: None,
            }));

        let input: GercInput = source.into();
        assert_eq!(input.item_count(), 1);
    }

    #[test]
    fn with_link_plan() {
        let input = GercInput::from_source_package(SourcePackage::default())
            .with_link_plan(ResolvedLinkPlan::default());
        assert!(input.has_link_plan());
        assert!(!input.has_analysis());
        assert!(!input.has_validation());
    }

    #[test]
    fn with_analysis() {
        let input = GercInput::from_source_package(SourcePackage::default())
            .with_analysis(LinkAnalysisPackage::default());
        assert!(input.has_analysis());
    }

    #[test]
    fn with_evidence_sets_both_optional_inputs() {
        let input = GercInput::from_source_package(SourcePackage::default()).with_evidence(
            EvidenceInputs {
                analysis: Some(LinkAnalysisPackage::default()),
                validation: Some(ValidationReport {
                    phases: Vec::new(),
                    entries: Vec::new(),
                    summary: ValidationSummary::default(),
                    matches: Vec::new(),
                }),
                link_plan: Some(ResolvedLinkPlan::default()),
            },
        );

        assert!(input.has_analysis());
        assert!(input.has_validation());
        assert!(input.has_link_plan());
    }

    #[test]
    fn normalize_dedup_functions() {
        let mut pkg = empty_package();
        pkg.items.push(sample_function("foo"));
        pkg.items.push(sample_function("foo"));
        pkg.items.push(sample_function("bar"));

        let mut input = input_from_binding(pkg);
        assert_eq!(input.item_count(), 3);
        input.normalize();
        assert_eq!(input.item_count(), 2);
    }

    #[test]
    fn normalize_preserves_provenance_alignment() {
        let mut pkg = empty_package();
        pkg.items.push(sample_function("a"));
        pkg.items.push(sample_function("b"));
        pkg.provenance.push(DeclarationProvenance {
            item_name: Some("a".into()),
            ..Default::default()
        });

        let mut input = input_from_binding(pkg);
        input.normalize();
        assert!(input.has_aligned_provenance());
    }

    #[test]
    fn normalize_is_idempotent() {
        let mut pkg = empty_package();
        pkg.items.push(sample_function("foo"));
        pkg.items.push(sample_function("foo"));

        let mut input = input_from_binding(pkg);
        input.normalize();
        let count_after_first = input.item_count();
        input.normalize();
        assert_eq!(input.item_count(), count_after_first);
    }

    #[test]
    fn from_source_json_with_function() {
        let json = r#"{
            "source_path": "test.h",
            "declarations": [
                {"Function": {
                    "name": "foo",
                    "parameters": [],
                    "return_type": "Void",
                    "variadic": false,
                    "source_offset": null
                }}
            ]
        }"#;
        let input = GercInput::from_source_json(json).unwrap();
        assert_eq!(input.item_count(), 1);
    }

    #[test]
    fn source_fixture_contract_matches_binding_projection() {
        let input = GercInput::from_source_package(fixture_source_package());
        let package = input.binding_package();

        assert_eq!(input.item_count(), 2);
        assert_eq!(package.source_path.as_deref(), Some("fixtures/demo.h"));
        assert_eq!(package.macros.len(), 1);
        assert!(matches!(
            &package.items[0],
            BindingItem::Function(function) if function.name == "demo_init"
        ));
        assert!(matches!(
            &package.items[1],
            BindingItem::Variable(variable) if variable.name == "demo_errno"
        ));
    }

    #[test]
    fn from_source_json_fixture_contract() {
        let json = to_string_pretty(&fixture_source_package()).unwrap();
        let input = GercInput::from_source_json(&json).unwrap();
        let package = input.binding_package();

        assert_eq!(input.item_count(), 2);
        assert_eq!(package.source_path.as_deref(), Some("fixtures/demo.h"));
        assert_eq!(package.macros.len(), 1);
        assert!(matches!(
            &package.items[0],
            BindingItem::Function(function) if function.name == "demo_init"
        ));
        assert!(matches!(
            &package.items[1],
            BindingItem::Variable(variable) if variable.name == "demo_errno"
        ));
    }

    #[test]
    fn from_source_json_invalid() {
        let result = GercInput::from_source_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn normalize_keeps_source_contract_usable_when_provenance_is_not_roundtripped() {
        let mut pkg = empty_package();
        pkg.items.push(sample_function("foo"));
        pkg.provenance.push(DeclarationProvenance {
            item_name: Some("foo".into()),
            ..Default::default()
        });

        let mut input = input_from_binding(pkg);
        input.normalize();
        assert_eq!(input.item_count(), 1);
        assert!(input.has_aligned_provenance());
    }
}
