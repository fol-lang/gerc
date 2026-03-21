use linc::{BindingPackage, SourcePackage};

pub(super) fn binding_package_from_source(source: &SourcePackage) -> BindingPackage {
    linc::from_source_package(source)
}

pub(super) fn source_package_from_binding(package: &BindingPackage) -> SourcePackage {
    linc::intake::adapters::from_binding_package(package)
}

pub(super) fn source_package_from_json(json: &str) -> Result<SourcePackage, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
