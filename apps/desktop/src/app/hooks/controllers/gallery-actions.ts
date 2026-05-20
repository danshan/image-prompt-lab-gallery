import { useState, type Dispatch, type MutableRefObject, type SetStateAction } from "react";
import {
  clearAlbumQuery,
  clearLibraryWorkspaceState,
  acceptSuggestionState,
  addReviewFormTag,
  applySuggestionFieldToReviewForm,
  beginDetailLoad,
  beginReviewFieldGeneration,
  buildBatchReviewPayloads,
  completeDetailLoad,
  completeReviewFieldGeneration,
  defaultGalleryQuery,
  defaultSettingsSection,
  failDetailLoad,
  failReviewFieldGeneration,
  isReviewFieldGenerating,
  markAssetReviewPending,
  moveQueuedTaskOrder,
  reorderByIds,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleSelection,
  type DetailLoadState,
  type GalleryQueryState,
  type ReviewFieldName,
  type ReviewFormState,
  type SettingsSection,
} from "../../../workbench-state";
import {
  createTaskDraft,
  mockAlbumList,
  mockDetail,
  mockGallery,
  mockLibraries,
  mockLibrary,
  mockLibraryStatus,
  mockProviderHealth,
  mockSuggestions,
  mockTasks,
} from "../../mock-data";
import { createReviewFormState } from "../../../workbench-state";
import {
  METADATA_POLL_INTERVAL_MS,
  initialUpdateState,
} from "../../types";
import {
  buildChildPath,
  mockDetailFor,
  nextAnimationFrame,
  previewGeneratedReviewField,
  reviewFieldContext,
  schemaPromptFromAsset,
  suggestionFromAsset,
  validLibraryFolderName,
} from "../../utils";
import {
  compareTaskOrder,
  completedTaskKey,
  galleryQueryInput,
  isRetryableTaskStatus,
  isTerminalFailureStatus,
  mergeTasks,
  taskActionKey,
} from "../../../studio-orchestration";
import {
  errorMessage,
  invokeCommand,
  pickDirectory,
  pickSaveZipPath,
  pickZipFile,
} from "../../tauri-adapter";
import type {
  AlbumListItem,
  AppLog,
  AppLogContent,
  AssetDetail,
  AssetView,
  DaemonTask,
  DaemonTaskDetail,
  GalleryAsset,
  Library,
  LibraryBackup,
  LibraryStatus,
  LightboxImage,
  ProviderHealth,
  ReferenceSource,
  Suggestion,
  TaskDraft,
  TaskPanel,
  UpdateCheck,
  UpdateState,
  View,
} from "../../types";

export function useGallerySelectionActions({
  library,
  query,
  detailState,
  setGallery,
  setSelectedAssetId,
  setDetailState,
  setInspectorOpen,
  setStatus,
  setRecoverableError,
}: {
  library: Library | null;
  query: GalleryQueryState;
  detailState: DetailLoadState<AssetDetail>;
  setGallery: Dispatch<SetStateAction<GalleryAsset[]>>;
  setSelectedAssetId: Dispatch<SetStateAction<string>>;
  setDetailState: Dispatch<SetStateAction<DetailLoadState<AssetDetail>>>;
  setInspectorOpen: Dispatch<SetStateAction<boolean>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
}) {
  function selectGalleryAsset(assetId: string) {
    setSelectedAssetId(assetId);
    setInspectorOpen(true);
  }

  function selectAssetVersion(versionId: string) {
    const detail = detailState.detail;
    if (!detail) {
      return;
    }
    void loadAssetDetail(detail.id, versionId);
  }

  async function refreshGallery() {
    if (!library) {
      return;
    }
    try {
      const items = await invokeCommand<GalleryAsset[]>("query_gallery", {
        input: galleryQueryInput(library.rootPath, query),
      });
      setGallery(items);
      setSelectedAssetId((current) => items.find((item) => item.id === current)?.id ?? items[0]?.id ?? "");
      setStatus(`${items.length} item${items.length === 1 ? "" : "s"}`);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function loadAssetDetail(assetId: string, versionId: string | null) {
    if (!library) {
      return;
    }
    setDetailState(beginDetailLoad(assetId));
    try {
      const detail = await invokeCommand<AssetDetail>("get_asset_detail", {
        input: {
          libraryPath: library.rootPath,
          assetId,
          currentVersionId: versionId,
        },
      });
      setDetailState(completeDetailLoad(assetId, detail));
    } catch (error) {
      setDetailState(failDetailLoad(assetId, errorMessage(error)));
    }
  }

  function loadPreviewAssetDetail(asset: GalleryAsset) {
    setDetailState(completeDetailLoad(asset.id, mockDetailFor(asset)));
  }

  return {
    selectGalleryAsset,
    selectAssetVersion,
    refreshGallery,
    loadAssetDetail,
    loadPreviewAssetDetail,
  };
}
