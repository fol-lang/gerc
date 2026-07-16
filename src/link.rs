use std::{ffi::OsString, path::PathBuf};

use linc::contract::{
    ArtifactFingerprint, LinkAtom, ProviderId, ResolvedArtifact, ResolvedLinkPlan,
};
use parc::contract::{ObjectFormat, TargetFingerprint, TargetSpec};

use crate::{GenerationError, GenerationResult};

/// Ordered, lossless Rust-side projection of the validated native link plan.
/// Paths and native names remain `PathBuf`/`OsString`; they are never shell-
/// split, UTF-8-normalized, deduplicated, or flattened into a text blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustLinkPlan {
    target_fingerprint: TargetFingerprint,
    object_format: ObjectFormat,
    atoms: Vec<RustLinkAtom>,
}

impl RustLinkPlan {
    pub(crate) fn from_validated(plan: &ResolvedLinkPlan, target: &TargetSpec) -> Self {
        let atoms =
            plan.atoms()
                .iter()
                .map(|atom| match atom {
                    LinkAtom::SearchNative(path) => RustLinkAtom::SearchNative(path.clone()),
                    LinkAtom::Object(artifact) => RustLinkAtom::Artifact(RustLinkArtifact::new(
                        RustLinkArtifactKind::Object,
                        artifact,
                    )),
                    LinkAtom::StaticLibrary(artifact) => RustLinkAtom::Artifact(
                        RustLinkArtifact::new(RustLinkArtifactKind::StaticLibrary, artifact),
                    ),
                    LinkAtom::DynamicLibrary(artifact) => RustLinkAtom::Artifact(
                        RustLinkArtifact::new(RustLinkArtifactKind::DynamicLibrary, artifact),
                    ),
                    LinkAtom::ImportLibrary(artifact) => RustLinkAtom::Artifact(
                        RustLinkArtifact::new(RustLinkArtifactKind::ImportLibrary, artifact),
                    ),
                    LinkAtom::Framework {
                        name,
                        search_path,
                        artifact,
                    } => RustLinkAtom::Framework {
                        name: name.clone(),
                        search_path: search_path.clone(),
                        artifact: RustLinkArtifact::new(RustLinkArtifactKind::Framework, artifact),
                    },
                    LinkAtom::GroupStart => RustLinkAtom::GroupStart,
                    LinkAtom::GroupEnd => RustLinkAtom::GroupEnd,
                })
                .collect();
        Self {
            target_fingerprint: target.fingerprint(),
            object_format: target.object_format(),
            atoms,
        }
    }

    pub const fn target_fingerprint(&self) -> TargetFingerprint {
        self.target_fingerprint
    }

    pub const fn object_format(&self) -> ObjectFormat {
        self.object_format
    }

    pub fn atoms(&self) -> &[RustLinkAtom] {
        &self.atoms
    }

    /// Exact argv for a direct GNU-family linker invocation. Every vector
    /// element is one process argument; no shell or whitespace parser is
    /// involved. Framework atoms fail rather than guessing host syntax.
    pub fn gnu_linker_arguments(&self) -> GenerationResult<GnuLinkerArguments> {
        self.require_elf_target()?;
        let mut arguments = Vec::new();
        for atom in &self.atoms {
            match atom {
                RustLinkAtom::SearchNative(path) => {
                    arguments.push(OsString::from("-L"));
                    arguments.push(path.as_os_str().to_os_string());
                }
                RustLinkAtom::Artifact(artifact) => {
                    arguments.push(artifact.canonical_path().as_os_str().to_os_string());
                }
                RustLinkAtom::Framework { .. } => {
                    return Err(GenerationError::UnsupportedLinkProjection {
                        reason: "Apple framework atom requested from the GNU linker projection",
                    });
                }
                RustLinkAtom::GroupStart => arguments.push(OsString::from("--start-group")),
                RustLinkAtom::GroupEnd => arguments.push(OsString::from("--end-group")),
            }
        }
        Ok(GnuLinkerArguments(arguments))
    }

    /// Exact arguments to append to a `rustc` invocation for the certified
    /// GNU-family target lane. Each native linker argument is carried by its
    /// own repeated `-C link-arg=...` pair; this never creates a
    /// whitespace-split `-Clink-args` blob. Exact artifact paths therefore
    /// cannot be mistaken for additional Rust input files.
    pub fn rustc_arguments(&self) -> GenerationResult<RustcLinkArguments> {
        self.require_elf_target()?;
        let mut arguments = Vec::new();
        for atom in &self.atoms {
            match atom {
                RustLinkAtom::SearchNative(path) => {
                    arguments.push(OsString::from("-L"));
                    arguments.push(native_prefixed("native=", path.as_os_str()));
                }
                RustLinkAtom::Artifact(artifact) => {
                    push_rustc_link_arg(&mut arguments, artifact.canonical_path().as_os_str());
                }
                RustLinkAtom::Framework { .. } => {
                    return Err(GenerationError::UnsupportedLinkProjection {
                        reason: "framework rustc argv is not certified for the GNU target lane",
                    });
                }
                RustLinkAtom::GroupStart => {
                    push_rustc_link_arg(&mut arguments, std::ffi::OsStr::new("--start-group"));
                }
                RustLinkAtom::GroupEnd => {
                    push_rustc_link_arg(&mut arguments, std::ffi::OsStr::new("--end-group"));
                }
            }
        }
        Ok(RustcLinkArguments(arguments))
    }

    /// Typed build-graph directives. These are domain values rather than
    /// newline-formatted `cargo:` strings, so native paths, exact objects,
    /// order, and repetition cannot be lost before FOL picks a backend.
    pub fn cargo_directives(&self) -> Vec<CargoLinkDirective> {
        self.atoms
            .iter()
            .map(|atom| match atom {
                RustLinkAtom::SearchNative(path) => CargoLinkDirective::SearchNative(path.clone()),
                RustLinkAtom::Artifact(artifact) => CargoLinkDirective::LinkArg(
                    artifact.canonical_path().as_os_str().to_os_string(),
                ),
                RustLinkAtom::Framework {
                    name,
                    search_path,
                    artifact,
                } => CargoLinkDirective::Framework {
                    name: name.clone(),
                    search_path: search_path.clone(),
                    artifact: artifact.clone(),
                },
                RustLinkAtom::GroupStart => {
                    CargoLinkDirective::LinkArg(OsString::from("--start-group"))
                }
                RustLinkAtom::GroupEnd => {
                    CargoLinkDirective::LinkArg(OsString::from("--end-group"))
                }
            })
            .collect()
    }

    fn require_elf_target(&self) -> GenerationResult<()> {
        if self.object_format != ObjectFormat::Elf {
            return Err(GenerationError::UnsupportedLinkProjection {
                reason: "GNU linker argv requested for a non-ELF target",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GnuLinkerArguments(Vec<OsString>);

impl GnuLinkerArguments {
    pub fn arguments(&self) -> &[OsString] {
        &self.0
    }

    pub fn into_arguments(self) -> Vec<OsString> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustcLinkArguments(Vec<OsString>);

impl RustcLinkArguments {
    pub fn arguments(&self) -> &[OsString] {
        &self.0
    }

    pub fn into_arguments(self) -> Vec<OsString> {
        self.0
    }
}

fn native_prefixed(prefix: &str, value: &std::ffi::OsStr) -> OsString {
    let mut argument = OsString::from(prefix);
    argument.push(value);
    argument
}

fn push_rustc_link_arg(arguments: &mut Vec<OsString>, value: &std::ffi::OsStr) {
    arguments.push(OsString::from("-C"));
    arguments.push(native_prefixed("link-arg=", value));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CargoLinkDirective {
    SearchNative(PathBuf),
    LinkArg(OsString),
    Framework {
        name: OsString,
        search_path: PathBuf,
        artifact: RustLinkArtifact,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustLinkAtom {
    SearchNative(PathBuf),
    Artifact(RustLinkArtifact),
    Framework {
        name: OsString,
        search_path: PathBuf,
        artifact: RustLinkArtifact,
    },
    GroupStart,
    GroupEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustLinkArtifactKind {
    Object,
    StaticLibrary,
    DynamicLibrary,
    ImportLibrary,
    Framework,
}

impl RustLinkArtifactKind {
    pub(crate) const fn fingerprint_tag(self) -> &'static [u8] {
        match self {
            Self::Object => b"object",
            Self::StaticLibrary => b"static-library",
            Self::DynamicLibrary => b"dynamic-library",
            Self::ImportLibrary => b"import-library",
            Self::Framework => b"framework",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustLinkArtifact {
    kind: RustLinkArtifactKind,
    provider: ProviderId,
    artifact_fingerprint: ArtifactFingerprint,
    canonical_path: PathBuf,
}

impl RustLinkArtifact {
    fn new(kind: RustLinkArtifactKind, artifact: &ResolvedArtifact) -> Self {
        Self {
            kind,
            provider: artifact.provider_id(),
            artifact_fingerprint: artifact.artifact_fingerprint(),
            canonical_path: artifact.canonical_path().to_path_buf(),
        }
    }

    pub const fn kind(&self) -> RustLinkArtifactKind {
        self.kind
    }

    pub const fn provider(&self) -> ProviderId {
        self.provider
    }

    pub const fn artifact_fingerprint(&self) -> ArtifactFingerprint {
        self.artifact_fingerprint
    }

    pub fn canonical_path(&self) -> &std::path::Path {
        &self.canonical_path
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt as _, path::PathBuf};

    use linc::contract::{
        ArtifactFingerprint, ArtifactKind, LinkAtom, ProviderProvenance, ProviderResolution,
        ResolvedArtifact, ResolvedArtifactInput, ResolvedLinkPlan,
    };
    use parc::contract::{ObjectFormat, TargetSpec};

    use super::{RustLinkAtom, RustLinkPlan};

    #[test]
    fn native_path_units_order_and_repetition_are_lossless() {
        let path = PathBuf::from(OsString::from_vec(b"/tmp/gerc-\xff".to_vec()));
        let upstream = ResolvedLinkPlan::try_new(vec![
            LinkAtom::SearchNative(path.clone()),
            LinkAtom::SearchNative(path.clone()),
        ])
        .expect("normalized absolute non-UTF8 path");
        let target = elf_target();
        let projected = RustLinkPlan::from_validated(&upstream, &target);
        assert_eq!(projected.atoms().len(), 2);
        assert_eq!(projected.atoms()[0], projected.atoms()[1]);
        match &projected.atoms()[0] {
            RustLinkAtom::SearchNative(actual) => assert_eq!(actual, &path),
            other => panic!("unexpected atom {other:?}"),
        }

        let rustc = projected
            .rustc_arguments()
            .expect("GNU rustc projection")
            .into_arguments();
        assert_eq!(rustc.len(), 4);
        assert_eq!(rustc[0], OsString::from("-L"));
        assert_eq!(rustc[2], OsString::from("-L"));
        let mut expected = b"native=".to_vec();
        expected.extend_from_slice(b"/tmp/gerc-\xff");
        assert_eq!(rustc[1], OsString::from_vec(expected.clone()));
        assert_eq!(rustc[3], OsString::from_vec(expected));
    }

    #[test]
    fn groups_are_individual_repeated_rustc_link_args() {
        let upstream = ResolvedLinkPlan::try_new(vec![
            LinkAtom::GroupStart,
            LinkAtom::GroupEnd,
            LinkAtom::GroupStart,
            LinkAtom::GroupEnd,
        ])
        .expect("balanced repeated groups");
        let target = elf_target();
        let arguments = RustLinkPlan::from_validated(&upstream, &target)
            .rustc_arguments()
            .expect("GNU rustc projection")
            .into_arguments();
        assert_eq!(
            arguments,
            [
                "-C",
                "link-arg=--start-group",
                "-C",
                "link-arg=--end-group",
                "-C",
                "link-arg=--start-group",
                "-C",
                "link-arg=--end-group",
            ]
            .map(OsString::from)
        );

        let wrong_target = RustLinkPlan {
            target_fingerprint: target.fingerprint(),
            object_format: ObjectFormat::Coff,
            atoms: Vec::new(),
        };
        assert!(wrong_target.rustc_arguments().is_err());
        assert!(wrong_target.gnu_linker_arguments().is_err());
    }

    #[test]
    fn exact_non_utf8_object_is_one_lossless_argv_value() {
        let (target, observed_target) = elf_target_and_observation();
        let path = PathBuf::from(OsString::from_vec(
            b"/tmp/gerc-exact-object-\xff.o".to_vec(),
        ));
        let object = ResolvedArtifact::try_new(ResolvedArtifactInput {
            artifact_fingerprint: ArtifactFingerprint::from_content(b"h4 exact object"),
            canonical_path: path.clone(),
            kind: ArtifactKind::Object,
            resolution: ProviderResolution::Explicit,
            provenance: ProviderProvenance::User,
            observed_target,
        })
        .expect("checked exact object");
        let upstream =
            ResolvedLinkPlan::try_new(vec![LinkAtom::Object(object)]).expect("exact object plan");
        let projected = RustLinkPlan::from_validated(&upstream, &target);
        let [RustLinkAtom::Artifact(artifact)] = projected.atoms() else {
            panic!("one exact artifact atom was expected");
        };
        assert_eq!(artifact.kind(), super::RustLinkArtifactKind::Object);
        assert_eq!(artifact.canonical_path(), path);

        let rustc = projected
            .rustc_arguments()
            .expect("ELF rustc arguments")
            .into_arguments();
        assert_eq!(rustc[0], "-C");
        let mut expected = b"link-arg=".to_vec();
        expected.extend_from_slice(b"/tmp/gerc-exact-object-\xff.o");
        assert_eq!(rustc[1], OsString::from_vec(expected));
        assert_eq!(
            projected
                .gnu_linker_arguments()
                .expect("GNU linker arguments")
                .into_arguments(),
            [path.into_os_string()]
        );
    }

    fn elf_target() -> TargetSpec {
        parc::contract::decode_source_package(parc::contract::corpus::COMPLETE_SOURCE_PACKAGE_JSON)
            .expect("checked target corpus")
            .target()
            .clone()
    }

    fn elf_target_and_observation() -> (TargetSpec, linc::contract::ObservedTarget) {
        let source = parc::contract::decode_source_package(
            parc::contract::corpus::COMPLETE_SOURCE_PACKAGE_JSON,
        )
        .expect("checked source corpus")
        .into_complete(&linc::contract::corpus::preservation_selection())
        .expect("complete preservation source");
        let analysis = linc::contract::corpus::validated_preservation_link_analysis(&source)
            .expect("checked preservation analysis");
        let observed = analysis.package().inventories()[0]
            .artifact()
            .observed_target()
            .clone();
        (source.source().target().clone(), observed)
    }
}
