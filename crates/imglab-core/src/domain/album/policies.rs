use crate::{AlbumKind, DomainError, DomainResult};

pub const MANUAL_ALBUM_KIND: &str = "manual";
pub const SMART_ALBUM_KIND: &str = "smart";
pub const SMART_QUERY_FIELDS: &[&str] = &[
    "text",
    "tags",
    "providers",
    "minRating",
    "reviewStatus",
    "category",
    "status",
    "createdAtFrom",
    "createdAtTo",
    "sort",
];

pub fn album_kind_to_str(kind: AlbumKind) -> &'static str {
    match kind {
        AlbumKind::Manual => MANUAL_ALBUM_KIND,
        AlbumKind::Smart => SMART_ALBUM_KIND,
    }
}

pub fn album_kind_from_str(kind: &str) -> AlbumKind {
    if kind == SMART_ALBUM_KIND {
        AlbumKind::Smart
    } else {
        AlbumKind::Manual
    }
}

pub fn ensure_manual_album_kind(kind: &str) -> DomainResult<()> {
    if kind != MANUAL_ALBUM_KIND {
        return Err(DomainError::InvalidSmartAlbumQuery {
            message: "manual album operation cannot be applied to smart album".to_string(),
        });
    }
    Ok(())
}

pub fn ensure_supported_smart_query_field(field: &str) -> DomainResult<()> {
    if !SMART_QUERY_FIELDS.contains(&field) {
        return Err(DomainError::InvalidSmartAlbumQuery {
            message: format!("unsupported smart query field: {field}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_kind_round_trips_storage_values() {
        assert_eq!(album_kind_to_str(AlbumKind::Manual), "manual");
        assert_eq!(album_kind_to_str(AlbumKind::Smart), "smart");
        assert_eq!(album_kind_from_str("manual"), AlbumKind::Manual);
        assert_eq!(album_kind_from_str("smart"), AlbumKind::Smart);
    }

    #[test]
    fn manual_operations_reject_smart_albums() {
        let error = ensure_manual_album_kind("smart").expect_err("smart album");
        assert!(matches!(error, DomainError::InvalidSmartAlbumQuery { .. }));
    }

    #[test]
    fn smart_query_fields_are_allowlisted() {
        assert!(ensure_supported_smart_query_field("providers").is_ok());
        assert!(matches!(
            ensure_supported_smart_query_field("unexpected"),
            Err(DomainError::InvalidSmartAlbumQuery { .. })
        ));
    }
}
