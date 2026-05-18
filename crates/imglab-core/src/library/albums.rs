use super::{
    assets::{ensure_asset_exists, load_asset_summary},
    database_error,
    gallery::validate_rating_range,
    serialization_error,
    storage::timestamp_string,
    LocalLibraryService,
};
use crate::{
    AlbumId, AlbumKind, AlbumListItem, AlbumService, AlbumSummary, AssetId,
    BatchAddAssetsToAlbumRequest, DomainError, DomainResult, GallerySort, LibraryId,
    LibraryService, ReorderAlbumItemsRequest, ReorderAlbumsRequest, ReviewStatusFilter,
    SmartAlbumQuery, UpdateAssetMetadataRequest,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use uuid::Uuid;

impl AlbumService for LocalLibraryService {
    fn list_albums(&self, library_id: &LibraryId) -> DomainResult<Vec<AlbumListItem>> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        list_albums(&connection)
    }

    fn create_manual_album(
        &self,
        library_id: &LibraryId,
        name: &str,
    ) -> DomainResult<AlbumSummary> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        create_album(&connection, name, AlbumKind::Manual, None)
    }

    fn create_smart_album(
        &self,
        request: crate::CreateSmartAlbumRequest,
    ) -> DomainResult<AlbumSummary> {
        validate_smart_query(&request.smart_query_json)?;
        let connection = Self::open_library_database(&request.library_path)?;
        create_album(
            &connection,
            &request.name,
            AlbumKind::Smart,
            Some(request.smart_query_json),
        )
    }

    fn add_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        let library = self.find_library_containing_album(album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        add_assets_to_manual_album(&connection, album_id, std::slice::from_ref(asset_id))
    }

    fn batch_add_assets(&self, request: BatchAddAssetsToAlbumRequest) -> DomainResult<()> {
        let library = self.find_library_containing_album(&request.album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        add_assets_to_manual_album(&connection, &request.album_id, &request.asset_ids)
    }

    fn remove_asset(&self, album_id: &AlbumId, asset_id: &AssetId) -> DomainResult<()> {
        let library = self.find_library_containing_album(album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        ensure_manual_album(&connection, album_id)?;
        let updated = connection
            .execute(
                "DELETE FROM album_items WHERE album_id = ?1 AND asset_id = ?2",
                params![album_id.0, asset_id.0],
            )
            .map_err(database_error)?;
        if updated == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: asset_id.0.clone(),
            });
        }
        Ok(())
    }

    fn rename_album(&self, album_id: &AlbumId, name: &str) -> DomainResult<AlbumSummary> {
        let library = self.find_library_containing_album(album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        let updated = connection
            .execute(
                "UPDATE albums SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, timestamp_string(), album_id.0],
            )
            .map_err(database_error)?;
        if updated == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: album_id.0.clone(),
            });
        }
        load_album_summary(&connection, album_id)
    }

    fn delete_album(&self, album_id: &AlbumId) -> DomainResult<()> {
        let library = self.find_library_containing_album(album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        transaction
            .execute(
                "DELETE FROM album_items WHERE album_id = ?1",
                params![album_id.0],
            )
            .map_err(database_error)?;
        let updated = transaction
            .execute("DELETE FROM albums WHERE id = ?1", params![album_id.0])
            .map_err(database_error)?;
        if updated == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: album_id.0.clone(),
            });
        }
        transaction.commit().map_err(database_error)
    }

    fn reorder_albums(&self, request: ReorderAlbumsRequest) -> DomainResult<()> {
        let connection = Self::open_library_database(&request.library_path)?;
        let existing = album_ids(&connection)?;
        if existing.len() != request.album_ids.len()
            || !request.album_ids.iter().all(|id| existing.contains(id))
        {
            return Err(DomainError::InvalidAssetReference {
                id: "album order must contain every album exactly once".to_string(),
            });
        }
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        for (index, album_id) in request.album_ids.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE albums SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                    params![index as i64 + 1, timestamp_string(), album_id.0],
                )
                .map_err(database_error)?;
        }
        transaction.commit().map_err(database_error)
    }

    fn reorder_album_items(&self, request: ReorderAlbumItemsRequest) -> DomainResult<()> {
        let library = self.find_library_containing_album(&request.album_id)?;
        let connection = Self::open_library_database(&library.root_path)?;
        ensure_manual_album(&connection, &request.album_id)?;
        let existing = album_item_asset_ids(&connection, &request.album_id)?;
        if existing.len() != request.asset_ids.len()
            || !request.asset_ids.iter().all(|id| existing.contains(id))
        {
            return Err(DomainError::InvalidAssetReference {
                id: "album item order must contain every album asset exactly once".to_string(),
            });
        }
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        for (index, asset_id) in request.asset_ids.iter().enumerate() {
            transaction
                .execute(
                    "
                    UPDATE album_items
                    SET sort_order = ?1
                    WHERE album_id = ?2 AND asset_id = ?3
                    ",
                    params![index as i64 + 1, request.album_id.0, asset_id.0],
                )
                .map_err(database_error)?;
        }
        transaction.commit().map_err(database_error)
    }

    fn update_asset_metadata(
        &self,
        request: UpdateAssetMetadataRequest,
    ) -> DomainResult<crate::AssetSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_asset_exists(&connection, &request.asset_id)?;
        if let Some(rating) = request.rating {
            validate_rating_range(rating, "rating", false)?;
        }

        connection
            .execute(
                "
                UPDATE assets
                SET title = COALESCE(?1, title),
                    description = COALESCE(?2, description),
                    schema_prompt = COALESCE(?3, schema_prompt),
                    rating = COALESCE(?4, rating),
                    category = COALESCE(?5, category),
                    status = COALESCE(?6, status),
                    updated_at = ?7
                WHERE id = ?8
                ",
                params![
                    request.title,
                    request.description,
                    request.schema_prompt,
                    request.rating,
                    request.category,
                    request.status,
                    timestamp_string(),
                    request.asset_id.0
                ],
            )
            .map_err(database_error)?;
        load_asset_summary(&connection, &request.asset_id)
    }
}

impl LocalLibraryService {
    fn find_library_containing_album(
        &self,
        album_id: &AlbumId,
    ) -> DomainResult<crate::LibrarySummary> {
        self.list_libraries(true)?
            .into_iter()
            .find(|library| {
                Self::open_library_database(&library.root_path)
                    .and_then(|connection| album_exists(&connection, album_id))
                    .unwrap_or(false)
            })
            .ok_or_else(|| DomainError::InvalidAssetReference {
                id: album_id.0.clone(),
            })
    }
}

fn list_albums(connection: &Connection) -> DomainResult<Vec<AlbumListItem>> {
    let mut statement = connection
        .prepare(
            "
            SELECT a.id, a.name, a.kind, COUNT(ai.asset_id) AS item_count, a.sort_order
            FROM albums a
            LEFT JOIN album_items ai ON ai.album_id = a.id
            GROUP BY a.id, a.name, a.kind, a.sort_order
            ORDER BY a.sort_order, a.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let kind: String = row.get(2)?;
            let count: i64 = row.get(3)?;
            Ok(AlbumListItem {
                id: AlbumId(row.get(0)?),
                name: row.get(1)?,
                kind: album_kind_from_str(&kind),
                item_count: count.max(0) as u32,
                sort_order: row.get(4)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn create_album(
    connection: &Connection,
    name: &str,
    kind: AlbumKind,
    smart_query_json: Option<String>,
) -> DomainResult<AlbumSummary> {
    let album_id = AlbumId(Uuid::new_v4().to_string());
    let now = timestamp_string();
    let sort_order: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM albums",
            [],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    let kind_str = match kind {
        AlbumKind::Manual => "manual",
        AlbumKind::Smart => "smart",
    };
    connection
        .execute(
            "
            INSERT INTO albums (id, name, description, kind, smart_query_json, sort_order, created_at, updated_at)
            VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?6)
            ",
            params![album_id.0, name, kind_str, smart_query_json, sort_order, now],
        )
        .map_err(database_error)?;
    Ok(AlbumSummary {
        id: album_id,
        name: name.to_string(),
        kind,
    })
}

fn album_kind_from_str(kind: &str) -> AlbumKind {
    if kind == "smart" {
        AlbumKind::Smart
    } else {
        AlbumKind::Manual
    }
}

fn validate_smart_query(query_json: &str) -> DomainResult<()> {
    let _ = parse_smart_query(query_json)?;
    Ok(())
}

pub(super) fn parse_smart_query(query_json: &str) -> DomainResult<SmartAlbumQuery> {
    let value: Value = serde_json::from_str(query_json).map_err(serialization_error)?;
    let object = value
        .as_object()
        .ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: "smart query must be a JSON object".to_string(),
        })?;
    const ALLOWED: &[&str] = &[
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
    for key in object.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            return Err(DomainError::InvalidSmartAlbumQuery {
                message: format!("unsupported smart query field: {key}"),
            });
        }
    }
    let query = SmartAlbumQuery {
        text: optional_string(object.get("text"), "text")?,
        tags: optional_string_array(object.get("tags"), "tags")?,
        providers: optional_string_array(object.get("providers"), "providers")?,
        min_rating: optional_u8(object.get("minRating"), "minRating")?,
        review_status: optional_review_status(object.get("reviewStatus"))?.unwrap_or_default(),
        category: optional_string(object.get("category"), "category")?,
        status: optional_string(object.get("status"), "status")?,
        created_at_from: optional_timestamp(object.get("createdAtFrom"), "createdAtFrom")?,
        created_at_to: optional_timestamp(object.get("createdAtTo"), "createdAtTo")?,
        sort: optional_gallery_sort(object.get("sort"))?,
    };
    if let (Some(from), Some(to)) = (&query.created_at_from, &query.created_at_to) {
        if from.parse::<u128>().unwrap_or_default() > to.parse::<u128>().unwrap_or_default() {
            return Err(DomainError::InvalidSmartAlbumQuery {
                message: "createdAtFrom must be less than or equal to createdAtTo".to_string(),
            });
        }
    }
    if let Some(min_rating) = query.min_rating {
        validate_rating_range(min_rating, "minRating", false)?;
    }
    Ok(query)
}

fn optional_string(value: Option<&Value>, field: &str) -> DomainResult<Option<String>> {
    value
        .map(|value| {
            value.as_str().map(ToString::to_string).ok_or_else(|| {
                DomainError::InvalidSmartAlbumQuery {
                    message: format!("{field} must be a string"),
                }
            })
        })
        .transpose()
}

fn optional_string_array(value: Option<&Value>, field: &str) -> DomainResult<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: format!("{field} must be an array"),
        })?;
    array
        .iter()
        .map(|item| {
            item.as_str().map(ToString::to_string).ok_or_else(|| {
                DomainError::InvalidSmartAlbumQuery {
                    message: format!("{field} values must be strings"),
                }
            })
        })
        .collect()
}

fn optional_u8(value: Option<&Value>, field: &str) -> DomainResult<Option<u8>> {
    value
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
                    message: format!("{field} must be an integer"),
                })
        })
        .transpose()
}

fn optional_timestamp(value: Option<&Value>, field: &str) -> DomainResult<Option<String>> {
    let Some(value) = optional_string(value, field)? else {
        return Ok(None);
    };
    if value.parse::<u128>().is_err() {
        return Err(DomainError::InvalidSmartAlbumQuery {
            message: format!("{field} must be a unix timestamp in milliseconds"),
        });
    }
    Ok(Some(value))
}

fn optional_review_status(value: Option<&Value>) -> DomainResult<Option<ReviewStatusFilter>> {
    optional_string(value, "reviewStatus")?
        .map(|value| match value.as_str() {
            "any" => Ok(ReviewStatusFilter::Any),
            "pending" | "pending_review" => Ok(ReviewStatusFilter::Pending),
            other => Err(DomainError::InvalidSmartAlbumQuery {
                message: format!("unsupported reviewStatus: {other}"),
            }),
        })
        .transpose()
}

fn optional_gallery_sort(value: Option<&Value>) -> DomainResult<Option<GallerySort>> {
    optional_string(value, "sort")?
        .map(|value| match value.as_str() {
            "newest" => Ok(GallerySort::Newest),
            "oldest" => Ok(GallerySort::Oldest),
            "rating_desc" | "ratingDesc" => Ok(GallerySort::RatingDesc),
            "title_asc" | "titleAsc" => Ok(GallerySort::TitleAsc),
            "provider_asc" | "providerAsc" => Ok(GallerySort::ProviderAsc),
            "album_order" | "albumOrder" => Ok(GallerySort::AlbumOrder),
            other => Err(DomainError::InvalidSmartAlbumQuery {
                message: format!("unsupported sort: {other}"),
            }),
        })
        .transpose()
}

fn album_exists(connection: &Connection, album_id: &AlbumId) -> DomainResult<bool> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM albums WHERE id = ?1",
            params![album_id.0],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    Ok(count > 0)
}

fn load_album_summary(connection: &Connection, album_id: &AlbumId) -> DomainResult<AlbumSummary> {
    connection
        .query_row(
            "SELECT id, name, kind FROM albums WHERE id = ?1",
            params![album_id.0],
            |row| {
                let kind: String = row.get(2)?;
                Ok(AlbumSummary {
                    id: AlbumId(row.get(0)?),
                    name: row.get(1)?,
                    kind: album_kind_from_str(&kind),
                })
            },
        )
        .map_err(database_error)
}

fn ensure_manual_album(connection: &Connection, album_id: &AlbumId) -> DomainResult<()> {
    let kind = connection
        .query_row(
            "SELECT kind FROM albums WHERE id = ?1",
            params![album_id.0],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| DomainError::InvalidAssetReference {
            id: album_id.0.clone(),
        })?;
    if kind != "manual" {
        return Err(DomainError::InvalidSmartAlbumQuery {
            message: "manual album operation cannot be applied to smart album".to_string(),
        });
    }
    Ok(())
}

fn add_assets_to_manual_album(
    connection: &Connection,
    album_id: &AlbumId,
    asset_ids: &[AssetId],
) -> DomainResult<()> {
    ensure_manual_album(connection, album_id)?;
    for asset_id in asset_ids {
        ensure_asset_exists(connection, asset_id)?;
    }
    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    let now = timestamp_string();
    for asset_id in asset_ids {
        let sort_order: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM album_items WHERE album_id = ?1",
                params![album_id.0],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        transaction
            .execute(
                "
                INSERT INTO album_items (album_id, asset_id, sort_order, added_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(album_id, asset_id) DO NOTHING
                ",
                params![album_id.0, asset_id.0, sort_order, now],
            )
            .map_err(database_error)?;
    }
    transaction.commit().map_err(database_error)
}

fn album_ids(connection: &Connection) -> DomainResult<Vec<AlbumId>> {
    let mut statement = connection
        .prepare("SELECT id FROM albums ORDER BY sort_order, name")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| Ok(AlbumId(row.get(0)?)))
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn album_item_asset_ids(connection: &Connection, album_id: &AlbumId) -> DomainResult<Vec<AssetId>> {
    let mut statement = connection
        .prepare("SELECT asset_id FROM album_items WHERE album_id = ?1 ORDER BY sort_order")
        .map_err(database_error)?;
    let rows = statement
        .query_map(params![album_id.0], |row| Ok(AssetId(row.get(0)?)))
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}
