use super::{
    database_error, gallery_task_origin::load_task_origins,
    gallery_version_tree::load_asset_version_tree_summaries,
};
use crate::{
    version_name, AlbumId, AlbumKind, AlbumMembershipView, AssetId, AssetVersionId, DomainResult,
    GalleryAssetView, GenerationEventId,
};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn load_gallery_asset_views(
    connection: &Connection,
) -> DomainResult<Vec<GalleryAssetView>> {
    let versions = load_latest_asset_versions(connection)?;
    let tree_summaries = load_asset_version_tree_summaries(connection)?;
    let events = load_gallery_events(connection)?;
    let version_counts = load_asset_version_counts(connection)?;
    let tags = load_all_asset_tags(connection)?;
    let review_counts = load_pending_review_counts(connection)?;
    let task_origins = load_task_origins(connection)?;
    let albums = load_all_asset_album_memberships(connection)?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, title, category, rating, status, created_at, updated_at
            FROM assets
            WHERE status <> 'reference'
            ORDER BY updated_at DESC, created_at DESC
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                AssetId(row.get::<_, String>(0)?),
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(database_error)?;

    let mut items = Vec::new();
    for row in rows {
        let (id, title, category, rating, status, created_at, updated_at) =
            row.map_err(database_error)?;
        let current_version = versions.get(&id.0);
        let tree_summary = tree_summaries.get(&id.0);
        let event = current_version
            .and_then(|version| version.generation_event_id.as_ref())
            .and_then(|event_id| events.by_id.get(&event_id.0))
            .or_else(|| events.latest_by_asset.get(&id.0));
        let task_origin = current_version
            .and_then(|version| task_origins.by_version.get(&version.id.0))
            .or_else(|| task_origins.by_asset.get(&id.0))
            .cloned();

        items.push(GalleryAssetView {
            id: id.clone(),
            title,
            category,
            rating,
            status,
            provider: event.as_ref().map(|event| event.provider.clone()),
            model_label: event.as_ref().map(|event| event.model_label.clone()),
            prompt: event.as_ref().map(|event| event.prompt.clone()),
            tags: tags.get(&id.0).cloned().unwrap_or_default(),
            review_pending_count: review_counts.get(&id.0).copied().unwrap_or_default(),
            current_version_id: current_version.as_ref().map(|version| version.id.clone()),
            current_version_number: current_version.map(|version| version.version_number),
            current_version_name: current_version.map(|version| version.version_name.clone()),
            current_version_tree_name: current_version
                .and_then(|version| {
                    tree_summary.and_then(|summary| summary.names_by_id.get(&version.id.0))
                })
                .cloned()
                .or_else(|| current_version.map(|version| version.version_name.clone())),
            image_path: current_version
                .as_ref()
                .map(|version| version.file_path.clone()),
            width: current_version.and_then(|version| version.width),
            height: current_version.and_then(|version| version.height),
            version_label: current_version.and_then(|version| version.version_label.clone()),
            version_count: version_counts.get(&id.0).copied().unwrap_or_default(),
            version_tree_branch_count: tree_summary
                .map(|summary| summary.branch_count)
                .unwrap_or_default(),
            task_origin,
            albums: albums.get(&id.0).cloned().unwrap_or_default(),
            album_context: None,
            created_at,
            updated_at,
        });
    }

    Ok(items)
}

#[derive(Debug, Clone)]
struct LatestVersionView {
    id: AssetVersionId,
    generation_event_id: Option<GenerationEventId>,
    file_path: PathBuf,
    width: Option<u32>,
    height: Option<u32>,
    version_number: u32,
    version_name: String,
    version_label: Option<String>,
}

#[derive(Debug, Clone)]
struct GalleryEventView {
    provider: String,
    model_label: String,
    prompt: String,
}

#[derive(Debug, Default)]
struct GalleryEvents {
    by_id: HashMap<String, GalleryEventView>,
    latest_by_asset: HashMap<String, GalleryEventView>,
}

fn load_latest_asset_versions(
    connection: &Connection,
) -> DomainResult<HashMap<String, LatestVersionView>> {
    let mut statement = connection
        .prepare(
            "
            SELECT av.id, av.asset_id, av.generation_event_id, av.file_path,
                   av.width, av.height, av.version_number, av.version_label
            FROM asset_versions av
            WHERE NOT EXISTS (
                SELECT 1
                FROM asset_versions newer
                WHERE newer.asset_id = av.asset_id
                  AND (
                    newer.created_at > av.created_at
                    OR (newer.created_at = av.created_at AND newer.id > av.id)
                  )
            )
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let version_number: u32 = row.get(6)?;
            Ok((
                row.get::<_, String>(1)?,
                LatestVersionView {
                    id: AssetVersionId(row.get(0)?),
                    generation_event_id: row.get::<_, Option<String>>(2)?.map(GenerationEventId),
                    file_path: PathBuf::from(row.get::<_, String>(3)?),
                    width: row.get(4)?,
                    height: row.get(5)?,
                    version_number,
                    version_name: version_name(version_number),
                    version_label: row.get(7)?,
                },
            ))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn load_gallery_events(connection: &Connection) -> DomainResult<GalleryEvents> {
    let mut events = GalleryEvents::default();

    let mut by_id_statement = connection
        .prepare("SELECT id, provider, provider_model, prompt FROM generation_events")
        .map_err(database_error)?;
    let by_id_rows = by_id_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                GalleryEventView {
                    provider: row.get(1)?,
                    model_label: row.get(2)?,
                    prompt: row.get(3)?,
                },
            ))
        })
        .map_err(database_error)?;
    events.by_id = collect_string_map(by_id_rows)?;

    let mut latest_statement = connection
        .prepare(
            "
            SELECT ge.asset_id, ge.provider, ge.provider_model, ge.prompt
            FROM generation_events ge
            WHERE ge.asset_id IS NOT NULL
              AND NOT EXISTS (
                SELECT 1
                FROM generation_events newer
                WHERE newer.asset_id = ge.asset_id
                  AND (
                    newer.started_at > ge.started_at
                    OR (newer.started_at = ge.started_at AND newer.id > ge.id)
                  )
              )
            ",
        )
        .map_err(database_error)?;
    let latest_rows = latest_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                GalleryEventView {
                    provider: row.get(1)?,
                    model_label: row.get(2)?,
                    prompt: row.get(3)?,
                },
            ))
        })
        .map_err(database_error)?;
    events.latest_by_asset = collect_string_map(latest_rows)?;
    Ok(events)
}

fn load_asset_version_counts(connection: &Connection) -> DomainResult<HashMap<String, u32>> {
    let mut statement = connection
        .prepare("SELECT asset_id, COUNT(*) FROM asset_versions GROUP BY asset_id")
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count.max(0) as u32))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn load_all_asset_tags(connection: &Connection) -> DomainResult<HashMap<String, Vec<String>>> {
    let mut statement = connection
        .prepare(
            "
            SELECT at.asset_id, t.name
            FROM asset_tags at
            INNER JOIN tags t ON t.id = at.tag_id
            ORDER BY at.asset_id, t.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(database_error)?;
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (asset_id, tag) = row.map_err(database_error)?;
        tags.entry(asset_id).or_default().push(tag);
    }
    Ok(tags)
}

fn load_pending_review_counts(connection: &Connection) -> DomainResult<HashMap<String, u32>> {
    let mut statement = connection
        .prepare(
            "
            SELECT asset_id, COUNT(*)
            FROM metadata_suggestions
            WHERE status = 'pending_review'
            GROUP BY asset_id
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let count: i64 = row.get(1)?;
            Ok((row.get::<_, String>(0)?, count.max(0) as u32))
        })
        .map_err(database_error)?;
    collect_string_map(rows)
}

fn load_all_asset_album_memberships(
    connection: &Connection,
) -> DomainResult<HashMap<String, Vec<AlbumMembershipView>>> {
    let mut statement = connection
        .prepare(
            "
            SELECT ai.asset_id, a.id, a.name, a.kind
            FROM album_items ai
            INNER JOIN albums a ON a.id = ai.album_id
            ORDER BY ai.asset_id, a.name
            ",
        )
        .map_err(database_error)?;
    let rows = statement
        .query_map([], |row| {
            let kind: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                AlbumMembershipView {
                    id: AlbumId(row.get(1)?),
                    name: row.get(2)?,
                    kind: if kind == "smart" {
                        AlbumKind::Smart
                    } else {
                        AlbumKind::Manual
                    },
                },
            ))
        })
        .map_err(database_error)?;

    let mut albums: HashMap<String, Vec<AlbumMembershipView>> = HashMap::new();
    for row in rows {
        let (asset_id, album) = row.map_err(database_error)?;
        albums.entry(asset_id).or_default().push(album);
    }
    Ok(albums)
}

fn collect_string_map<T>(
    rows: impl Iterator<Item = rusqlite::Result<(String, T)>>,
) -> DomainResult<HashMap<String, T>> {
    let mut map = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(database_error)?;
        map.insert(key, value);
    }
    Ok(map)
}
