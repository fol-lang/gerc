use std::collections::{BTreeMap, BTreeSet};

use linc::contract::ValidatedLinkAnalysis;
use parc::contract::{ClosureRequirement, CompleteSourcePackage, DeclarationId, Selection};

use crate::{GenerationContext, GenerationError, GenerationResult};

/// Exact, ID-keyed generation roots. Names are never accepted as selection
/// keys, and duplicate IDs are rejected rather than silently deduplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSelection {
    declarations: Vec<DeclarationId>,
}

impl ItemSelection {
    pub fn try_new(
        declarations: impl IntoIterator<Item = DeclarationId>,
    ) -> GenerationResult<Self> {
        let mut declarations: Vec<_> = declarations.into_iter().collect();
        if declarations.is_empty() {
            return Err(GenerationError::EmptySelection);
        }

        let mut seen = BTreeSet::new();
        for declaration in &declarations {
            if !seen.insert(*declaration) {
                return Err(GenerationError::DuplicateSelection {
                    declaration: *declaration,
                });
            }
        }
        declarations.sort_unstable();
        Ok(Self { declarations })
    }

    /// Resolves the PARC selection to its exact declaration-ID roots.
    pub fn from_complete(source: &CompleteSourcePackage) -> Self {
        let declarations = match source.selection() {
            Selection::AllSupported => source
                .source()
                .declarations()
                .iter()
                .filter(|declaration| {
                    source
                        .source()
                        .selection_contains(source.selection(), declaration.id)
                })
                .map(|declaration| declaration.id)
                .collect(),
            Selection::Only(ids) | Selection::OpaqueOnly(ids) => ids.clone(),
        };
        Self { declarations }
    }

    pub fn declarations(&self) -> &[DeclarationId] {
        &self.declarations
    }

    pub fn contains(&self, declaration: DeclarationId) -> bool {
        self.declarations.binary_search(&declaration).is_ok()
    }
}

/// The only production generation intake boundary.
///
/// Both upstream values are checked proof wrappers. No raw package, source-
/// only route, JSON transmuter, or optional-evidence fallback exists.
#[derive(Debug, Clone)]
pub struct GenerationRequest<'a> {
    source: &'a CompleteSourcePackage,
    evidence: &'a ValidatedLinkAnalysis,
    selection: &'a ItemSelection,
    declaration_closure: Vec<RequiredDeclaration>,
}

impl<'a> GenerationRequest<'a> {
    pub fn try_new(
        source: &'a CompleteSourcePackage,
        evidence: &'a ValidatedLinkAnalysis,
        selection: &'a ItemSelection,
    ) -> GenerationResult<Self> {
        let package = evidence.package();
        let context = GenerationContext::new(
            source.source().fingerprint(),
            source.source().target_fingerprint(),
            package.fingerprint(),
        );
        let result = (|| {
            if package.source_fingerprint() != source.source().fingerprint() {
                return Err(GenerationError::SourceFingerprintMismatch);
            }
            if package.target_fingerprint() != source.source().target_fingerprint() {
                return Err(GenerationError::TargetFingerprintMismatch);
            }
            let available_roots = ItemSelection::from_complete(source);
            if !selection
                .declarations()
                .iter()
                .all(|declaration| available_roots.contains(*declaration))
            {
                return Err(GenerationError::SelectionMismatch);
            }

            let declaration_closure = selected_closure(source, selection)?;

            let evidence_ids: BTreeSet<_> = package
                .declaration_evidence()
                .iter()
                .map(|evidence| evidence.declaration())
                .collect();
            if !evidence_covers_required(&declaration_closure, &evidence_ids) {
                return Err(GenerationError::EvidenceCoverageMismatch);
            }

            Ok(Self {
                source,
                evidence,
                selection,
                declaration_closure,
            })
        })();
        result.map_err(|error| error.with_context(context))
    }

    pub const fn source(&self) -> &'a CompleteSourcePackage {
        self.source
    }

    pub const fn evidence(&self) -> &'a ValidatedLinkAnalysis {
        self.evidence
    }

    pub const fn selection(&self) -> &'a ItemSelection {
        self.selection
    }

    pub(crate) fn declaration_closure(&self) -> &[RequiredDeclaration] {
        &self.declaration_closure
    }
}

fn evidence_covers_required(
    required: &[RequiredDeclaration],
    evidence: &BTreeSet<DeclarationId>,
) -> bool {
    required
        .iter()
        .all(|entry| evidence.contains(&entry.declaration()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequiredDeclaration {
    declaration: DeclarationId,
    requirement: ClosureRequirement,
}

impl RequiredDeclaration {
    pub(crate) const fn declaration(self) -> DeclarationId {
        self.declaration
    }

    pub(crate) const fn requirement(self) -> ClosureRequirement {
        self.requirement
    }
}

fn selected_closure(
    source: &CompleteSourcePackage,
    selection: &ItemSelection,
) -> GenerationResult<Vec<RequiredDeclaration>> {
    let mut union = BTreeMap::<DeclarationId, ClosureRequirement>::new();
    for declaration in selection.declarations() {
        let root_requirement = source
            .declaration_closure()
            .binary_search_by_key(declaration, |entry| entry.declaration())
            .ok()
            .map(|index| source.declaration_closure()[index].requirement())
            .ok_or(GenerationError::SelectionMismatch)?;
        let single_root = match root_requirement {
            parc::contract::ClosureRequirement::Opaque => Selection::opaque([*declaration]),
            parc::contract::ClosureRequirement::Definition => Selection::only([*declaration]),
        }
        .map_err(|_| GenerationError::SelectionMismatch)?;
        let complete = source
            .source()
            .clone()
            .into_complete(&single_root)
            .map_err(|_| GenerationError::SelectionMismatch)?;
        for entry in complete.declaration_closure() {
            union
                .entry(entry.declaration())
                .and_modify(|current| *current = (*current).max(entry.requirement()))
                .or_insert(entry.requirement());
        }
    }

    Ok(union
        .into_iter()
        .map(|(declaration, requirement)| RequiredDeclaration {
            declaration,
            requirement,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, str::FromStr as _};

    use parc::contract::{ClosureRequirement, DeclarationId};

    use super::{evidence_covers_required, RequiredDeclaration};

    #[test]
    fn evidence_coverage_requires_every_transitive_declaration() {
        let root = DeclarationId::from_str(
            "pdecl1_524bcccd395cfaad5d0697f01bc545663e82eaad03be1e515beeb81933f5b37d",
        )
        .expect("root id");
        let transitive = DeclarationId::from_str(
            "pdecl1_e1aa560084aa2b17941ce5f7d3ce72cf86cb6fcc38919e74579b59cd2ee8f103",
        )
        .expect("transitive id");
        let required = [
            RequiredDeclaration {
                declaration: root,
                requirement: ClosureRequirement::Definition,
            },
            RequiredDeclaration {
                declaration: transitive,
                requirement: ClosureRequirement::Definition,
            },
        ];
        assert!(!evidence_covers_required(
            &required,
            &BTreeSet::from([root])
        ));
        assert!(evidence_covers_required(
            &required,
            &BTreeSet::from([root, transitive])
        ));
    }
}
