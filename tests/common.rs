pub fn from_linc_source(source: &linc::SourcePackage) -> gec::SourcePackage {
    let json = serde_json::to_string(source).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_binding_package(package: &linc::ir::BindingPackage) -> gec::SourcePackage {
    let source = linc::intake::adapters::from_binding_package(package);
    from_linc_source(&source)
}
