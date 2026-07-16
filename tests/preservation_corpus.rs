mod support;

use std::{
    ffi::{OsStr, OsString},
    fs,
    path::Path,
    process::Command,
};

use gerc::{
    generate, GenerationErrorCode, GenerationRequest, ItemSelection, RustItem, RustLinkAtom,
    RustScalar, RustTypeKind,
};
use linc::contract::{corpus as linc_corpus, CallableAbiAssessment};
use parc::contract::{
    CallingConvention, ExactInteger, MacroCategory, MacroValue, SourceDeclarationKind,
};

#[test]
fn preservation_supported_subset_generates_deterministically_and_compiles_no_std() {
    let (source, evidence) = support::preservation_pair();
    let packet = support::declaration_id(source.source(), "parc_packet");
    let mode = support::declaration_id(source.source(), "parc_mode");
    let selection = ItemSelection::try_new([packet, mode]).expect("supported ABI roots");

    let first = generate(
        GenerationRequest::try_new(&source, &evidence, &selection).expect("typed request"),
    )
    .expect("record/enum subset must generate");
    let second = generate(
        GenerationRequest::try_new(&source, &evidence, &selection).expect("repeat request"),
    )
    .expect("repeat generation");
    assert_eq!(first, second);

    assert_eq!(
        first.manifest().source_fingerprint(),
        source.source().fingerprint()
    );
    assert_eq!(
        first.manifest().target_fingerprint(),
        source.source().target_fingerprint()
    );
    assert_eq!(
        first.manifest().evidence_fingerprint(),
        linc_corpus::preservation_link_analysis_fingerprint()
    );

    let record = first
        .projection()
        .items()
        .iter()
        .find_map(|item| match item {
            RustItem::Record(record) if record.declaration() == packet => Some(record),
            _ => None,
        })
        .expect("projected packet record");
    assert_eq!(record.size_bits(), Some(32));
    assert_eq!(record.alignment_bits(), Some(32));
    assert_eq!(record.packing_bits(), None);
    assert_eq!(
        record.source_kind(),
        source_record_kind(source.source(), packet)
    );
    assert_eq!(
        record.source_completeness(),
        parc::contract::RecordCompleteness::Complete
    );
    assert_eq!(record.fields().len(), 1);
    assert_eq!(record.fields()[0].offset_bits(), 0);
    assert_eq!(record.fields()[0].size_bits(), 32);
    assert!(matches!(
        record.fields()[0].ty().kind(),
        RustTypeKind::Scalar(RustScalar::CInt {
            storage_bits: 32,
            alignment_bits: 32,
        })
    ));
    let source_record = match &source
        .source()
        .declaration(packet)
        .expect("source packet")
        .kind
    {
        SourceDeclarationKind::Record(record) => record,
        other => panic!("unexpected packet kind {other:?}"),
    };
    let source_record_declaration = source
        .source()
        .declaration(packet)
        .expect("source packet declaration");
    assert_eq!(
        record.source().identity(),
        &source_record_declaration.identity
    );
    assert_eq!(
        record.source().name(),
        source_record_declaration.name.as_ref()
    );
    assert_eq!(record.source().linkage(), source_record_declaration.linkage);
    assert_eq!(
        record.source().visibility(),
        source_record_declaration.visibility
    );
    assert_eq!(
        record.source().support(),
        &source_record_declaration.support
    );
    assert_eq!(
        record.source().occurrences(),
        source_record_declaration.occurrences
    );
    assert_eq!(
        record
            .source()
            .name()
            .expect("named source record")
            .normalized,
        "parc_packet"
    );
    assert_eq!(record.rust_name().as_str(), "parc_packet");
    assert_eq!(record.fields()[0].range(), source_record.fields[0].range);
    assert_eq!(
        record.fields()[0].provenance(),
        &source_record.fields[0].provenance
    );
    assert_eq!(
        record.fields()[0].source_name(),
        source_record.fields[0].name.as_ref()
    );
    assert_eq!(
        record.fields()[0].attributes(),
        source_record.fields[0].attributes
    );
    assert_eq!(
        record.fields()[0].support(),
        &source_record.fields[0].support
    );
    assert_eq!(
        record.fields()[0].identity_tokens(),
        source_record.fields[0].identity_tokens
    );
    assert_eq!(
        record.fields()[0].duplicate_ordinal(),
        source_record.fields[0].duplicate_ordinal
    );

    let enumeration = first
        .projection()
        .items()
        .iter()
        .find_map(|item| match item {
            RustItem::Enum(enumeration) if enumeration.declaration() == mode => Some(enumeration),
            _ => None,
        })
        .expect("projected mode enum");
    assert_eq!(enumeration.storage(), RustScalar::U32);
    let source_enum = match &source.source().declaration(mode).expect("source mode").kind {
        SourceDeclarationKind::Enum(enumeration) => enumeration,
        other => panic!("unexpected mode kind {other:?}"),
    };
    assert_eq!(enumeration.source().name().unwrap().normalized, "parc_mode");
    assert_eq!(enumeration.rust_name().as_str(), "parc_mode");
    assert_eq!(
        enumeration.explicit_underlying_type().is_some(),
        source_enum.explicit_underlying_type.is_some()
    );
    assert_eq!(enumeration.variants()[0].value(), ExactInteger::signed(7));
    assert_eq!(
        enumeration.variants()[0].source_name(),
        &source_enum.variants[0].name
    );
    assert_eq!(
        enumeration.variants()[0].attributes(),
        source_enum.variants[0].attributes
    );
    assert_eq!(
        enumeration.variants()[0].support(),
        &source_enum.variants[0].support
    );
    assert_eq!(
        enumeration.variants()[0].identity_tokens(),
        source_enum.variants[0].identity_tokens
    );
    assert_eq!(
        enumeration.variants()[0].duplicate_ordinal(),
        source_enum.variants[0].duplicate_ordinal
    );

    let abi_macro = first
        .projection()
        .macros()
        .iter()
        .find(|source_macro| source_macro.source_name() == "PARC_ABI_LEVEL")
        .expect("ABI macro preserved");
    assert_eq!(abi_macro.category(), MacroCategory::AbiAffecting);
    assert_eq!(
        abi_macro.value(),
        Some(&MacroValue::Integer {
            value: ExactInteger::signed(7)
        })
    );
    assert!(abi_macro.emitted());
    let source_abi_macro = source
        .source()
        .macros()
        .iter()
        .find(|source_macro| source_macro.name == "PARC_ABI_LEVEL")
        .expect("source ABI macro");
    assert_eq!(abi_macro.identity_file(), source_abi_macro.identity_file);
    assert_eq!(abi_macro.support(), &source_abi_macro.support);

    let names: Vec<_> = first
        .link_plan()
        .atoms()
        .iter()
        .map(|atom| match atom {
            RustLinkAtom::Artifact(artifact) => artifact
                .canonical_path()
                .file_name()
                .expect("corpus artifact file name")
                .to_os_string(),
            other => panic!("unexpected corpus atom {other:?}"),
        })
        .collect();
    assert_eq!(
        names,
        [
            "libfirst.a",
            "librepeat.a",
            "libmiddle.so",
            "librepeat.a",
            "libparc_fixture.a",
        ]
        .map(OsString::from)
    );
    assert_eq!(first.link_plan().atoms()[1], first.link_plan().atoms()[3]);
    assert_eq!(
        first.link_plan().target_fingerprint(),
        source.source().target_fingerprint()
    );
    assert_eq!(
        first.link_plan().object_format(),
        source.source().target().object_format()
    );
    let rustc_arguments = first
        .link_plan()
        .rustc_arguments()
        .expect("certified GNU rustc arguments")
        .into_arguments();
    assert_eq!(rustc_arguments.len(), first.link_plan().atoms().len() * 2);
    assert!(rustc_arguments
        .chunks_exact(2)
        .all(|pair| pair[0] == "-C" && pair[1].to_string_lossy().starts_with("link-arg=/")));
    assert_eq!(rustc_arguments[2..4], rustc_arguments[6..8]);
    assert!(!rustc_arguments
        .iter()
        .any(|argument| argument.to_string_lossy().contains("link-args")));

    let generated = first
        .files()
        .get("src/lib.rs")
        .expect("deterministic Rust source");
    let source_text = generated.utf8_contents().expect("UTF-8 Rust source");
    assert!(source_text.contains("#![no_std]"));
    assert!(source_text.contains("pub value: core::ffi::c_int"));
    assert!(source_text.contains("pub type parc_mode = u32;"));
    assert!(source_text.contains("core::mem::offset_of!(parc_packet, value) == 0"));
    assert!(source_text.contains("core::mem::size_of::<parc_mode>() == 4"));
    assert!(source_text.contains("core::mem::align_of::<parc_mode>() == 4"));
    compile_generated(source_text);
}

fn source_record_kind(
    source: &parc::contract::SourcePackage,
    declaration: parc::contract::DeclarationId,
) -> parc::contract::RecordKind {
    match &source.declaration(declaration).expect("source record").kind {
        SourceDeclarationKind::Record(record) => record.kind,
        other => panic!("unexpected record kind {other:?}"),
    }
}

#[test]
fn preservation_win64_function_is_safely_rejected_on_linux() {
    let (source, evidence) = support::preservation_pair();
    let open = support::declaration_id(source.source(), "parc_open");
    let selection = ItemSelection::try_new([open]).expect("Win64 root ID");
    let source_function = match &source
        .source()
        .declaration(open)
        .expect("source open function")
        .kind
    {
        SourceDeclarationKind::Function(function) => function,
        other => panic!("unexpected open kind {other:?}"),
    };
    assert_eq!(source_function.calling_convention, CallingConvention::Win64);
    let callable_evidence = evidence
        .package()
        .declaration_evidence()
        .iter()
        .find(|entry| entry.declaration() == open)
        .expect("open declaration evidence")
        .callable_abi();
    assert!(matches!(
        callable_evidence,
        CallableAbiAssessment::Confirmed {
            calling_convention: CallingConvention::Win64,
            ..
        }
    ));
    let request = GenerationRequest::try_new(&source, &evidence, &selection)
        .expect("upstream evidence for Win64 declaration is preserved and checked");
    let error = generate(request).expect_err("Win64 must not be projected for a Linux target");
    assert_eq!(
        error.code(),
        GenerationErrorCode::UnsupportedCallingConvention
    );
    assert_eq!(error.stable_code(), "GERC-E2000");
    let context = error.context().expect("fingerprint error context");
    assert_eq!(context.source_fingerprint(), source.source().fingerprint());
    assert_eq!(
        context.evidence_fingerprint(),
        evidence.package().fingerprint()
    );
}

#[test]
fn generated_output_is_identical_across_processes_and_working_directories() {
    let root = std::env::temp_dir().join(format!("gerc-h4-cross-process-{}", std::process::id()));
    let first = root.join("first/working/directory");
    let second = root.join("second/other/directory");
    fs::create_dir_all(&first).expect("create first working directory");
    fs::create_dir_all(&second).expect("create second working directory");

    let first_fingerprint = child_generation_fingerprint(&first);
    let second_fingerprint = child_generation_fingerprint(&second);
    fs::remove_dir_all(&root).expect("remove owned cross-process directory");
    assert_eq!(first_fingerprint, second_fingerprint);
}

#[test]
fn cross_process_generation_fingerprint_child() {
    if std::env::var_os("GERC_H4_FINGERPRINT_CHILD").is_none() {
        return;
    }
    let (source, evidence) = support::preservation_pair();
    let packet = support::declaration_id(source.source(), "parc_packet");
    let mode = support::declaration_id(source.source(), "parc_mode");
    let selection = ItemSelection::try_new([packet, mode]).expect("child ABI roots");
    let bundle = generate(
        GenerationRequest::try_new(&source, &evidence, &selection).expect("child typed request"),
    )
    .expect("child strict generation");
    println!(
        "GERC_H4_FINGERPRINT={}",
        bundle.manifest().generation_fingerprint()
    );
}

fn child_generation_fingerprint(working_directory: &Path) -> String {
    let output = Command::new(std::env::current_exe().expect("current integration-test binary"))
        .arg("--exact")
        .arg("cross_process_generation_fingerprint_child")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env("GERC_H4_FINGERPRINT_CHILD", "1")
        .current_dir(working_directory)
        .output()
        .expect("run deterministic generation child");
    assert!(
        output.status.success(),
        "generation child failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    const MARKER: &str = "GERC_H4_FINGERPRINT=";
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| {
            line.find(MARKER)
                .and_then(|offset| line[offset + MARKER.len()..].split_whitespace().next())
        })
        .expect("child fingerprint marker")
        .to_owned()
}

fn compile_generated(source: &str) {
    let directory = std::env::temp_dir().join(format!(
        "gerc-preservation-generated-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create generated-source test directory");
    let input = directory.join("lib.rs");
    fs::write(&input, source).expect("write generated Rust source");
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| OsStr::new("rustc").to_owned());
    let result = Command::new(rustc)
        .arg("--crate-name=gerc_preservation_generated")
        .arg("--crate-type=lib")
        .arg("--edition=2021")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(directory.join("lib.rmeta"))
        .arg(&input)
        .output()
        .expect("run rustc on generated source");
    let _ = fs::remove_dir_all(&directory);
    assert!(
        result.status.success(),
        "generated no_std source did not compile:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
