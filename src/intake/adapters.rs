use linc::{BindingPackage, SourcePackage};

use super::GecInput;

pub(super) fn binding_package_from_json(json: &str) -> Result<BindingPackage, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Transitional compatibility adapter from a legacy `linc::BindingPackage`.
pub fn input_from_binding_package(package: BindingPackage) -> GecInput {
    GecInput::from_package(package)
}

/// Transitional compatibility adapter from a legacy `BindingPackage` JSON payload.
pub fn input_from_binding_json(json: &str) -> Result<GecInput, String> {
    binding_package_from_json(json).map(GecInput::from_package)
}

/// Transitional compatibility adapter that recovers a `SourcePackage` from a
/// legacy `BindingPackage`.
pub fn source_from_binding_package(package: &BindingPackage) -> SourcePackage {
    linc::intake::adapters::from_binding_package(package)
}
