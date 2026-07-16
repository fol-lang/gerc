use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use crate::{GenerationError, GenerationResult};

/// A deterministic, in-memory output set. GERC does not own arbitrary
/// filesystem materialization or overwrite policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFileSet {
    files: Vec<GeneratedFile>,
}

impl GeneratedFileSet {
    pub(crate) fn try_new(mut files: Vec<GeneratedFile>) -> GenerationResult<Self> {
        files.sort_by(|left, right| left.path.0.cmp(&right.path.0));
        let mut paths = BTreeSet::new();
        for file in &files {
            if !paths.insert(file.path.0.clone()) {
                return Err(GenerationError::GeneratedFileInvariant {
                    reason: "generated file path occurs more than once",
                });
            }
        }
        Ok(Self { files })
    }

    pub fn files(&self) -> &[GeneratedFile] {
        &self.files
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Option<&GeneratedFile> {
        let path = path.as_ref();
        self.files
            .binary_search_by(|file| file.path.as_path().cmp(path))
            .ok()
            .map(|index| &self.files[index])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFile {
    path: GeneratedPath,
    contents: Vec<u8>,
}

impl GeneratedFile {
    pub(crate) fn utf8(path: &'static str, contents: String) -> GenerationResult<Self> {
        Ok(Self {
            path: GeneratedPath::try_new(path)?,
            contents: contents.into_bytes(),
        })
    }

    pub fn path(&self) -> &GeneratedPath {
        &self.path
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }

    pub fn utf8_contents(&self) -> Option<&str> {
        std::str::from_utf8(&self.contents).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneratedPath(PathBuf);

impl GeneratedPath {
    fn try_new(path: &'static str) -> GenerationResult<Self> {
        if path.as_bytes().contains(&0) {
            return Err(GenerationError::GeneratedFileInvariant {
                reason: "generated paths cannot contain NUL",
            });
        }
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                !matches!(component, Component::Normal(_)) || component.as_os_str().is_empty()
            })
        {
            return Err(GenerationError::GeneratedFileInvariant {
                reason: "generated paths must be nonempty normalized relative paths",
            });
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}
