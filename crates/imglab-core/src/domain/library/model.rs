use crate::{DomainError, DomainResult, LibraryId, LibrarySummary};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryManifest {
    pub id: String,
    pub name: String,
    pub schema_version: u32,
    pub created_at: String,
    pub app: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLibrary {
    pub id: LibraryId,
    pub name: String,
    pub root_path: PathBuf,
    pub schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryAlias(String);

impl RegistryAlias {
    pub fn parse(alias: &str) -> DomainResult<Self> {
        let trimmed = alias.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidLibraryAlias {
                message: "library alias cannot be empty".to_string(),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn ensure_schema_supported(
    schema_version: u32,
    current_schema_version: u32,
) -> DomainResult<()> {
    if schema_version > current_schema_version {
        return Err(DomainError::SchemaMismatch {
            expected: current_schema_version,
            found: schema_version,
        });
    }
    Ok(())
}

pub fn library_from_manifest(root_path: &Path, manifest: &LibraryManifest) -> ResourceLibrary {
    ResourceLibrary {
        id: LibraryId(manifest.id.clone()),
        name: manifest.name.clone(),
        root_path: root_path.to_path_buf(),
        schema_version: manifest.schema_version,
    }
}

pub fn summary_from_manifest(
    root_path: &Path,
    manifest: &LibraryManifest,
    hidden: bool,
) -> LibrarySummary {
    let library = library_from_manifest(root_path, manifest);
    LibrarySummary {
        id: library.id,
        name: library.name,
        root_path: library.root_path,
        hidden,
        schema_version: library.schema_version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(schema_version: u32) -> LibraryManifest {
        LibraryManifest {
            id: "library-1".to_string(),
            name: "Library".to_string(),
            schema_version,
            created_at: "1".to_string(),
            app: "image-prompt-lab".to_string(),
        }
    }

    #[test]
    fn schema_compatibility_rejects_future_versions() {
        let error = ensure_schema_supported(7, 6).expect_err("future schema");
        assert!(matches!(error, DomainError::SchemaMismatch { .. }));
    }

    #[test]
    fn registry_alias_rejects_empty_text() {
        let error = RegistryAlias::parse("   ").expect_err("empty alias");
        assert!(matches!(error, DomainError::InvalidLibraryAlias { .. }));
    }

    #[test]
    fn summary_preserves_manifest_identity() {
        let summary = summary_from_manifest(Path::new("/tmp/library"), &manifest(6), false);
        assert_eq!(summary.id.0, "library-1");
        assert_eq!(summary.name, "Library");
        assert_eq!(summary.schema_version, 6);
        assert!(!summary.hidden);
    }
}
