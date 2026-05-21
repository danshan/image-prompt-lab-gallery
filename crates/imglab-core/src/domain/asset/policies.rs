use crate::{AssetId, DomainError, DomainResult};

pub fn version_name(version_number: u32) -> String {
    format!("v{version_number}")
}

pub fn next_version_number(current_max_version_number: Option<u32>) -> u32 {
    current_max_version_number.unwrap_or(0) + 1
}

pub fn classify_reference_source(
    output_asset_id: &AssetId,
    input_asset_id: &AssetId,
) -> super::ReferenceSourceKind {
    if output_asset_id == input_asset_id {
        super::ReferenceSourceKind::SameAssetParent
    } else {
        super::ReferenceSourceKind::CrossAssetReference
    }
}

pub fn ensure_same_asset_parent(
    output_asset_id: &AssetId,
    input_asset_id: &AssetId,
) -> DomainResult<()> {
    if output_asset_id != input_asset_id {
        return Err(DomainError::InvalidAssetReference {
            id: input_asset_id.0.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_id(value: &str) -> AssetId {
        AssetId(value.to_string())
    }

    #[test]
    fn version_names_are_numeric_and_user_facing() {
        assert_eq!(version_name(1), "v1");
        assert_eq!(version_name(42), "v42");
    }

    #[test]
    fn next_version_number_starts_at_one() {
        assert_eq!(next_version_number(None), 1);
        assert_eq!(next_version_number(Some(3)), 4);
    }

    #[test]
    fn reference_source_classification_separates_cross_asset_inputs() {
        assert_eq!(
            classify_reference_source(&asset_id("asset-1"), &asset_id("asset-1")),
            super::super::ReferenceSourceKind::SameAssetParent
        );
        assert_eq!(
            classify_reference_source(&asset_id("asset-1"), &asset_id("asset-2")),
            super::super::ReferenceSourceKind::CrossAssetReference
        );
    }

    #[test]
    fn same_asset_parent_rejects_cross_asset_inputs() {
        let error = ensure_same_asset_parent(&asset_id("asset-1"), &asset_id("asset-2"))
            .expect_err("cross asset");
        assert!(matches!(error, DomainError::InvalidAssetReference { .. }));
    }
}
