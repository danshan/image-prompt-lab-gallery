use super::{
    assets::{ensure_asset_exists, load_asset_summary},
    database_error, serialization_error,
    storage::timestamp_string,
    LocalLibraryService,
};
use crate::{
    AssetId, AssetSummary, DomainError, DomainResult, LibraryId, MetadataReviewService,
    MetadataSuggestion, MetadataSuggestionId, ReviewMetadataSuggestionRequest,
};
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

impl MetadataReviewService for LocalLibraryService {
    fn create_suggestion(
        &self,
        request: crate::CreateMetadataSuggestionRequest,
    ) -> DomainResult<MetadataSuggestion> {
        let connection = Self::open_library_database(&request.library_path)?;
        ensure_asset_exists(&connection, &request.asset_id)?;

        let suggestion_id = MetadataSuggestionId(Uuid::new_v4().to_string());
        let tags_json =
            serde_json::to_string(&request.suggested_tags).map_err(serialization_error)?;
        let now = timestamp_string();

        connection
            .execute(
                "
                INSERT INTO metadata_suggestions (
                    id, asset_id, source, suggested_title, suggested_description,
                    suggested_tags_json, suggested_category, confidence_json,
                    status, created_at, reviewed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending_review', ?9, NULL)
                ",
                params![
                    suggestion_id.0,
                    request.asset_id.0,
                    request.source,
                    request.suggested_title,
                    request.suggested_description,
                    tags_json,
                    request.suggested_category,
                    request.confidence_json,
                    now
                ],
            )
            .map_err(database_error)?;

        Ok(MetadataSuggestion {
            id: suggestion_id,
            asset_id: request.asset_id,
            suggested_title: request.suggested_title,
            suggested_description: request.suggested_description,
            suggested_tags: request.suggested_tags,
            suggested_category: request.suggested_category,
            confidence_json: request.confidence_json,
            status: "pending_review".to_string(),
        })
    }

    fn list_pending(
        &self,
        library_path: &Path,
        library_id: &LibraryId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        let connection = Self::open_library_database(library_path)?;
        let mut statement = connection
            .prepare(
                "
                SELECT ms.id, ms.asset_id, ms.suggested_title, ms.suggested_description,
                       ms.suggested_tags_json, ms.suggested_category, ms.confidence_json, ms.status
                FROM metadata_suggestions ms
                INNER JOIN assets a ON a.id = ms.asset_id
                WHERE a.library_id = ?1 AND ms.status = 'pending_review'
                ORDER BY ms.created_at
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![library_id.0], metadata_suggestion_from_row)
            .map_err(database_error)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn accept(&self, request: ReviewMetadataSuggestionRequest) -> DomainResult<AssetSummary> {
        let connection = Self::open_library_database(&request.library_path)?;
        let suggestion = load_suggestion(&connection, &request.suggestion_id)?;
        let now = timestamp_string();
        let transaction = connection.unchecked_transaction().map_err(database_error)?;

        transaction
            .execute(
                "
                UPDATE assets
                SET title = ?1, description = ?2, category = ?3, updated_at = ?4
                WHERE id = ?5
                ",
                params![
                    request.title,
                    request.description,
                    request.category,
                    now,
                    suggestion.asset_id.0
                ],
            )
            .map_err(database_error)?;

        for tag in request.tags {
            attach_tag(
                &transaction,
                &suggestion.asset_id,
                &tag,
                "metadata_review",
                &now,
            )?;
        }

        transaction
            .execute(
                "UPDATE metadata_suggestions SET status = 'accepted', reviewed_at = ?1 WHERE id = ?2",
                params![now, request.suggestion_id.0],
            )
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)?;

        load_asset_summary(&connection, &suggestion.asset_id)
    }

    fn reject(
        &self,
        library_path: &Path,
        suggestion_id: &MetadataSuggestionId,
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(library_path)?;
        let updated = connection
            .execute(
                "UPDATE metadata_suggestions SET status = 'rejected', reviewed_at = ?1 WHERE id = ?2",
                params![timestamp_string(), suggestion_id.0],
            )
            .map_err(database_error)?;

        if updated == 0 {
            return Err(DomainError::InvalidAssetReference {
                id: suggestion_id.0.clone(),
            });
        }

        Ok(())
    }
}

pub(super) fn attach_tag(
    connection: &Connection,
    asset_id: &AssetId,
    tag: &str,
    source: &str,
    confirmed_at: &str,
) -> DomainResult<()> {
    let tag_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "
            INSERT INTO tags (id, name, color, created_at)
            VALUES (?1, ?2, NULL, ?3)
            ON CONFLICT(name) DO NOTHING
            ",
            params![tag_id, tag, confirmed_at],
        )
        .map_err(database_error)?;
    let existing_tag_id: String = connection
        .query_row("SELECT id FROM tags WHERE name = ?1", params![tag], |row| {
            row.get(0)
        })
        .map_err(database_error)?;
    connection
        .execute(
            "
            INSERT INTO asset_tags (asset_id, tag_id, source, confirmed_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(asset_id, tag_id) DO UPDATE SET confirmed_at = excluded.confirmed_at
            ",
            params![asset_id.0, existing_tag_id, source, confirmed_at],
        )
        .map_err(database_error)?;
    Ok(())
}

fn metadata_suggestion_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MetadataSuggestion> {
    let tags_json: String = row.get(4)?;
    let suggested_tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(MetadataSuggestion {
        id: MetadataSuggestionId(row.get(0)?),
        asset_id: AssetId(row.get(1)?),
        suggested_title: row.get(2)?,
        suggested_description: row.get(3)?,
        suggested_tags,
        suggested_category: row.get(5)?,
        confidence_json: row.get(6)?,
        status: row.get(7)?,
    })
}

fn load_suggestion(
    connection: &Connection,
    suggestion_id: &MetadataSuggestionId,
) -> DomainResult<MetadataSuggestion> {
    connection
        .query_row(
            "
            SELECT id, asset_id, suggested_title, suggested_description,
                   suggested_tags_json, suggested_category, confidence_json, status
            FROM metadata_suggestions
            WHERE id = ?1
            ",
            params![suggestion_id.0],
            metadata_suggestion_from_row,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => DomainError::InvalidAssetReference {
                id: suggestion_id.0.clone(),
            },
            other => database_error(other),
        })
}
