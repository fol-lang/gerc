use linc::contract::{corpus as linc_corpus, ValidatedLinkAnalysis};
use parc::contract::{corpus as parc_corpus, CompleteSourcePackage, SourcePackage};

pub fn decoded_source() -> SourcePackage {
    parc::contract::decode_source_package(parc_corpus::COMPLETE_SOURCE_PACKAGE_JSON)
        .expect("packaged PARC preservation corpus must decode")
}

pub fn preservation_pair() -> (CompleteSourcePackage, ValidatedLinkAnalysis) {
    let source = decoded_source()
        .into_complete(&linc_corpus::preservation_selection())
        .expect("packaged PARC preservation selection must be complete");
    let evidence = linc_corpus::validated_preservation_link_analysis(&source)
        .expect("packaged LINC preservation analysis must cover PARC closure");
    (source, evidence)
}

pub fn declaration_id(source: &SourcePackage, name: &str) -> parc::contract::DeclarationId {
    source
        .declarations()
        .iter()
        .find(|declaration| {
            declaration
                .name
                .as_ref()
                .is_some_and(|source_name| source_name.normalized == name)
        })
        .unwrap_or_else(|| panic!("corpus declaration {name:?}"))
        .id
}
