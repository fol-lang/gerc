use bic::{BindingPackage, ResolvedLinkPlan, ValidationReport};

/// Primary input container for a `gec` generation run.
///
/// Wraps a required `bic::BindingPackage` plus optional enrichment data.
/// The `BindingPackage` is the single source of truth for C declarations;
/// validation and link-plan data are supplementary evidence.
#[derive(Debug, Clone)]
pub struct GecInput {
    /// The core `bic` analysis output — required.
    pub package: BindingPackage,
    /// Optional validation report (declaration-vs-artifact matching).
    pub validation: Option<ValidationReport>,
    /// Optional resolved link plan (library/artifact resolution).
    pub link_plan: Option<ResolvedLinkPlan>,
}

impl GecInput {
    /// Create a new input from a `BindingPackage` alone.
    pub fn from_package(package: BindingPackage) -> Self {
        Self {
            package,
            validation: None,
            link_plan: None,
        }
    }

    /// Attach a validation report.
    pub fn with_validation(mut self, report: ValidationReport) -> Self {
        self.validation = Some(report);
        self
    }

    /// Attach a resolved link plan.
    pub fn with_link_plan(mut self, plan: ResolvedLinkPlan) -> Self {
        self.link_plan = Some(plan);
        self
    }

    /// Returns `true` if the package contains no items and no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.package.is_empty()
    }

    /// Number of declaration items in the underlying package.
    pub fn item_count(&self) -> usize {
        self.package.item_count()
    }

    /// Whether validation evidence is attached.
    pub fn has_validation(&self) -> bool {
        self.validation.is_some()
    }

    /// Whether a resolved link plan is attached.
    pub fn has_link_plan(&self) -> bool {
        self.link_plan.is_some()
    }
}

impl From<BindingPackage> for GecInput {
    fn from(package: BindingPackage) -> Self {
        Self::from_package(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_package() -> BindingPackage {
        BindingPackage::new()
    }

    #[test]
    fn from_package_basic() {
        let input = GecInput::from_package(empty_package());
        assert!(input.is_empty());
        assert_eq!(input.item_count(), 0);
        assert!(!input.has_validation());
        assert!(!input.has_link_plan());
    }

    #[test]
    fn from_trait() {
        let input: GecInput = empty_package().into();
        assert!(input.is_empty());
    }

    #[test]
    fn with_link_plan() {
        let input = GecInput::from_package(empty_package())
            .with_link_plan(ResolvedLinkPlan::default());
        assert!(input.has_link_plan());
        assert!(!input.has_validation());
    }
}
