use crate::{AssetId, AssetVersionId, GenerationEventId};
use std::path::PathBuf;

pub use crate::dto::{AssetSummary, IntegrityIssue, IntegrityIssueKind, VersionSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub id: AssetId,
    pub title: Option<String>,
    pub category: Option<String>,
    pub rating: Option<u8>,
    pub status: String,
}

impl Asset {
    pub fn from_summary(summary: AssetSummary) -> Self {
        Self {
            id: summary.id,
            title: summary.title,
            category: summary.category,
            rating: summary.rating,
            status: summary.status,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetVersion {
    pub id: AssetVersionId,
    pub asset_id: AssetId,
    pub parent_version_id: Option<AssetVersionId>,
    pub generation_event_id: Option<GenerationEventId>,
    pub version_number: u32,
    pub file_path: PathBuf,
    pub checksum_algorithm: String,
    pub checksum: String,
    pub mime_type: String,
}

impl AssetVersion {
    pub fn from_summary(summary: VersionSummary) -> Self {
        Self {
            id: summary.id,
            asset_id: summary.asset_id,
            parent_version_id: summary.parent_version_id,
            generation_event_id: summary.generation_event_id,
            version_number: summary.version_number,
            file_path: summary.file_path,
            checksum_algorithm: summary.checksum_algorithm,
            checksum: summary.checksum,
            mime_type: summary.mime_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceSourceKind {
    SameAssetParent,
    CrossAssetReference,
}
