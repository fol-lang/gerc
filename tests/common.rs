#![allow(dead_code)]

pub fn from_linc_source(source: &linc::SourcePackage) -> gec::SourcePackage {
    let json = serde_json::to_string(source).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_binding_package(package: &linc::ir::BindingPackage) -> gec::SourcePackage {
    let source = linc::intake::adapters::from_binding_package(package);
    from_linc_source(&source)
}

pub fn from_linc_validation(report: &linc::ValidationReport) -> gec::ValidationReport {
    let json = serde_json::to_string(report).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_linc_link_plan(plan: &linc::ResolvedLinkPlan) -> gec::ResolvedLinkPlan {
    let json = serde_json::to_string(plan).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn from_linc_analysis(analysis: &linc::LinkAnalysisPackage) -> gec::LinkAnalysisPackage {
    let json = serde_json::to_string(analysis).unwrap();
    serde_json::from_str(&json).unwrap()
}
