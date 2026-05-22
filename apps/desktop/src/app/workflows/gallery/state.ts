export type GallerySort = "newest" | "oldest" | "ratingDesc" | "titleAsc" | "providerAsc" | "albumOrder";
export type ReviewStatusFilter = "any" | "pending";
export type GalleryAlbumFilterState =
  | { mode: "any" }
  | { mode: "inAny"; albumIds: string[] }
  | { mode: "unassigned" };

export type GalleryQueryState = {
  text: string;
  providers: string[];
  minRating: number | null;
  reviewStatus: ReviewStatusFilter;
  tags: string[];
  albumFilter: GalleryAlbumFilterState;
  sort: GallerySort;
};

export type GalleryFilterAlbum = {
  id: string;
};

export type GalleryFilterAsset = {
  title: string | null;
  category: string | null;
  rating: number | null;
  status: string;
  provider: string | null;
  modelLabel: string | null;
  prompt?: string | null;
  tags: string[];
  reviewPendingCount: number;
  albums?: GalleryFilterAlbum[];
  createdAt: string;
  updatedAt: string;
};

export type DetailLoadState<TDetail> = {
  assetId: string | null;
  detail: TDetail | null;
  loading: boolean;
  error: string | null;
};

export const defaultGalleryQuery: GalleryQueryState = {
  text: "",
  providers: [],
  minRating: null,
  reviewStatus: "any",
  tags: [],
  albumFilter: { mode: "any" },
  sort: "newest",
};

export function updateGalleryQuery(
  query: GalleryQueryState,
  patch: Partial<GalleryQueryState>,
): GalleryQueryState {
  return {
    ...query,
    ...patch,
  };
}

export function toggleGalleryProvider(
  query: GalleryQueryState,
  provider: string,
): GalleryQueryState {
  const providers = query.providers.includes(provider)
    ? query.providers.filter((item) => item !== provider)
    : [...query.providers, provider];
  return updateGalleryQuery(query, { providers });
}

export function toggleGalleryTag(query: GalleryQueryState, tag: string): GalleryQueryState {
  const tags = query.tags.includes(tag)
    ? query.tags.filter((item) => item !== tag)
    : [...query.tags, tag];
  return updateGalleryQuery(query, { tags });
}

export function clearGalleryTextFilter(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, { text: "" });
}

export function clearGalleryProviderFilter(
  query: GalleryQueryState,
  provider: string,
): GalleryQueryState {
  return updateGalleryQuery(query, {
    providers: query.providers.filter((item) => item !== provider),
  });
}

export function clearGalleryMinRatingFilter(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, { minRating: null });
}

export function clearGalleryReviewFilter(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, { reviewStatus: "any" });
}

export function clearGalleryTagFilter(query: GalleryQueryState, tag: string): GalleryQueryState {
  return updateGalleryQuery(query, {
    tags: query.tags.filter((item) => item !== tag),
  });
}

export function galleryAlbumFilterIds(query: GalleryQueryState): string[] {
  return query.albumFilter.mode === "inAny" ? query.albumFilter.albumIds : [];
}

export function clearGalleryAlbumFilter(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, {
    albumFilter: { mode: "any" },
    sort: query.sort === "albumOrder" ? "newest" : query.sort,
  });
}

export function setGalleryAlbumFilter(
  query: GalleryQueryState,
  albumIds: string[],
): GalleryQueryState {
  return updateGalleryQuery(query, {
    albumFilter: normalizeGalleryAlbumFilter({ mode: "inAny", albumIds }),
    sort: query.sort === "albumOrder" ? "newest" : query.sort,
  });
}

export function toggleGalleryAlbumFilter(
  query: GalleryQueryState,
  albumId: string,
): GalleryQueryState {
  const current = query.albumFilter.mode === "inAny" ? query.albumFilter.albumIds : [];
  const albumIds = current.includes(albumId)
    ? current.filter((item) => item !== albumId)
    : [...current, albumId];
  return setGalleryAlbumFilter(query, albumIds);
}

export function removeGalleryAlbumFilter(
  query: GalleryQueryState,
  albumId: string,
): GalleryQueryState {
  if (query.albumFilter.mode !== "inAny") {
    return query;
  }
  return setGalleryAlbumFilter(
    query,
    query.albumFilter.albumIds.filter((item) => item !== albumId),
  );
}

export function setGalleryUnassignedAlbumFilter(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, {
    albumFilter: { mode: "unassigned" },
    sort: query.sort === "albumOrder" ? "newest" : query.sort,
  });
}

export function normalizeGalleryAlbumFilter(
  albumFilter: GalleryAlbumFilterState,
): GalleryAlbumFilterState {
  if (albumFilter.mode !== "inAny") {
    return albumFilter;
  }
  const albumIds = Array.from(new Set(albumFilter.albumIds));
  return albumIds.length > 0 ? { mode: "inAny", albumIds } : { mode: "any" };
}

export function resetGalleryQuery(): GalleryQueryState {
  return { ...defaultGalleryQuery, providers: [], tags: [], albumFilter: { mode: "any" } };
}

export function beginDetailLoad<TDetail>(assetId: string): DetailLoadState<TDetail> {
  return {
    assetId,
    detail: null,
    loading: true,
    error: null,
  };
}

export function completeDetailLoad<TDetail>(
  assetId: string,
  detail: TDetail,
): DetailLoadState<TDetail> {
  return {
    assetId,
    detail,
    loading: false,
    error: null,
  };
}

export function failDetailLoad<TDetail>(
  assetId: string,
  error: string,
): DetailLoadState<TDetail> {
  return {
    assetId,
    detail: null,
    loading: false,
    error,
  };
}

export function clearSelectionForLibrarySwitch<TDetail>(): DetailLoadState<TDetail> {
  return {
    assetId: null,
    detail: null,
    loading: false,
    error: null,
  };
}

export function applyGalleryQuery<TAsset extends GalleryFilterAsset>(
  assets: TAsset[],
  query: GalleryQueryState,
): TAsset[] {
  const text = query.text.trim().toLocaleLowerCase();
  const filtered = assets.filter((asset) => {
    if (text.length > 0 && !assetMatchesText(asset, text)) {
      return false;
    }
    if (
      query.providers.length > 0 &&
      (!asset.provider || !query.providers.includes(asset.provider))
    ) {
      return false;
    }
    if (query.minRating !== null && (asset.rating ?? 0) < query.minRating) {
      return false;
    }
    if (query.reviewStatus === "pending" && asset.reviewPendingCount === 0) {
      return false;
    }
    if (!query.tags.every((tag) => asset.tags.includes(tag))) {
      return false;
    }
    if (!assetMatchesAlbumFilter(asset, query.albumFilter)) {
      return false;
    }
    return true;
  });

  return filtered.sort((left, right) => compareGalleryAssets(left, right, query.sort));
}

function assetMatchesAlbumFilter(
  asset: GalleryFilterAsset,
  albumFilter: GalleryAlbumFilterState,
): boolean {
  const albums = asset.albums ?? [];
  switch (albumFilter.mode) {
    case "any":
      return true;
    case "unassigned":
      return albums.length === 0;
    case "inAny":
      return albumFilter.albumIds.length === 0 || albums.some((album) => albumFilter.albumIds.includes(album.id));
  }
}

function assetMatchesText(asset: GalleryFilterAsset, text: string): boolean {
  return [
    asset.title,
    asset.category,
    asset.status,
    asset.provider,
    asset.modelLabel,
    asset.prompt,
    ...asset.tags,
  ].some((value) => value?.toLocaleLowerCase().includes(text));
}

function compareGalleryAssets(
  left: GalleryFilterAsset,
  right: GalleryFilterAsset,
  sort: GallerySort,
): number {
  switch (sort) {
    case "oldest":
      return compareText(left.createdAt, right.createdAt);
    case "ratingDesc":
      return (right.rating ?? 0) - (left.rating ?? 0) || compareNullableText(left.title, right.title);
    case "titleAsc":
      return compareNullableText(left.title, right.title);
    case "providerAsc":
      return compareNullableText(left.provider, right.provider);
    case "albumOrder":
      return 0;
    case "newest":
      return compareText(right.updatedAt, left.updatedAt) || compareText(right.createdAt, left.createdAt);
  }
}

function compareNullableText(left: string | null, right: string | null): number {
  return compareText(left ?? "", right ?? "");
}

function compareText(left: string, right: string): number {
  return left.localeCompare(right);
}

export function formatAspectRatio(width: number | null, height: number | null): string {
  if (!isPositiveDimension(width) || !isPositiveDimension(height)) {
    return "Unavailable";
  }
  const divisor = greatestCommonDivisor(width, height);
  return `${width / divisor}:${height / divisor}`;
}

function isPositiveDimension(value: number | null): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0;
}

function greatestCommonDivisor(left: number, right: number): number {
  let a = Math.abs(left);
  let b = Math.abs(right);
  while (b !== 0) {
    const next = a % b;
    a = b;
    b = next;
  }
  return a;
}
