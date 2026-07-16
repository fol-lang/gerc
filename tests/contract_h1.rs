mod support;

use gerc::{GenerationErrorCode, GenerationRequest, ItemSelection};
use parc::contract::Selection;

#[test]
fn item_selection_is_nonempty_unique_and_id_keyed() {
    let (source, _) = support::preservation_pair();
    let packet = support::declaration_id(source.source(), "parc_packet");

    let empty = ItemSelection::try_new([]).expect_err("empty selection must fail");
    assert_eq!(empty.code(), GenerationErrorCode::EmptySelection);
    assert_eq!(empty.stable_code(), "GERC-E1000");

    let duplicate = ItemSelection::try_new([packet, packet]).expect_err("duplicate IDs must fail");
    assert_eq!(duplicate.code(), GenerationErrorCode::DuplicateSelection);
}

#[test]
fn request_accepts_supported_root_subset_and_rejects_id_outside_roots() {
    let (source, evidence) = support::preservation_pair();
    let packet = support::declaration_id(source.source(), "parc_packet");
    let subset = ItemSelection::try_new([packet]).expect("ID subset");
    GenerationRequest::try_new(&source, &evidence, &subset).expect("supported subset request");

    let missing = support::declaration_id(source.source(), "parc_missing");
    let outside = ItemSelection::try_new([missing]).expect("valid ID selection shape");
    let error = GenerationRequest::try_new(&source, &evidence, &outside)
        .expect_err("package declaration outside proved roots must fail");
    assert_eq!(error.code(), GenerationErrorCode::SelectionMismatch);
    assert!(error.context().is_some());
}

#[test]
fn request_rejects_checked_evidence_that_misses_a_required_root() {
    let (_, evidence) = support::preservation_pair();
    let package = support::decoded_source();
    let missing = support::declaration_id(&package, "parc_missing");
    let source = package
        .into_complete(&Selection::only([missing]).expect("missing selection"))
        .expect("source declaration itself is complete");
    let selection = ItemSelection::try_new([missing]).expect("ID selection");
    let error = GenerationRequest::try_new(&source, &evidence, &selection)
        .expect_err("evidence for another checked closure must not be reused");
    assert_eq!(error.code(), GenerationErrorCode::EvidenceCoverageMismatch);
    assert_eq!(error.stable_code(), "GERC-E1102");
}
