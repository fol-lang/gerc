use linc::{BindingPackage, SourcePackage};

pub(super) fn binding_package_from_source(source: &SourcePackage) -> BindingPackage {
    linc::intake::adapters::to_binding_package(source)
}

pub(super) fn source_package_from_json(json: &str) -> Result<SourcePackage, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
