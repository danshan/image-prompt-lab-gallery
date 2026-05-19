import { useMemo } from "react";
import {
  applyGalleryQuery,
  countActiveTasks,
  pendingReviewItems,
  sortedNonEmptyProviders,
  type GalleryFilterAsset,
  type GalleryQueryState,
  type ProviderState,
  type QueueTaskState,
  type ReviewStatusState,
} from "./workbench-state";
import { isRetryableTaskStatus, isTerminalFailureStatus } from "./studio-orchestration";

export function useGalleryDerivedState<TAsset extends GalleryFilterAsset & ProviderState & { id: string }>({
  runningInTauri,
  gallery,
  previewGallery,
  query,
  selectedAssetId,
}: {
  runningInTauri: boolean;
  gallery: TAsset[];
  previewGallery: TAsset[];
  query: GalleryQueryState;
  selectedAssetId: string;
}) {
  const sourceGallery = runningInTauri ? gallery : previewGallery;
  const displayedGallery = useMemo(
    () => (runningInTauri ? gallery : applyGalleryQuery(previewGallery, query)),
    [runningInTauri, gallery, previewGallery, query],
  );
  const selectedAsset = useMemo(
    () => displayedGallery.find((asset) => asset.id === selectedAssetId) ?? displayedGallery[0] ?? null,
    [displayedGallery, selectedAssetId],
  );
  const availableTags = useMemo(
    () => Array.from(new Set(sourceGallery.flatMap((asset) => asset.tags))).sort(),
    [sourceGallery],
  );
  const availableCategories = useMemo(
    () =>
      Array.from(new Set(sourceGallery.map((asset) => asset.category).filter((category): category is string => Boolean(category)))).sort(),
    [sourceGallery],
  );
  const availableProviders = useMemo(
    () => sortedNonEmptyProviders(sourceGallery),
    [sourceGallery],
  );
  return {
    displayedGallery,
    selectedAsset,
    availableTags,
    availableCategories,
    availableProviders,
  };
}

export function useReviewDerivedState<TSuggestion extends ReviewStatusState & { id: string }>(
  suggestions: TSuggestion[],
  selectedSuggestionId: string | null,
) {
  const pendingSuggestions = useMemo(
    () => pendingReviewItems(suggestions),
    [suggestions],
  );
  const selectedSuggestion = useMemo(
    () => pendingSuggestions.find((suggestion) => suggestion.id === selectedSuggestionId) ?? pendingSuggestions[0] ?? null,
    [pendingSuggestions, selectedSuggestionId],
  );
  return { pendingSuggestions, selectedSuggestion };
}

export function useTaskActivitySummary<TTask extends QueueTaskState>(tasks: TTask[]) {
  return useMemo(
    () => ({
      queueCount: countActiveTasks(tasks),
      runningTaskCount: tasks.filter((task) => task.status === "running").length,
      failedTaskCount: tasks.filter((task) => isTerminalFailureStatus(task.status) || isRetryableTaskStatus(task.status)).length,
    }),
    [tasks],
  );
}
