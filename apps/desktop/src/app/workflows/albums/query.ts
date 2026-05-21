import type { GallerySort } from "./state";
import type { GalleryAsset } from "../../types";

export type SmartAlbumBuilderState = {
  text: string;
  tags: string;
  providers: string;
  minRating: string;
  reviewPending: boolean;
  category: string;
  status: string;
  createdAtFrom: string;
  createdAtTo: string;
  sort: GallerySort;
};

export function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

export function smartAlbumQueryJson(builder: SmartAlbumBuilderState): string {
  return JSON.stringify({
    ...(builder.text.trim() ? { text: builder.text.trim() } : {}),
    ...(builder.tags.trim() ? { tags: splitCsv(builder.tags) } : {}),
    ...(builder.providers.trim() ? { providers: splitCsv(builder.providers) } : {}),
    ...(builder.minRating.trim() ? { minRating: Number(builder.minRating.trim()) } : {}),
    ...(builder.reviewPending ? { reviewStatus: "pending" } : {}),
    ...(builder.category ? { category: builder.category } : {}),
    ...(builder.status.trim() ? { status: builder.status.trim() } : {}),
    ...(builder.createdAtFrom.trim() ? { createdAtFrom: builder.createdAtFrom.trim() } : {}),
    ...(builder.createdAtTo.trim() ? { createdAtTo: builder.createdAtTo.trim() } : {}),
    sort: builder.sort,
  });
}

export function previewSmartAlbumCount(
  assets: GalleryAsset[],
  builder: SmartAlbumBuilderState,
): number {
  const text = builder.text.trim().toLocaleLowerCase();
  const tags = splitCsv(builder.tags);
  const providers = splitCsv(builder.providers);
  const minRating = builder.minRating.trim() ? Number(builder.minRating.trim()) : null;

  return assets.filter((asset) => {
    if (
      text &&
      ![asset.title, asset.category, asset.status, asset.provider, asset.modelLabel, asset.prompt, ...asset.tags].some(
        (value) => value?.toLocaleLowerCase().includes(text),
      )
    ) {
      return false;
    }
    if (tags.length > 0 && !tags.every((tag) => asset.tags.includes(tag))) {
      return false;
    }
    if (providers.length > 0 && (!asset.provider || !providers.includes(asset.provider))) {
      return false;
    }
    if (minRating !== null && (asset.rating ?? 0) < minRating) {
      return false;
    }
    if (builder.reviewPending && asset.reviewPendingCount === 0) {
      return false;
    }
    if (builder.category && asset.category !== builder.category) {
      return false;
    }
    if (builder.status.trim() && asset.status !== builder.status.trim()) {
      return false;
    }
    if (builder.createdAtFrom.trim() && asset.createdAt < builder.createdAtFrom.trim()) {
      return false;
    }
    if (builder.createdAtTo.trim() && asset.createdAt > builder.createdAtTo.trim()) {
      return false;
    }
    return true;
  }).length;
}
