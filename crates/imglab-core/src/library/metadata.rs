use super::{
    assets::{ensure_asset_exists, load_asset_summary},
    database_error, serialization_error,
    storage::timestamp_string,
    LocalLibraryService,
};
use crate::{
    AssetId, AssetSummary, BatchReviewMetadataSuggestionRequest, ConfidenceScoreView, DomainError,
    DomainResult, LibraryId, MetadataReviewService, MetadataSuggestion, MetadataSuggestionId,
    ReviewMetadataSuggestionRequest,
};
use rusqlite::{params, Connection};
use serde_json::Value;
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
                    suggested_schema_prompt, suggested_tags_json, suggested_category, confidence_json,
                    status, created_at, reviewed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending_review', ?10, NULL)
                ",
                params![
                    suggestion_id.0,
                    request.asset_id.0,
                    request.source,
                    request.suggested_title,
                    request.suggested_description,
                    request.suggested_schema_prompt,
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
            suggested_schema_prompt: request.suggested_schema_prompt,
            suggested_tags: request.suggested_tags,
            suggested_category: request.suggested_category,
            confidence_json: request.confidence_json,
            status: "pending_review".to_string(),
            created_at: Some(now),
            reviewed_at: None,
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
                       ms.suggested_schema_prompt, ms.suggested_tags_json, ms.suggested_category,
                       ms.confidence_json, ms.status, ms.created_at, ms.reviewed_at
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
        accept_suggestions(&connection, &[request])?;
        load_asset_summary(&connection, &suggestion.asset_id)
    }

    fn batch_accept(
        &self,
        request: BatchReviewMetadataSuggestionRequest,
    ) -> DomainResult<Vec<AssetSummary>> {
        let connection = Self::open_library_database(&request.library_path)?;
        let suggestions = accept_suggestions(&connection, &request.suggestions)?;
        suggestions
            .iter()
            .map(|suggestion| load_asset_summary(&connection, &suggestion.asset_id))
            .collect()
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

    fn batch_reject(
        &self,
        library_path: &Path,
        suggestion_ids: &[MetadataSuggestionId],
    ) -> DomainResult<()> {
        let connection = Self::open_library_database(library_path)?;
        let suggestions = suggestion_ids
            .iter()
            .map(|suggestion_id| load_suggestion(&connection, suggestion_id))
            .collect::<DomainResult<Vec<_>>>()?;
        ensure_all_pending(&suggestions)?;
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        let now = timestamp_string();
        for suggestion_id in suggestion_ids {
            transaction
                .execute(
                    "UPDATE metadata_suggestions SET status = 'rejected', reviewed_at = ?1 WHERE id = ?2",
                    params![now, suggestion_id.0],
                )
                .map_err(database_error)?;
        }
        transaction.commit().map_err(database_error)
    }

    fn list_history(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
    ) -> DomainResult<Vec<MetadataSuggestion>> {
        let connection = Self::open_library_database(library_path)?;
        ensure_asset_exists(&connection, asset_id)?;
        let mut statement = connection
            .prepare(
                "
                SELECT id, asset_id, suggested_title, suggested_description,
                       suggested_schema_prompt, suggested_tags_json, suggested_category,
                       confidence_json, status, created_at, reviewed_at
                FROM metadata_suggestions
                WHERE asset_id = ?1
                ORDER BY created_at DESC, id
                ",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map(params![asset_id.0], metadata_suggestion_from_row)
            .map_err(database_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
    }

    fn normalize_confidence(&self, confidence_json: &str) -> ConfidenceScoreView {
        normalize_confidence_json(confidence_json)
    }
}

fn accept_suggestions(
    connection: &Connection,
    requests: &[ReviewMetadataSuggestionRequest],
) -> DomainResult<Vec<MetadataSuggestion>> {
    let suggestions = requests
        .iter()
        .map(|request| load_suggestion(connection, &request.suggestion_id))
        .collect::<DomainResult<Vec<_>>>()?;
    ensure_all_pending(&suggestions)?;
    for (request, suggestion) in requests.iter().zip(suggestions.iter()) {
        if let Some(category) = request.category.as_deref() {
            ensure_existing_category(connection, &suggestion.asset_id, category)?;
        }
    }
    let now = timestamp_string();
    let transaction = connection.unchecked_transaction().map_err(database_error)?;
    for (request, suggestion) in requests.iter().zip(suggestions.iter()) {
        transaction
            .execute(
                "
                UPDATE assets
                SET title = ?1, description = ?2, schema_prompt = ?3, category = ?4, updated_at = ?5
                WHERE id = ?6
                ",
                params![
                    request.title,
                    request.description,
                    request.schema_prompt,
                    request.category,
                    now,
                    suggestion.asset_id.0
                ],
            )
            .map_err(database_error)?;

        for tag in &request.tags {
            attach_tag(
                &transaction,
                &suggestion.asset_id,
                tag,
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
    }
    transaction.commit().map_err(database_error)?;
    Ok(suggestions)
}

fn ensure_all_pending(suggestions: &[MetadataSuggestion]) -> DomainResult<()> {
    for suggestion in suggestions {
        if suggestion.status != "pending_review" {
            return Err(DomainError::InvalidAssetReference {
                id: format!("suggestion is not pending: {}", suggestion.id.0),
            });
        }
    }
    Ok(())
}

fn ensure_existing_category(
    connection: &Connection,
    asset_id: &AssetId,
    category: &str,
) -> DomainResult<()> {
    let count: i64 = connection
        .query_row(
            "
            SELECT COUNT(*)
            FROM assets candidate
            INNER JOIN assets target ON target.library_id = candidate.library_id
            WHERE target.id = ?1 AND candidate.category = ?2
            ",
            params![asset_id.0, category],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    if count == 0 {
        return Err(DomainError::InvalidGenerationParameters {
            message: format!("category must already exist in this library: {category}"),
        });
    }
    Ok(())
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
    let tags_json: String = row.get(5)?;
    let suggested_tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(MetadataSuggestion {
        id: MetadataSuggestionId(row.get(0)?),
        asset_id: AssetId(row.get(1)?),
        suggested_title: row.get(2)?,
        suggested_description: row.get(3)?,
        suggested_schema_prompt: row.get(4)?,
        suggested_tags,
        suggested_category: row.get(6)?,
        confidence_json: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
        reviewed_at: row.get(10)?,
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
                   suggested_schema_prompt, suggested_tags_json, suggested_category, confidence_json, status,
                   created_at, reviewed_at
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

fn normalize_confidence_json(confidence_json: &str) -> ConfidenceScoreView {
    let Ok(value) = serde_json::from_str::<Value>(confidence_json) else {
        return ConfidenceScoreView {
            overall: None,
            title: None,
            description: None,
            schema_prompt: None,
            tags: None,
            category: None,
        };
    };
    let fields = value.get("fields");
    ConfidenceScoreView {
        overall: normalize_score(value.get("overall")),
        title: normalize_score(fields.and_then(|fields| fields.get("title"))),
        description: normalize_score(fields.and_then(|fields| fields.get("description"))),
        schema_prompt: normalize_score(fields.and_then(|fields| fields.get("schemaPrompt"))),
        tags: normalize_score(fields.and_then(|fields| fields.get("tags"))),
        category: normalize_score(fields.and_then(|fields| fields.get("category"))),
    }
}

fn normalize_score(value: Option<&Value>) -> Option<u8> {
    let value = value?.as_f64()?;
    if !(0.0..=100.0).contains(&value) {
        return None;
    }
    let normalized = if value <= 1.0 { value * 100.0 } else { value };
    Some(normalized.round().clamp(0.0, 100.0) as u8)
}
