export {
  clearGalleryAlbumFilter,
  clearGalleryMinRatingFilter,
  clearGalleryProviderFilter,
  clearGalleryReviewFilter,
  galleryAlbumFilterIds,
  removeGalleryAlbumFilter,
  resetGalleryQuery,
  setGalleryAlbumFilter,
  setGalleryUnassignedAlbumFilter,
  toggleGalleryAlbumFilter,
  updateGalleryQuery,
  type DetailLoadState,
  type GalleryQueryState,
  type GallerySort,
  type ReviewStatusFilter,
} from "../gallery/state.js";
import {
  resetGalleryQuery,
  setGalleryAlbumFilter,
  type GalleryQueryState,
} from "../gallery/state.js";
export { reorderByIds } from "../shared/state.js";

import type { AlbumListItem } from "../../types.js";

export function albumContentsQuery(
  albumId: string | null,
  albumKind: AlbumListItem["kind"] | null = "manual",
): GalleryQueryState {
  if (!albumId) {
    return resetGalleryQuery();
  }
  return {
    ...setGalleryAlbumFilter(resetGalleryQuery(), [albumId]),
    sort: albumKind === "manual" ? "albumOrder" : "newest",
  };
}

export function defaultAlbumAddSourceQuery(): GalleryQueryState {
  return resetGalleryQuery();
}

export function selectAlbumState(_current: string | null, albumId: string): string {
  return albumId;
}

export function clearSelectedAlbumState(): string | null {
  return null;
}

export function filterAlbumAddCandidates<TAsset extends { id: string; albums?: { id: string }[] }>(
  assets: TAsset[],
  albumId: string | null,
): TAsset[] {
  if (!albumId) {
    return assets;
  }
  return assets.filter((asset) => !(asset.albums ?? []).some((album) => album.id === albumId));
}

export function openAlbumQuery(_query: GalleryQueryState, albumId: string): GalleryQueryState {
  return albumContentsQuery(albumId);
}

export function clearAlbumQuery(_query: GalleryQueryState): GalleryQueryState {
  return resetGalleryQuery();
}

export function createPreviewAlbum(
  id: string,
  name: string,
  kind: AlbumListItem["kind"],
  sortOrder: number,
): AlbumListItem {
  return {
    id,
    name,
    kind,
    itemCount: 0,
    sortOrder,
  };
}

export function renameAlbumState(
  albums: AlbumListItem[],
  albumId: string,
  name: string,
): AlbumListItem[] {
  return albums.map((album) => (album.id === albumId ? { ...album, name } : album));
}

export function removeAlbumState(albums: AlbumListItem[], albumId: string): AlbumListItem[] {
  return albums.filter((album) => album.id !== albumId);
}

export function incrementAlbumItemCount(
  albums: AlbumListItem[],
  albumId: string,
  count: number,
): AlbumListItem[] {
  return albums.map((album) =>
    album.id === albumId
      ? { ...album, itemCount: (album.itemCount ?? 0) + count }
      : album,
  );
}
