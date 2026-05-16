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
  sort: GallerySort;
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

export function rejectSuggestionState<TSuggestion extends SuggestionState>(
  suggestions: TSuggestion[],
  suggestionId: string,
) {
  return suggestions.filter((suggestion) => suggestion.id !== suggestionId);
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
