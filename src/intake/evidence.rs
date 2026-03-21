use linc::{ResolvedLinkPlan, ValidationReport};

/// Optional upstream evidence attached to a generation run.
///
/// This stays separate from the required declaration package so intake can
/// evolve toward the split `source + evidence` model described in `PLAN.md`.
#[derive(Debug, Clone, Default)]
pub struct EvidenceInputs {
    pub validation: Option<ValidationReport>,
    pub link_plan: Option<ResolvedLinkPlan>,
}
