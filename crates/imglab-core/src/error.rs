use std::error::Error;
use std::fmt::{Display, Formatter};

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    LibraryNotFound {
        path: String,
    },
    SchemaMismatch {
        expected: u32,
        found: u32,
    },
    ProviderUnavailable {
        provider: String,
    },
    CredentialMissing {
        provider: String,
    },
    GenerationFailed {
        provider: String,
        message: String,
    },
    InvalidAssetReference {
        id: String,
    },
    InvalidTaskReference {
        id: String,
    },
    FileIntegrityMismatch {
        version_id: String,
        message: String,
    },
    ConcurrentWriteConflict {
        message: String,
    },
    InvalidSmartAlbumQuery {
        message: String,
    },
    InvalidGalleryQuery {
        message: String,
    },
    InvalidLibraryBackup {
        message: String,
    },
    InvalidLibraryAlias {
        message: String,
    },
    ImportDestinationNotEmpty {
        path: String,
    },
    ZipIoError {
        path: String,
        message: String,
    },
    UnsupportedProvider {
        provider: String,
    },
    UnsupportedProviderCapability {
        provider: String,
        capability: String,
    },
    InvalidGenerationParameters {
        message: String,
    },
    Io {
        path: String,
        message: String,
    },
    Database {
        message: String,
    },
    Serialization {
        message: String,
    },
}

impl DomainError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::LibraryNotFound { .. } => "LibraryNotFound",
            Self::SchemaMismatch { .. } => "SchemaMismatch",
            Self::ProviderUnavailable { .. } => "ProviderUnavailable",
            Self::CredentialMissing { .. } => "CredentialMissing",
            Self::GenerationFailed { .. } => "GenerationFailed",
            Self::InvalidAssetReference { .. } => "InvalidAssetReference",
            Self::InvalidTaskReference { .. } => "InvalidTaskReference",
            Self::FileIntegrityMismatch { .. } => "FileIntegrityMismatch",
            Self::ConcurrentWriteConflict { .. } => "ConcurrentWriteConflict",
            Self::InvalidSmartAlbumQuery { .. } => "InvalidSmartAlbumQuery",
            Self::InvalidGalleryQuery { .. } => "InvalidGalleryQuery",
            Self::InvalidLibraryBackup { .. } => "InvalidLibraryBackup",
            Self::InvalidLibraryAlias { .. } => "InvalidLibraryAlias",
            Self::ImportDestinationNotEmpty { .. } => "ImportDestinationNotEmpty",
            Self::ZipIoError { .. } => "ZipIoError",
            Self::UnsupportedProvider { .. } => "UnsupportedProvider",
            Self::UnsupportedProviderCapability { .. } => "UnsupportedProviderCapability",
            Self::InvalidGenerationParameters { .. } => "InvalidGenerationParameters",
            Self::Io { .. } => "Io",
            Self::Database { .. } => "Database",
            Self::Serialization { .. } => "Serialization",
        }
    }

    pub fn recoverable(&self) -> bool {
        !matches!(
            self,
            Self::SchemaMismatch { .. } | Self::ConcurrentWriteConflict { .. }
        )
    }
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryNotFound { path } => write!(f, "library not found: {path}"),
            Self::SchemaMismatch { expected, found } => {
                write!(f, "schema mismatch: expected {expected}, found {found}")
            }
            Self::ProviderUnavailable { provider } => write!(f, "provider unavailable: {provider}"),
            Self::CredentialMissing { provider } => write!(f, "credential missing: {provider}"),
            Self::GenerationFailed { provider, message } => {
                write!(f, "generation failed for {provider}: {message}")
            }
            Self::InvalidAssetReference { id } => write!(f, "invalid asset reference: {id}"),
            Self::InvalidTaskReference { id } => write!(f, "invalid task reference: {id}"),
            Self::FileIntegrityMismatch {
                version_id,
                message,
            } => {
                write!(f, "file integrity mismatch for {version_id}: {message}")
            }
            Self::ConcurrentWriteConflict { message } => {
                write!(f, "concurrent write conflict: {message}")
            }
            Self::InvalidSmartAlbumQuery { message } => {
                write!(f, "invalid smart album query: {message}")
            }
            Self::InvalidGalleryQuery { message } => {
                write!(f, "invalid gallery query: {message}")
            }
            Self::InvalidLibraryBackup { message } => {
                write!(f, "invalid library backup: {message}")
            }
            Self::InvalidLibraryAlias { message } => write!(f, "invalid library alias: {message}"),
            Self::ImportDestinationNotEmpty { path } => {
                write!(f, "import destination is not empty: {path}")
            }
            Self::ZipIoError { path, message } => write!(f, "zip io error at {path}: {message}"),
            Self::UnsupportedProvider { provider } => write!(f, "unsupported provider: {provider}"),
            Self::UnsupportedProviderCapability {
                provider,
                capability,
            } => {
                write!(
                    f,
                    "unsupported provider capability for {provider}: {capability}"
                )
            }
            Self::InvalidGenerationParameters { message } => {
                write!(f, "invalid generation parameters: {message}")
            }
            Self::Io { path, message } => write!(f, "io error at {path}: {message}"),
            Self::Database { message } => write!(f, "database error: {message}"),
            Self::Serialization { message } => write!(f, "serialization error: {message}"),
        }
    }
}

impl Error for DomainError {}
