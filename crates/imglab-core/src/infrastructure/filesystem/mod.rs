use crate::application::ports::ManagedFileStore;
use crate::library::{
    extension_for_mime_type, file_digest, image_dimensions, load_version, managed_original_path,
    mime_type_for_extension, normalized_extension, timestamp_string, LocalLibraryService,
    CURRENT_CHECKSUM_ALGORITHM,
};
use crate::{AssetVersionId, DomainError, DomainResult, ManagedFileImport, ManagedFileMetadata};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

impl ManagedFileStore for LocalLibraryService {
    fn read_source_bytes(&self, source_path: &Path) -> DomainResult<Vec<u8>> {
        fs::read(source_path).map_err(|error| io_error(source_path, error))
    }

    fn import_original(
        &self,
        library_path: &Path,
        source_path: &Path,
        mime_type_override: Option<&str>,
    ) -> DomainResult<ManagedFileImport> {
        if !source_path.is_file() {
            return Err(DomainError::Io {
                path: source_path.display().to_string(),
                message: "source file does not exist".to_string(),
            });
        }

        let extension = mime_type_override
            .map(extension_for_mime_type)
            .map(str::to_string)
            .unwrap_or_else(|| normalized_extension(source_path));
        let version_id = AssetVersionId(Uuid::new_v4().to_string());
        let timestamp = timestamp_string();
        let relative_path = managed_original_path(&version_id, &extension, &timestamp);
        let destination_path = library_path.join(&relative_path);

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        let temporary_path = destination_path.with_extension(format!("{extension}.tmp"));
        fs::copy(source_path, &temporary_path).map_err(|error| io_error(&temporary_path, error))?;
        fs::rename(&temporary_path, &destination_path)
            .map_err(|error| io_error(&destination_path, error))?;

        let checksum = file_digest(&destination_path, CURRENT_CHECKSUM_ALGORITHM)?;
        let (width, height) = image_dimensions(&destination_path)?;
        let mime_type = mime_type_override
            .map(str::to_string)
            .unwrap_or_else(|| mime_type_for_extension(&extension).to_string());

        Ok(ManagedFileImport {
            version_id,
            metadata: ManagedFileMetadata {
                file_path: relative_path,
                checksum_algorithm: CURRENT_CHECKSUM_ALGORITHM.to_string(),
                checksum,
                width,
                height,
                mime_type,
            },
        })
    }

    fn read_version_bytes(
        &self,
        library_path: &Path,
        version_id: &AssetVersionId,
    ) -> DomainResult<Vec<u8>> {
        let connection = LocalLibraryService::open_library_database(library_path)?;
        let version = load_version(&connection, version_id)?;
        let path = library_path.join(version.file_path);
        fs::read(&path).map_err(|error| io_error(&path, error))
    }

    fn write_generated_bytes(
        &self,
        _library_path: &Path,
        mime_type: &str,
        bytes: &[u8],
    ) -> DomainResult<PathBuf> {
        let extension = extension_for_mime_type(mime_type);
        let temp_dir = std::env::temp_dir().join(format!("imglab-generated-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).map_err(|error| io_error(&temp_dir, error))?;
        let temp_file = temp_dir.join(format!("output.{extension}"));
        fs::write(&temp_file, bytes).map_err(|error| io_error(&temp_file, error))?;
        Ok(temp_file)
    }
}

fn io_error(path: &Path, error: std::io::Error) -> DomainError {
    DomainError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_original_with_managed_path_and_file_metadata() {
        let root = std::env::temp_dir().join(format!("imglab-file-store-test-{}", Uuid::new_v4()));
        let source = root.join("source.png");
        fs::create_dir_all(&root).expect("create root");
        fs::write(&source, png_bytes(32, 18)).expect("write source");

        let service = LocalLibraryService::new(root.join("registry.sqlite"));
        let imported = service
            .import_original(&root, &source, None)
            .expect("import original");
        let stored_path = root.join(&imported.metadata.file_path);

        assert!(stored_path.is_file());
        assert!(imported.metadata.file_path.starts_with("originals"));
        assert_eq!(imported.metadata.mime_type, "image/png");
        assert_eq!(imported.metadata.width, Some(32));
        assert_eq!(imported.metadata.height, Some(18));
        assert_eq!(imported.metadata.checksum_algorithm, "SHA-256");
        assert!(!imported.metadata.checksum.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png.extend_from_slice(&[8, 2, 0, 0, 0]);
        png.extend_from_slice(&0u32.to_be_bytes());
        png
    }
}
