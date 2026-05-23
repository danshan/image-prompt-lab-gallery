use super::{
    gallery_cards::load_gallery_asset_views,
    gallery_filtering::{
        apply_gallery_album_filter, apply_gallery_filter_spec, effective_gallery_album_filter,
        load_album_filter_contexts, load_album_membership_context, single_album_filter_context,
        single_album_filter_id, sort_gallery_items, validate_gallery_album_context,
        AlbumFilterContext, GalleryFilterSpec,
    },
    LocalLibraryService,
};
use crate::{
    AssetId, AssetSummary, AssetVersionId, DomainError, DomainResult, GalleryAssetView,
    GalleryQuery, GalleryReadService, LibraryService, SearchQuery, SearchService,
};
use std::path::Path;

impl SearchService for LocalLibraryService {
    fn search(
        &self,
        library_id: &crate::LibraryId,
        query: SearchQuery,
    ) -> DomainResult<Vec<AssetSummary>> {
        let library = self
            .list_libraries(true)?
            .into_iter()
            .find(|library| library.id == *library_id)
            .ok_or_else(|| DomainError::LibraryNotFound {
                path: library_id.0.clone(),
            })?;
        let connection = Self::open_library_database(&library.root_path)?;
        super::gallery_search::search_assets(&connection, query)
    }
}

impl GalleryReadService for LocalLibraryService {
    fn query_gallery(
        &self,
        library_path: &Path,
        query: GalleryQuery,
    ) -> DomainResult<Vec<GalleryAssetView>> {
        validate_gallery_query(&query)?;
        let connection = Self::open_library_database(library_path)?;
        let album_filter = effective_gallery_album_filter(&query);
        let album_contexts = load_album_filter_contexts(&connection, &album_filter)?;
        validate_gallery_album_context(&query, &album_filter, &album_contexts)?;
        let single_album_context = single_album_filter_context(&album_filter, &album_contexts);
        let album_context_view = single_album_filter_id(&album_filter)
            .map(|album_id| load_album_membership_context(&connection, album_id))
            .transpose()?;
        let mut items = load_gallery_asset_views(&connection)?;

        apply_gallery_filter_spec(&mut items, GalleryFilterSpec::from_gallery_query(&query));
        apply_gallery_album_filter(&connection, &mut items, &album_filter, &album_contexts)?;

        let effective_sort = match single_album_context {
            Some(AlbumFilterContext::Smart(smart_query)) => smart_query.sort.unwrap_or(query.sort),
            _ => query.sort,
        };

        sort_gallery_items(
            &mut items,
            effective_sort,
            single_album_context,
            &connection,
        )?;

        if let Some(album) = album_context_view {
            for item in &mut items {
                item.album_context = Some(album.clone());
            }
        }

        Ok(items)
    }

    fn get_asset_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<crate::AssetDetailView> {
        let connection = Self::open_library_database(library_path)?;
        super::gallery_detail::get_asset_detail(
            library_path,
            &connection,
            asset_id,
            current_version_id,
        )
    }

    fn get_asset_inspector_detail(
        &self,
        library_path: &Path,
        asset_id: &AssetId,
        current_version_id: Option<&AssetVersionId>,
    ) -> DomainResult<crate::AssetInspectorDetailView> {
        let connection = Self::open_library_database(library_path)?;
        super::gallery_detail::get_asset_inspector_detail(
            library_path,
            &connection,
            asset_id,
            current_version_id,
        )
    }
}

pub(super) fn validate_rating_range(
    value: u8,
    field: &str,
    gallery_query: bool,
) -> DomainResult<()> {
    if (1..=5).contains(&value) {
        return Ok(());
    }

    let message = format!("{field} must be between 1 and 5");
    if gallery_query {
        Err(DomainError::InvalidGalleryQuery { message })
    } else {
        Err(DomainError::InvalidGenerationParameters { message })
    }
}

fn validate_gallery_query(query: &GalleryQuery) -> DomainResult<()> {
    if let Some(min_rating) = query.min_rating {
        validate_rating_range(min_rating, "min_rating", true)?;
    }
    Ok(())
}
