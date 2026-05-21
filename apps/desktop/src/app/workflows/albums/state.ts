export {
  updateGalleryQuery,
  type DetailLoadState,
  type GalleryQueryState,
  type GallerySort,
} from "../gallery/state.js";
import { updateGalleryQuery, type GalleryQueryState } from "../gallery/state.js";
export { reorderByIds } from "../shared/state.js";

import type { AlbumListItem } from "../../types.js";

export function openAlbumQuery(query: GalleryQueryState, albumId: string): GalleryQueryState {
  return updateGalleryQuery(query, { albumId });
}

export function clearAlbumQuery(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, { albumId: null });
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
