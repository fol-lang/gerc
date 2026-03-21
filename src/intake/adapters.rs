use linc::BindingPackage;

pub(super) fn binding_package_from_json(json: &str) -> Result<BindingPackage, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
