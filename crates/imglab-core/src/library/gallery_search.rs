use super::gallery::load_gallery_asset_views;
use crate::{AssetSummary, DomainResult, GalleryAssetView, SearchQuery};
use rusqlite::Connection;

pub(super) fn search_assets(
    connection: &Connection,
    query: SearchQuery,
) -> DomainResult<Vec<AssetSummary>> {
    let mut items = load_gallery_asset_views(connection)?;
    if let Some(text) = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        let needle = text.to_ascii_lowercase();
        items.retain(|item| search_item_matches_text(item, &needle));
    }
    if let Some(provider) = query.provider {
        items.retain(|item| item.provider.as_deref() == Some(provider.as_str()));
    }
    if let Some(min_rating) = query.min_rating {
        items.retain(|item| item.rating.unwrap_or_default() >= min_rating);
    }
    if let Some(status) = query.status {
        items.retain(|item| item.status == status);
    }
    if let Some(category) = query.category {
        items.retain(|item| item.category.as_deref() == Some(category.as_str()));
    }
    if !query.tags.is_empty() {
        let wanted = query.tags;
        items.retain(|item| wanted.iter().all(|tag| item.tags.contains(tag)));
    }
    Ok(items
        .into_iter()
        .map(|item| AssetSummary {
            id: item.id,
            title: item.title,
            category: item.category,
            rating: item.rating,
            status: item.status,
        })
        .collect())
}

fn search_item_matches_text(item: &GalleryAssetView, needle: &str) -> bool {
    super::gallery::gallery_item_matches_text(item, needle)
        || item.status.to_ascii_lowercase().contains(needle)
}
