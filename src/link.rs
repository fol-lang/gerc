use std::{ffi::OsString, path::PathBuf};

use linc::contract::{
    ArtifactFingerprint, LinkAtom, ProviderId, ResolvedArtifact, ResolvedLinkPlan,
};

/// Ordered, lossless Rust-side projection of the validated native link plan.
/// Paths and native names remain `PathBuf`/`OsString`; they are never shell-
/// split, UTF-8-normalized, deduplicated, or flattened into a text blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustLinkPlan {
    atoms: Vec<RustLinkAtom>,
}

impl RustLinkPlan {
    pub(crate) fn from_validated(plan: &ResolvedLinkPlan) -> Self {
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
        Self { atoms }
    }

    pub fn atoms(&self) -> &[RustLinkAtom] {
        &self.atoms
    }
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

    use linc::contract::{LinkAtom, ResolvedLinkPlan};

    use super::{RustLinkAtom, RustLinkPlan};

    #[test]
    fn native_path_units_order_and_repetition_are_lossless() {
        let path = PathBuf::from(OsString::from_vec(b"/tmp/gerc-\xff".to_vec()));
        let upstream = ResolvedLinkPlan::try_new(vec![
            LinkAtom::SearchNative(path.clone()),
            LinkAtom::SearchNative(path.clone()),
        ])
        .expect("normalized absolute non-UTF8 path");
        let projected = RustLinkPlan::from_validated(&upstream);
        assert_eq!(projected.atoms().len(), 2);
        assert_eq!(projected.atoms()[0], projected.atoms()[1]);
        match &projected.atoms()[0] {
            RustLinkAtom::SearchNative(actual) => assert_eq!(actual, &path),
            other => panic!("unexpected atom {other:?}"),
        }
    }
}
