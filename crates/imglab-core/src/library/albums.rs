use super::{
    assets::{ensure_asset_exists, load_asset_summary},
    database_error,
    gallery::validate_rating_range,
    serialization_error,
    storage::timestamp_string,
    LocalLibraryService,
};
use crate::{
    AlbumId, AlbumKind, AlbumListItem, AlbumService, AlbumSummary, AssetId, DomainError,
    DomainResult, LibraryId, LibraryService, UpdateAssetMetadataRequest,
};
use rusqlite::{params, Connection};
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
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| {
                Self::open_library_database(&library.root_path)
                    .and_then(|connection| {
                        let count: i64 = connection
                            .query_row(
                                "SELECT COUNT(*) FROM albums WHERE id = ?1",
                                params![album_id.0],
                                |row| row.get(0),
                            )
                            .map_err(database_error)?;
                        Ok(count > 0)
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| DomainError::InvalidAssetReference {
                id: album_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let sort_order: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM album_items WHERE album_id = ?1",
                params![album_id.0],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        connection
            .execute(
                "
                INSERT INTO album_items (album_id, asset_id, sort_order, added_at)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(album_id, asset_id) DO NOTHING
                ",
                params![album_id.0, asset_id.0, sort_order, timestamp_string()],
            )
            .map_err(database_error)?;
        Ok(())
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

fn list_albums(connection: &Connection) -> DomainResult<Vec<AlbumListItem>> {
    let mut statement = connection
        .prepare(
            "
            SELECT a.id, a.name, a.kind, COUNT(ai.asset_id) AS item_count
            FROM albums a
            LEFT JOIN album_items ai ON ai.album_id = a.id
            GROUP BY a.id, a.name, a.kind
            ORDER BY a.name
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
    let kind_str = match kind {
        AlbumKind::Manual => "manual",
        AlbumKind::Smart => "smart",
    };
    connection
        .execute(
            "
            INSERT INTO albums (id, name, description, kind, smart_query_json, created_at, updated_at)
            VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?5)
            ",
            params![album_id.0, name, kind_str, smart_query_json, now],
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
    let value: serde_json::Value = serde_json::from_str(query_json).map_err(serialization_error)?;
    let object = value
        .as_object()
        .ok_or_else(|| DomainError::InvalidSmartAlbumQuery {
            message: "smart query must be a JSON object".to_string(),
        })?;
    const ALLOWED: &[&str] = &[
        "tags", "rating", "provider", "date", "status", "category", "text",
    ];
    for key in object.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            return Err(DomainError::InvalidSmartAlbumQuery {
                message: format!("unsupported smart query field: {key}"),
            });
        }
    }
    Ok(())
}
