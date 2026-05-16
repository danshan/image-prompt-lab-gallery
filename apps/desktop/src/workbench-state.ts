export type JobStatus = "queued" | "running" | "completed" | "failed";

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
