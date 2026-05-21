import type { Dispatch, SetStateAction } from "react";
import type { DetailLoadState } from "../../workflows/gallery";
import {
  mockSuggestions,
} from "../../mock-data";
import {
  nextAnimationFrame,
  previewGeneratedReviewField,
  reviewFieldContext,
  schemaPromptFromAsset,
  suggestionFromAsset,
} from "../../utils";
import {
  mergeTasks,
} from "../../../studio-orchestration";
import {
  errorMessage,
  invokeCommand,
} from "../../tauri-adapter";
import {
  acceptSuggestionState,
  addReviewFormTag,
  applySuggestionFieldToReviewForm,
  beginReviewFieldGeneration,
  buildBatchReviewPayloads,
  completeReviewFieldGeneration,
  createReviewFormState,
  failReviewFieldGeneration,
  isReviewFieldGenerating,
  markAssetReviewPending,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleSelection,
  type ReviewFieldName,
  type ReviewFormState,
} from "../../workflows/review/state";
import type {
  AssetDetail,
  AssetView,
  DaemonTask,
  DaemonTaskDetail,
  GalleryAsset,
  Library,
  Suggestion,
  View,
} from "../../types";

export function useReviewActions({
  runningInTauri,
  library,
  selectedSuggestion,
  reviewForm,
  suggestions,
  selectedSuggestionIds,
  pendingSuggestions,
  suggestionRegenerating,
  gallery,
  selectedAsset,
  availableCategories,
  selectedTaskId,
  detailState,
  setSuggestions,
  setSelectedSuggestionId,
  setSelectedSuggestionIds,
  setSuggestionHistory,
  setSuggestionRegenerating,
  setReviewForm,
  setGallery,
  setDetailState,
  setTasks,
  setSelectedTaskId,
  setActiveView,
  setStatus,
  setRecoverableError,
  refreshGallery,
  refreshSuggestions,
  refreshAlbums,
  refreshSuggestionHistory,
  refreshTasks,
  loadAssetDetail,
  waitForMetadataFieldResult,
  waitForMetadataSuggestionResult,
}: {
  runningInTauri: boolean;
  library: Library | null;
  selectedSuggestion: Suggestion | null;
  reviewForm: ReviewFormState | null;
  suggestions: Suggestion[];
  selectedSuggestionIds: string[];
  pendingSuggestions: Suggestion[];
  suggestionRegenerating: boolean;
  gallery: GalleryAsset[];
  selectedAsset: GalleryAsset | null;
  availableCategories: string[];
  selectedTaskId: string | null;
  detailState: DetailLoadState<AssetDetail>;
  setSuggestions: Dispatch<SetStateAction<Suggestion[]>>;
  setSelectedSuggestionId: Dispatch<SetStateAction<string | null>>;
  setSelectedSuggestionIds: Dispatch<SetStateAction<string[]>>;
  setSuggestionHistory: Dispatch<SetStateAction<Suggestion[]>>;
  setSuggestionRegenerating: Dispatch<SetStateAction<boolean>>;
  setReviewForm: Dispatch<SetStateAction<ReviewFormState | null>>;
  setGallery: Dispatch<SetStateAction<GalleryAsset[]>>;
  setDetailState: Dispatch<SetStateAction<DetailLoadState<AssetDetail>>>;
  setTasks: Dispatch<SetStateAction<DaemonTask[]>>;
  setSelectedTaskId: Dispatch<SetStateAction<string | null>>;
  setActiveView: Dispatch<SetStateAction<View>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
  refreshGallery: () => Promise<void>;
  refreshSuggestions: () => Promise<void>;
  refreshAlbums: () => Promise<void>;
  refreshSuggestionHistory: (suggestion: Suggestion) => Promise<void>;
  refreshTasks: (options?: { showLoading?: boolean }) => Promise<DaemonTask[]>;
  loadAssetDetail: (assetId: string, versionId: string | null) => Promise<void>;
  waitForMetadataFieldResult: (taskId: string, suggestionId: string, field: ReviewFieldName, baseRevision: string) => Promise<string>;
  waitForMetadataSuggestionResult: (taskId: string) => Promise<string>;
}) {
  function selectSuggestion(suggestion: Suggestion) {
    setSelectedSuggestionId(suggestion.id);
    setReviewForm(createReviewFormState(suggestion));
  }

  function toggleSuggestionForBatch(suggestionId: string) {
    setSelectedSuggestionIds((current) => toggleSelection(current, suggestionId));
  }

  async function acceptReviewForm() {
    if (!library || !selectedSuggestion || !reviewForm) {
      return;
    }
    const finalForm = addReviewFormTag(reviewForm, reviewForm.tagInput);
    const finalSuggestion: Suggestion = {
      ...selectedSuggestion,
      title: finalForm.title.trim() || null,
      description: finalForm.description.trim() || null,
      schemaPrompt: finalForm.schemaPrompt.trim() || null,
      tags: reviewFormTags(finalForm),
      category:
        finalForm.category.trim() && availableCategories.includes(finalForm.category.trim())
          ? finalForm.category.trim()
          : null,
    };
    await acceptSuggestion(finalSuggestion);
  }

  async function acceptSuggestion(suggestion: Suggestion) {
    if (!library) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("accept_suggestion", {
        input: {
          libraryPath: library.rootPath,
          suggestionId: suggestion.id,
          title: suggestion.title,
          description: suggestion.description,
          schemaPrompt: suggestion.schemaPrompt,
          tags: suggestion.tags,
          category: suggestion.category,
        },
      });
      setGallery((current) => {
        const state = acceptSuggestionState(current, suggestions, {
          id: suggestion.id,
          assetId: asset.id,
          title: asset.title,
          description: suggestion.description,
          schemaPrompt: suggestion.schemaPrompt,
          category: asset.category,
          tags: suggestion.tags,
          status: suggestion.status,
        });
        return state.assets;
      });
      setSuggestions((current) => current.filter((item) => item.id !== suggestion.id));
      await Promise.all([
        refreshGallery(),
        refreshSuggestions(),
        detailState.detail?.id === asset.id
          ? loadAssetDetail(asset.id, selectedAsset?.currentVersionId ?? null)
          : Promise.resolve(),
      ]);
      setSelectedSuggestionIds((current) => current.filter((id) => id !== suggestion.id));
      setReviewForm(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function batchAcceptReviewSuggestions() {
    if (!library) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    if (ids.length === 0) {
      return;
    }
    const finalForm = reviewForm ? addReviewFormTag(reviewForm, reviewForm.tagInput) : null;
    const payloads = buildBatchReviewPayloads(suggestions, ids, finalForm).map((suggestion) => ({
      libraryPath: library.rootPath,
      suggestionId: suggestion.id,
      title: suggestion.title,
      description: suggestion.description ?? null,
      schemaPrompt: suggestion.schemaPrompt ?? null,
      tags: suggestion.tags,
      category:
        suggestion.category && availableCategories.includes(suggestion.category)
          ? suggestion.category
          : null,
    }));
    try {
      await invokeCommand<AssetView[]>("batch_accept_suggestions", {
        input: {
          libraryPath: library.rootPath,
          suggestions: payloads,
        },
      });
      await Promise.all([refreshGallery(), refreshSuggestions()]);
      setSelectedSuggestionIds([]);
      setReviewForm(null);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshSuggestions();
    }
  }

  async function batchRejectReviewSuggestions() {
    if (!library) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    if (ids.length === 0) {
      return;
    }
    try {
      await invokeCommand("batch_reject_suggestions", {
        input: {
          libraryPath: library.rootPath,
          suggestionIds: ids,
        },
      });
      await Promise.all([refreshGallery(), refreshSuggestions()]);
      setSelectedSuggestionIds([]);
      setReviewForm(null);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshSuggestions();
    }
  }

  async function addReviewSelectionToAlbum(albumId: string) {
    if (!library || albumId.length === 0) {
      return;
    }
    const ids = selectedOrCurrentIds(selectedSuggestionIds, selectedSuggestion?.id ?? null);
    const assetIds = suggestions
      .filter((suggestion) => ids.includes(suggestion.id))
      .map((suggestion) => suggestion.assetId);
    if (assetIds.length === 0) {
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId,
          assetIds,
        },
      });
      await refreshAlbums();
      setRecoverableError(null);
      setStatus("Review assets added to album");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function pickReviewHistoryField(suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") {
    if (!reviewForm) {
      return;
    }
    setReviewForm(applySuggestionFieldToReviewForm(reviewForm, suggestion, field));
  }

  function restoreReviewForm() {
    if (selectedSuggestion) {
      setReviewForm(createReviewFormState(selectedSuggestion));
    }
  }

  async function regenerateReviewField(field: ReviewFieldName) {
    if (!reviewForm || !selectedSuggestion || isReviewFieldGenerating(reviewForm, field)) {
      return;
    }
    const requestId = crypto.randomUUID();
    const suggestionId = selectedSuggestion.id;
    setReviewForm(beginReviewFieldGeneration(reviewForm, field, requestId));
    const asset = gallery.find((item) => item.id === selectedSuggestion.assetId) ?? selectedAsset;
    const sourceText = asset?.prompt ?? selectedSuggestion.title ?? reviewForm.title;
    if (!runningInTauri) {
      const value = previewGeneratedReviewField(field, asset, sourceText);
      setReviewForm((current) =>
        current ? completeReviewFieldGeneration(current, suggestionId, field, requestId, value, null) : current,
      );
      return;
    }
    if (!library) {
      setReviewForm((current) =>
        current
          ? failReviewFieldGeneration(current, suggestionId, field, requestId, "Open a real library before regenerating review metadata.")
          : current,
      );
      return;
    }
    try {
      await nextAnimationFrame();
      const created = await invokeCommand<DaemonTask[]>("enqueue_generation_tasks", {
        input: {
          libraryPath: library.rootPath,
          tasks: [
            {
              taskType: "metadata_field_generation",
              provider: "codex-cli",
              prompt: `${field} metadata generation`,
              operation: "text_to_image",
              parametersJson: "{}",
              priority: 0,
              maxAttempts: 3,
              input: {
                suggestionId,
                assetId: selectedSuggestion.assetId,
                field,
                baseRevision: requestId,
                context: reviewFieldContext(reviewForm, selectedSuggestion, asset),
              },
            },
          ],
        },
      });
      setTasks((current) => mergeTasks(created, current));
      setSelectedTaskId(created[0]?.id ?? selectedTaskId);
      const result = await waitForMetadataFieldResult(created[0].id, suggestionId, field, requestId);
      setReviewForm((current) =>
        current ? completeReviewFieldGeneration(current, suggestionId, field, requestId, result, null) : current,
      );
      setRecoverableError(null);
      void refreshTasks();
    } catch (error) {
      const message = errorMessage(error);
      setReviewForm((current) =>
        current ? failReviewFieldGeneration(current, suggestionId, field, requestId, message, null) : current,
      );
      setRecoverableError(message);
    }
  }

  async function regenerateFullSuggestion() {
    if (!selectedSuggestion || !reviewForm || suggestionRegenerating) {
      return;
    }
    setSuggestionRegenerating(true);
    setStatus("Regenerating suggestion");
    if (!runningInTauri) {
      const regenerated: Suggestion = {
        ...selectedSuggestion,
        id: `suggestion-${crypto.randomUUID()}`,
        title: `${reviewForm.title || selectedSuggestion.title || "Untitled"} variant`,
        description: reviewForm.description || selectedSuggestion.description,
        schemaPrompt: reviewForm.schemaPrompt || selectedSuggestion.schemaPrompt,
        tags: reviewFormTags(reviewForm),
        category: reviewForm.category || selectedSuggestion.category,
      };
      setSuggestions((current) => [regenerated, ...current]);
      setSuggestionHistory((current) => [regenerated, ...current]);
      setSelectedSuggestionId(regenerated.id);
      setReviewForm(createReviewFormState(regenerated));
      setStatus("Suggestion regenerated");
      setSuggestionRegenerating(false);
      return;
    }
    if (!library) {
      setRecoverableError("Open a real library before regenerating suggestions.");
      setSuggestionRegenerating(false);
      return;
    }
    try {
      const created = await invokeCommand<DaemonTask[]>("enqueue_generation_tasks", {
        input: {
          libraryPath: library.rootPath,
          tasks: [
            {
              taskType: "metadata_suggestion_generation",
              provider: "codex-cli",
              prompt: "metadata suggestion generation",
              operation: "text_to_image",
              parametersJson: "{}",
              priority: 0,
              maxAttempts: 3,
              input: {
                suggestionId: selectedSuggestion.id,
                assetId: selectedSuggestion.assetId,
                baseRevision: reviewForm.suggestionId,
                context: reviewFieldContext(
                  reviewForm,
                  selectedSuggestion,
                  gallery.find((item) => item.id === selectedSuggestion.assetId) ?? selectedAsset,
                ),
              },
            },
          ],
        },
      });
      setTasks((current) => mergeTasks(created, current));
      setSelectedTaskId(created[0]?.id ?? selectedTaskId);
      const regeneratedSuggestionId = await waitForMetadataSuggestionResult(created[0].id);
      const nextSuggestions = await invokeCommand<Suggestion[]>("list_pending_suggestions", { libraryPath: library.rootPath });
      setSuggestions(nextSuggestions);
      const regenerated = nextSuggestions.find((item) => item.id === regeneratedSuggestionId);
      if (regenerated) {
        await refreshSuggestionHistory(regenerated);
        setSelectedSuggestionId(regenerated.id);
        setReviewForm(createReviewFormState(regenerated));
      }
      setStatus("Suggestion regenerated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setSuggestionRegenerating(false);
    }
  }

  async function requestAssetReview(asset: GalleryAsset) {
    const existing = pendingSuggestions.find((suggestion) => suggestion.assetId === asset.id);
    if (existing) {
      setSelectedSuggestionId(existing.id);
      setReviewForm(createReviewFormState(existing));
      setActiveView("review");
      return;
    }
    if (!library) {
      const suggestion = suggestionFromAsset(asset);
      setSuggestions((current) => [suggestion, ...current.filter((item) => item.assetId !== asset.id)]);
      setGallery((current) => markAssetReviewPending(current, asset.id));
      setDetailState((current) =>
        current.detail?.id === asset.id
          ? { ...current, detail: { ...current.detail, reviewPendingCount: Math.max(current.detail.reviewPendingCount, 1) } }
          : current,
      );
      setSelectedSuggestionId(suggestion.id);
      setActiveView("review");
      return;
    }
    try {
      const suggestion = await invokeCommand<Suggestion>("create_suggestion", {
        input: {
          libraryPath: library.rootPath,
          assetId: asset.id,
          title: asset.title,
          description: null,
          schemaPrompt: schemaPromptFromAsset(asset, asset.prompt ?? asset.title ?? ""),
          tags: asset.tags,
          category: asset.category && availableCategories.includes(asset.category) ? asset.category : null,
          confidenceJson: JSON.stringify({ source: "manual_re_review" }),
        },
      });
      setGallery((current) => markAssetReviewPending(current, asset.id));
      setSelectedSuggestionId(suggestion.id);
      setActiveView("review");
      await Promise.all([
        refreshGallery(),
        refreshSuggestions(),
        detailState.detail?.id === asset.id ? loadAssetDetail(asset.id, asset.currentVersionId) : Promise.resolve(),
      ]);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  return {
    selectSuggestion,
    toggleSuggestionForBatch,
    acceptReviewForm,
    batchAcceptReviewSuggestions,
    batchRejectReviewSuggestions,
    addReviewSelectionToAlbum,
    pickReviewHistoryField,
    restoreReviewForm,
    regenerateReviewField,
    regenerateFullSuggestion,
    requestAssetReview,
  };
}
