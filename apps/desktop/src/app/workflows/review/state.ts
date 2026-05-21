export { type DetailLoadState } from "../gallery/state.js";
export { selectedOrCurrentIds, toggleSelection } from "../shared/state.js";

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

export type ReviewDraftSuggestionState = SuggestionState & {
  description?: string | null;
  schemaPrompt?: string | null;
};

export type ReviewFormState = {
  suggestionId: string;
  title: string;
  description: string;
  schemaPrompt: string;
  tags: string[];
  tagInput: string;
  category: string;
  generation: ReviewFieldGenerationMap;
};

export type EditableSuggestionState = {
  id: string;
  title: string | null;
  description?: string | null;
  schemaPrompt?: string | null;
  tags: string[];
  category: string | null;
};

export type ReviewFieldName = "title" | "description" | "schemaPrompt";

export type ReviewFieldGenerationState = {
  loading: boolean;
  requestId: string | null;
  error: string | null;
  logPath: string | null;
};

export type ReviewFieldGenerationMap = Record<ReviewFieldName, ReviewFieldGenerationState>;

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

export function createReviewFormState(suggestion: EditableSuggestionState): ReviewFormState {
  return {
    suggestionId: suggestion.id,
    title: suggestion.title ?? "",
    description: suggestion.description ?? "",
    schemaPrompt: suggestion.schemaPrompt ?? "",
    tags: uniqueTags(suggestion.tags),
    tagInput: "",
    category: suggestion.category ?? "",
    generation: createReviewFieldGenerationMap(),
  };
}

export function createReviewFieldGenerationMap(): ReviewFieldGenerationMap {
  return {
    title: createIdleReviewFieldGenerationState(),
    description: createIdleReviewFieldGenerationState(),
    schemaPrompt: createIdleReviewFieldGenerationState(),
  };
}

export function beginReviewFieldGeneration(
  form: ReviewFormState,
  field: ReviewFieldName,
  requestId: string,
): ReviewFormState {
  return {
    ...form,
    generation: {
      ...form.generation,
      [field]: {
        loading: true,
        requestId,
        error: null,
        logPath: null,
      },
    },
  };
}

export function completeReviewFieldGeneration(
  form: ReviewFormState,
  suggestionId: string,
  field: ReviewFieldName,
  requestId: string,
  value: string,
  logPath: string | null = null,
): ReviewFormState {
  if (!isCurrentReviewFieldRequest(form, suggestionId, field, requestId)) {
    return form;
  }
  return {
    ...form,
    [field]: value,
    generation: {
      ...form.generation,
      [field]: {
        loading: false,
        requestId: null,
        error: null,
        logPath,
      },
    },
  };
}

export function failReviewFieldGeneration(
  form: ReviewFormState,
  suggestionId: string,
  field: ReviewFieldName,
  requestId: string,
  error: string,
  logPath: string | null = null,
): ReviewFormState {
  if (!isCurrentReviewFieldRequest(form, suggestionId, field, requestId)) {
    return form;
  }
  return {
    ...form,
    generation: {
      ...form.generation,
      [field]: {
        loading: false,
        requestId: null,
        error,
        logPath,
      },
    },
  };
}

export function isReviewFieldGenerating(form: ReviewFormState, field: ReviewFieldName): boolean {
  return form.generation[field].loading;
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

export function applySuggestionFieldToReviewForm(
  form: ReviewFormState,
  suggestion: EditableSuggestionState,
  field: ReviewFieldName | "tags" | "category",
): ReviewFormState {
  switch (field) {
    case "title":
      return { ...form, title: suggestion.title ?? "" };
    case "description":
      return { ...form, description: suggestion.description ?? "" };
    case "schemaPrompt":
      return { ...form, schemaPrompt: suggestion.schemaPrompt ?? "" };
    case "tags":
      return { ...form, tags: uniqueTags(suggestion.tags) };
    case "category":
      return { ...form, category: suggestion.category ?? "" };
  }
}

export function buildBatchReviewPayloads<TSuggestion extends ReviewDraftSuggestionState>(
  suggestions: TSuggestion[],
  selectedIds: string[],
  currentForm: ReviewFormState | null,
): TSuggestion[] {
  const ids = new Set(selectedIds);
  return suggestions
    .filter((suggestion) => ids.has(suggestion.id))
    .map((suggestion) =>
      currentForm?.suggestionId === suggestion.id
        ? {
            ...suggestion,
            title: currentForm.title || null,
            description: currentForm.description || null,
            schemaPrompt: currentForm.schemaPrompt || null,
            tags: reviewFormTags(currentForm),
            category: currentForm.category || null,
          }
        : suggestion,
    );
}

export function clearCurationStateForLibrarySwitch() {
  return {
    selectedAlbumId: null as string | null,
    selectedSuggestionId: null as string | null,
    reviewForm: null as ReviewFormState | null,
  };
}

function uniqueTags(tags: string[]): string[] {
  return Array.from(new Set(tags.map((tag) => tag.trim()).filter((tag) => tag.length > 0)));
}

function createIdleReviewFieldGenerationState(): ReviewFieldGenerationState {
  return {
    loading: false,
    requestId: null,
    error: null,
    logPath: null,
  };
}

function isCurrentReviewFieldRequest(
  form: ReviewFormState,
  suggestionId: string,
  field: ReviewFieldName,
  requestId: string,
): boolean {
  return form.suggestionId === suggestionId && form.generation[field].requestId === requestId;
}
