export type JobStatus = "queued" | "running" | "completed" | "failed";
export type GallerySort = "newest" | "oldest" | "ratingDesc" | "titleAsc" | "providerAsc";
export type ReviewStatusFilter = "any" | "pending";

export type AssetState = {
  id: string;
  title: string | null;
  category: string | null;
  rating: number | null;
  tags: string[];
};

export type SuggestionState = {
  id: string;
  assetId: string;
  title: string | null;
  category: string | null;
  tags: string[];
};

export type QueueJobState = {
  id: string;
  status: JobStatus;
};

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

export function acceptSuggestionState<TAsset extends AssetState, TSuggestion extends SuggestionState>(
  assets: TAsset[],
  suggestions: TSuggestion[],
  suggestion: TSuggestion,
) {
  return {
    assets: assets.map((asset) =>
      asset.id === suggestion.assetId
        ? {
            ...asset,
            title: suggestion.title,
            category: suggestion.category,
            tags: suggestion.tags,
          }
        : asset,
    ),
    suggestions: suggestions.filter((item) => item.id !== suggestion.id),
  };
}

export function removeSuggestionState<TSuggestion extends SuggestionState>(
  suggestions: TSuggestion[],
  suggestionId: string,
) {
  return suggestions.filter((suggestion) => suggestion.id !== suggestionId);
}

export function markAssetReviewPending<TAsset extends AssetState & { reviewPendingCount: number }>(
  assets: TAsset[],
  assetId: string,
): TAsset[] {
  return assets.map((asset) =>
    asset.id === assetId
      ? {
          ...asset,
          reviewPendingCount: Math.max(asset.reviewPendingCount, 1),
        }
      : asset,
  );
}

export function updateQueueJobStatus<TJob extends QueueJobState>(
  queue: TJob[],
  jobId: string,
  status: JobStatus,
) {
  return queue.map((job) => (job.id === jobId ? { ...job, status } : job));
}

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

export function openAlbumQuery(query: GalleryQueryState, albumId: string): GalleryQueryState {
  return updateGalleryQuery(query, { albumId });
}

export function clearAlbumQuery(query: GalleryQueryState): GalleryQueryState {
  return updateGalleryQuery(query, { albumId: null });
}

export type ReviewFormState = {
  suggestionId: string;
  title: string;
  description: string;
  schemaPrompt: string;
  tags: string[];
  tagInput: string;
  category: string;
};

export type EditableSuggestionState = {
  id: string;
  title: string | null;
  description?: string | null;
  schemaPrompt?: string | null;
  tags: string[];
  category: string | null;
};

export function createReviewFormState(suggestion: EditableSuggestionState): ReviewFormState {
  return {
    suggestionId: suggestion.id,
    title: suggestion.title ?? "",
    description: suggestion.description ?? "",
    schemaPrompt: suggestion.schemaPrompt ?? "",
    tags: uniqueTags(suggestion.tags),
    tagInput: "",
    category: suggestion.category ?? "",
  };
}

export function reviewFormTags(form: ReviewFormState): string[] {
  return uniqueTags(form.tags);
}

export function addReviewFormTag(form: ReviewFormState, tag: string): ReviewFormState {
  const normalized = tag.trim();
  if (!normalized) {
    return { ...form, tagInput: "" };
  }
  return {
    ...form,
    tags: uniqueTags([...form.tags, normalized]),
    tagInput: "",
  };
}

export function removeReviewFormTag(form: ReviewFormState, tag: string): ReviewFormState {
  return {
    ...form,
    tags: form.tags.filter((item) => item !== tag),
  };
}

function uniqueTags(tags: string[]): string[] {
  return Array.from(new Set(tags.map((tag) => tag.trim()).filter((tag) => tag.length > 0)));
}

export function clearCurationStateForLibrarySwitch() {
  return {
    selectedAlbumId: null as string | null,
    selectedSuggestionId: null as string | null,
    reviewForm: null as ReviewFormState | null,
  };
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
