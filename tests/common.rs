#![allow(dead_code)]

pub fn from_linc_source(source: &linc::SourcePackage) -> gerc::SourcePackage {
    let json = serde_json::to_string(source).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_binding_package(package: &linc::ir::BindingPackage) -> gerc::SourcePackage {
    let source = linc::intake::adapters::from_binding_package(package);
    from_linc_source(&source)
}

pub fn from_linc_validation(report: &linc::ValidationReport) -> gerc::ValidationReport {
    let json = serde_json::to_string(report).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_linc_link_plan(plan: &linc::ResolvedLinkPlan) -> gerc::ResolvedLinkPlan {
    let json = serde_json::to_string(plan).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_linc_analysis(analysis: &linc::LinkAnalysisPackage) -> gerc::LinkAnalysisPackage {
    let json = serde_json::to_string(analysis).unwrap();
    serde_json::from_str(&json).unwrap()
}
