export type GallerySort = "newest" | "oldest" | "ratingDesc" | "titleAsc" | "providerAsc" | "albumOrder";
export type ReviewStatusFilter = "any" | "pending";

export type GalleryQueryState = {
  text: string;
  providers: string[];
  minRating: number | null;
  reviewStatus: ReviewStatusFilter;
  tags: string[];
  albumId: string | null;
  sort: GallerySort;
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
  albumId: null,
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

export function resetGalleryQuery(): GalleryQueryState {
  return { ...defaultGalleryQuery, providers: [], tags: [] };
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
    return true;
  });

  return filtered.sort((left, right) => compareGalleryAssets(left, right, query.sort));
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
