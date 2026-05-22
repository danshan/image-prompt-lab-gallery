import type { GalleryQueryState } from "./state";

export function galleryQueryInput(libraryPath: string, query: GalleryQueryState) {
  return {
    libraryPath,
    text: query.text.trim() || null,
    providers: query.providers,
    minRating: query.minRating,
    reviewStatus: query.reviewStatus,
    tags: query.tags,
    sort: query.sort,
    albumFilter: query.albumFilter.mode === "inAny" && query.albumFilter.albumIds.length === 0
      ? { mode: "any" }
      : query.albumFilter,
  };
}
