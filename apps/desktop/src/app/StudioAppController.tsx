import React, { useMemo, useRef, useState } from "react";
import {
  applyGalleryQuery,
  beginDetailLoad,
  clearSelectionForLibrarySwitch,
  completeDetailLoad,
  defaultGalleryQuery,
  failDetailLoad,
  type DetailLoadState,
  type GalleryQueryState,
  resetGalleryQuery,
} from "./workflows/gallery";
import {
  clearCurationStateForLibrarySwitch,
  type ReviewFieldName,
  type ReviewFormState,
} from "./workflows/review";
import { toggleSelection } from "./workflows/shared/state";
import { clearLibraryWorkspaceState } from "./workflows/library/state";
import { ActivityStrip, AppShell } from "../studio-shell";
import { Icon } from "../studio-icons";
import { Sidebar } from "../studio-navigation";
import { useGalleryDerivedState } from "./hooks/gallery";
import { useReviewDerivedState } from "./hooks/review";
import { useTaskActivitySummary } from "./hooks/tasks";
import {
  useAlbumControllerState,
  useGallerySelectionActions,
  useGallerySelectionControllerState,
  useReviewActions,
  useReviewControllerState,
  useTaskGenerationActions,
  useTaskGenerationControllerState,
} from "./hooks/controllers";
import { useStudioRefreshPolicy } from "./hooks/controllers/refresh-policy";
import {
  compareTaskOrder,
  completedTaskKey,
  formatOperation,
  formatVersionName,
  galleryQueryInput,
  isRetryableTaskStatus,
  isTerminalFailureStatus,
  mergeTasks,
  shortIdentifier,
  statusLabel,
  taskActionKey,
  taskPrompt,
} from "../studio-orchestration";
import {
  convertImagePath,
  errorMessage,
  hasTauriRuntime,
  invokeCommand,
  pickDirectory,
  pickImageFile,
  pickSaveZipPath,
  pickZipFile,
} from "./tauri-adapter";
import { GalleryWorkspace, ImageLightbox } from "./screens/gallery/GalleryWorkspace";
import {
  AlbumsWorkspace,
  GenerationComposer,
  Inspector,
  ReviewWorkspace,
  SettingsWorkspace,
  StudioOverviewBand,
  TaskWorkspace,
  WorkspaceToolbar,
} from "./screens/workflows";
import { StarRatingControl, StarRatingDisplay } from "./components/rating";
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
} from "./mock-data";
import {
  buildChildPath,
  descriptionFromPrompt,
  nextAnimationFrame,
  previewGeneratedReviewField,
  reviewFieldContext,
  schemaPromptFromAsset,
  suggestionFromAsset,
  thumbnailAspectRatio,
  thumbnailImageStyle,
  thumbnailStyle,
  titleFromPrompt,
  validLibraryFolderName,
} from "./utils";
import {
  albumContentsQuery,
  defaultAlbumAddSourceQuery,
  filterAlbumAddCandidates,
  incrementAlbumItemCount,
  createPreviewAlbum,
  removeAlbumState,
  reorderByIds,
  renameAlbumState,
} from "./workflows/albums";
import {
  defaultSettingsSection,
  useAppOperationsActions,
  useAppOperationsControllerState,
  useLibrarySettingsActions,
  useLibrarySettingsControllerState,
  type SettingsSection,
} from "./workflows/settings";
import { initialUpdateState } from "./types";
import type {
  Album,
  AlbumListItem,
  AppLog,
  AppLogContent,
  AssetDetail,
  AssetView,
  CommandError,
  DaemonTask,
  DaemonTaskDetail,
  FileContext,
  GalleryAsset,
  GeneratedReviewField,
  Library,
  LibraryBackup,
  LibraryStatus,
  LightboxImage,
  PromoteAssetVersionResult,
  ProviderHealth,
  ReferenceSource,
  Suggestion,
  TaskDraft,
  TaskPanel,
  UpdateCheck,
  UpdateState,
  View,
  ConfidenceScore,
} from "./types";

export function StudioAppController() {
  const runningInTauri = hasTauriRuntime();
  const [activeView, setActiveView] = useState<View>("gallery");
  const {
    libraries,
    setLibraries,
    library,
    setLibrary,
    libraryStatus,
    setLibraryStatus,
    providerHealth,
    setProviderHealth,
    libraryFolderNameInput,
    setLibraryFolderNameInput,
    libraryNameInput,
    setLibraryNameInput,
    settingsSection,
    setSettingsSection,
    pendingLibraryActions,
    setPendingLibraryActions,
    missingLibraryPaths,
    setMissingLibraryPaths,
  } = useLibrarySettingsControllerState(runningInTauri);
  const {
    gallery,
    setGallery,
    selectedGalleryAssetIds,
    setSelectedGalleryAssetIds,
    query,
    setQuery,
    selectedAssetId,
    setSelectedAssetId,
    detailState,
    setDetailState,
    lightboxImage,
    setLightboxImage,
    sidebarExpanded,
    setSidebarExpanded,
    inspectorOpen,
    setInspectorOpen,
  } = useGallerySelectionControllerState(runningInTauri);
  const {
    albums,
    setAlbums,
    selectedAlbumId,
    setSelectedAlbumId,
    albumGallery,
    setAlbumGallery,
    albumSearchInput,
    setAlbumSearchInput,
    albumNameInput,
    setAlbumNameInput,
    albumCreateOpen,
    setAlbumCreateOpen,
    albumLoading,
    setAlbumLoading,
    albumAddDrawerOpen,
    setAlbumAddDrawerOpen,
    albumAddQuery,
    setAlbumAddQuery,
    albumAddGallery,
    setAlbumAddGallery,
    albumAddSelectionIds,
    setAlbumAddSelectionIds,
    albumAddSubmitting,
    setAlbumAddSubmitting,
  } = useAlbumControllerState(runningInTauri);
  const {
    suggestions,
    setSuggestions,
    selectedSuggestionId,
    setSelectedSuggestionId,
    selectedSuggestionIds,
    setSelectedSuggestionIds,
    suggestionHistory,
    setSuggestionHistory,
    suggestionRegenerating,
    setSuggestionRegenerating,
    reviewForm,
    setReviewForm,
  } = useReviewControllerState(runningInTauri);
  const {
    taskDrafts,
    setTaskDrafts,
    tasks,
    setTasks,
    selectedTaskId,
    setSelectedTaskId,
    taskDetail,
    setTaskDetail,
    tasksLoading,
    setTasksLoading,
    daemonOnline,
    setDaemonOnline,
    pendingTaskActions,
    setPendingTaskActions,
    prompt,
    setPrompt,
    provider,
    setProvider,
    composerOpen,
    setComposerOpen,
    composerInputVersionId,
    setComposerInputVersionId,
    composerInputFile,
    setComposerInputFile,
    composerInputFileName,
    setComposerInputFileName,
    generationSubmitting,
    setGenerationSubmitting,
    activeTaskPanel,
    setActiveTaskPanel,
  } = useTaskGenerationControllerState(runningInTauri);
  const {
    status,
    setStatus,
    recoverableError,
    setRecoverableError,
    appLogs,
    setAppLogs,
    logsLoading,
    setLogsLoading,
    selectedLogPath,
    setSelectedLogPath,
    selectedLogContent,
    setSelectedLogContent,
    logContentLoading,
    setLogContentLoading,
    updateState,
    setUpdateState,
  } = useAppOperationsControllerState(runningInTauri);
  const logReadRequestRef = useRef<string | null>(null);
  const completedTaskKeysRef = useRef<Set<string>>(new Set());
  const metadataPollTimeoutsRef = useRef<Set<number>>(new Set());
  const {
    refreshLibraries,
    refreshLibraryStatus,
    createLibrary,
    openExistingLibraryFromPrompt,
    renameLibraryAlias,
    unregisterLibrary,
    exportLibraryBackup,
    importLibraryBackup,
    revealLibraryFolder,
  } = useLibrarySettingsActions({
    runningInTauri,
    library,
    query,
    libraryFolderNameInput,
    libraryNameInput,
    setLibraries,
    setLibrary,
    setLibraryStatus,
    setProviderHealth,
    setGallery,
    setSelectedGalleryAssetIds,
    setSelectedAssetId,
    setDetailState,
    setAlbums,
    setSelectedAlbumId,
    setAlbumSearchInput,
    setAlbumNameInput,
    setAlbumCreateOpen,
    setSuggestions,
    setSelectedSuggestionId,
    setSelectedSuggestionIds,
    setSuggestionHistory,
    setReviewForm,
    setTasks,
    setSelectedTaskId,
    setTaskDetail,
    setQuery,
    setLibraryFolderNameInput,
    setStatus,
    setRecoverableError,
    setPendingLibraryActions,
    setMissingLibraryPaths,
  });
  const {
    selectGalleryAsset,
    selectAssetVersion,
    refreshGallery,
    loadAssetDetail,
    loadPreviewAssetDetail,
  } = useGallerySelectionActions({
    library,
    query,
    detailState,
    setGallery,
    setSelectedAssetId,
    setDetailState,
    setInspectorOpen,
    setStatus,
    setRecoverableError,
  });
  const {
    refreshAppLogs,
    readAppLog,
    checkForAppUpdate,
    installAppUpdate,
    restartApp,
  } = useAppOperationsActions({
    runningInTauri,
    logReadRequestRef,
    setAppLogs,
    setLogsLoading,
    setSelectedLogPath,
    setSelectedLogContent,
    setLogContentLoading,
    setUpdateState,
    setStatus,
    setRecoverableError,
  });
  const {
    refreshDaemonHealth,
    refreshTasks,
    loadTaskDetail,
    waitForMetadataFieldResult,
    waitForMetadataSuggestionResult,
    startGeneration,
    openComposerForTextGeneration,
    openComposerForVersionGeneration,
    openComposerForReferenceGeneration,
    enqueueTaskDrafts,
    reorderQueuedTask,
    runTaskAction,
  } = useTaskGenerationActions({
    runningInTauri,
    library,
    prompt,
    provider,
    taskDrafts,
    tasks,
    selectedTaskId,
    generationSubmitting,
    pendingTaskActions,
    completedTaskKeysRef,
    metadataPollTimeoutsRef,
    setDaemonOnline,
    setTasks,
    setSelectedTaskId,
    setTaskDetail,
    setTasksLoading,
    setPendingTaskActions,
    setTaskDrafts,
    setPrompt,
    setComposerOpen,
    setComposerInputVersionId,
    setComposerInputFile,
    setComposerInputFileName,
    setGenerationSubmitting,
    setActiveView,
    setActiveTaskPanel,
    setStatus,
    setRecoverableError,
    refreshGallery,
    refreshSuggestions,
    detailState,
    loadAssetDetail,
  });


  const {
    displayedGallery,
    selectedAsset,
    availableTags,
    availableCategories,
    availableProviders,
  } = useGalleryDerivedState({
    runningInTauri,
    gallery,
    previewGallery: mockGallery,
    query,
    selectedAssetId,
  });
  const selectedAlbum = useMemo(
    () => albums.find((album) => album.id === selectedAlbumId) ?? null,
    [albums, selectedAlbumId],
  );
  const { pendingSuggestions, selectedSuggestion } = useReviewDerivedState(suggestions, selectedSuggestionId);
  const { queueCount, runningTaskCount, failedTaskCount } = useTaskActivitySummary(tasks);
  const {
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
  } = useReviewActions({
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
  });
  const changeView = (view: View) => {
    setActiveView(view);
    setSidebarExpanded(false);
  };
  const selectTask = (taskId: string) => {
    setSelectedTaskId(taskId);
    setActiveTaskPanel("detail");
  };

  useStudioRefreshPolicy({
    runningInTauri,
    library,
    activeView,
    selectedTaskId,
    selectedSuggestion,
    selectedAsset,
    albumAddDrawerOpen,
    albumAddQuery,
    galleryQuery: query,
    selectedAlbumId,
    selectedAlbumKind: selectedAlbum?.kind,
    albums,
    reviewFormSuggestionId: reviewForm?.suggestionId ?? null,
    selectedAssetCurrentVersionId: selectedAsset?.currentVersionId,
    metadataPollTimeoutsRef,
    completedTaskKeysRef,
    refreshLibraries,
    checkForAppUpdate,
    refreshGallery,
    refreshSelectedAlbumContents,
    refreshAlbumAddCandidates,
    refreshAlbums,
    refreshSuggestions,
    refreshDaemonHealth,
    refreshTasks,
    loadTaskDetail,
    refreshAppLogs,
    refreshSuggestionHistory,
    loadAssetDetail,
    loadPreviewAssetDetail,
    setTaskDetail,
    setSelectedSuggestionId,
    setReviewForm,
    setSuggestionHistory,
    setDetailState,
  });

  async function refreshAlbums() {
    if (!runningInTauri || !library) {
      setAlbums([]);
      return;
    }
    setAlbumLoading(true);
    try {
      const items = await invokeCommand<AlbumListItem[]>("list_albums", { libraryPath: library.rootPath });
      setAlbums(items);
      setSelectedAlbumId((current) => (current && items.some((item) => item.id === current) ? current : null));
      setRecoverableError(null);
    } catch (error) {
      setAlbums([]);
      setRecoverableError(errorMessage(error));
    } finally {
      setAlbumLoading(false);
    }
  }

  async function refreshSelectedAlbumContents(albumId = selectedAlbumId) {
    const album = albums.find((item) => item.id === albumId) ?? null;
    if (!albumId || !album) {
      setAlbumGallery([]);
      return;
    }
    const contentsQuery = albumContentsQuery(albumId, album.kind);
    if (!runningInTauri || !library) {
      const source = applyPreviewGalleryQuery(contentsQuery);
      setAlbumGallery(source);
      return;
    }
    try {
      const items = await invokeCommand<GalleryAsset[]>("query_gallery", {
        input: galleryQueryInput(library.rootPath, contentsQuery),
      });
      setAlbumGallery(items);
      setRecoverableError(null);
    } catch (error) {
      setAlbumGallery([]);
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshAlbumAddCandidates(nextQuery = albumAddQuery) {
    if (!selectedAlbumId) {
      setAlbumAddGallery([]);
      return;
    }
    if (!runningInTauri || !library) {
      setAlbumAddGallery(filterAlbumAddCandidates(applyPreviewGalleryQuery(nextQuery), selectedAlbumId));
      return;
    }
    try {
      const items = await invokeCommand<GalleryAsset[]>("query_gallery", {
        input: galleryQueryInput(library.rootPath, nextQuery),
      });
      setAlbumAddGallery(filterAlbumAddCandidates(items, selectedAlbumId));
      setRecoverableError(null);
    } catch (error) {
      setAlbumAddGallery([]);
      setRecoverableError(errorMessage(error));
    }
  }

  function applyPreviewGalleryQuery(nextQuery: GalleryQueryState) {
    return applyGalleryQuery(mockGallery, nextQuery);
  }

  async function refreshSuggestions() {
    if (!runningInTauri || !library) {
      setSuggestions([]);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      return;
    }
    try {
      const items = await invokeCommand<Suggestion[]>("list_pending_suggestions", { libraryPath: library.rootPath });
      setSuggestions(items);
      setSelectedSuggestionId((current) => (current && items.some((item) => item.id === current) ? current : items[0]?.id ?? null));
      setRecoverableError(null);
    } catch (error) {
      setSuggestions([]);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshSuggestionHistory(suggestion: Suggestion) {
    if (!runningInTauri || !library) {
      setSuggestionHistory(
        mockSuggestions.filter((item) => item.assetId === suggestion.assetId),
      );
      return;
    }
    try {
      const history = await invokeCommand<Suggestion[]>("list_suggestion_history", {
        input: {
          libraryPath: library.rootPath,
          assetId: suggestion.assetId,
        },
      });
      setSuggestionHistory(history);
      setRecoverableError(null);
    } catch (error) {
      setSuggestionHistory([]);
      setRecoverableError(errorMessage(error));
    }
  }

  function switchLibrary(libraryId: string) {
    const nextLibrary = libraries.find((item) => item.id === libraryId) ?? null;
    const cleared = clearCurationStateForLibrarySwitch();
    setLibrary(nextLibrary);
    setSelectedAssetId("");
    setDetailState(clearSelectionForLibrarySwitch());
    setGallery([]);
    setAlbums(runningInTauri ? [] : mockAlbumList);
    setSelectedAlbumId(cleared.selectedAlbumId);
    setAlbumGallery([]);
    setAlbumSearchInput("");
    setAlbumNameInput("");
    setAlbumCreateOpen(false);
    setAlbumAddDrawerOpen(false);
    setAlbumAddQuery(defaultAlbumAddSourceQuery());
    setAlbumAddGallery([]);
    setAlbumAddSelectionIds([]);
    setSelectedSuggestionId(cleared.selectedSuggestionId);
    setSelectedSuggestionIds([]);
    setSuggestionHistory([]);
    setTasks(runningInTauri ? [] : mockTasks);
    setSelectedTaskId(null);
    setTaskDetail(null);
    setReviewForm(cleared.reviewForm);
    setSuggestions(runningInTauri ? [] : mockSuggestions);
    setQuery(resetGalleryQuery());
    setRecoverableError(null);
    setStatus(nextLibrary ? "Library switched" : "No library selected");
    if (runningInTauri && nextLibrary) {
      void refreshLibraryStatus(nextLibrary.rootPath);
    } else {
      setLibraryStatus(nextLibrary ? mockLibraryStatus : null);
    }
  }

  async function updateRating(rating: number) {
    const detail = detailState.detail;
    if (!library || !detail) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("update_asset_metadata", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          rating,
          category: detail.category,
          status: detail.status,
        },
      });
      setGallery((current) =>
        current.map((item) => (item.id === asset.id ? { ...item, rating: asset.rating } : item)),
      );
      setDetailState((current) =>
        current.detail ? { ...current, detail: { ...current.detail, rating: asset.rating } } : current,
      );
      setStatus("Rating updated");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function updateTitle(title: string) {
    const detail = detailState.detail;
    const trimmed = title.trim();
    if (!library || !detail || trimmed.length === 0 || trimmed === detail.title) {
      return;
    }
    try {
      const asset = await invokeCommand<AssetView>("update_asset_metadata", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          title: trimmed,
        },
      });
      setGallery((current) =>
        current.map((item) => (item.id === asset.id ? { ...item, title: asset.title } : item)),
      );
      setDetailState((current) =>
        current.detail ? { ...current, detail: { ...current.detail, title: asset.title } } : current,
      );
      setStatus("Title updated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function addTagToSelectedAsset(tag: string) {
    const detail = detailState.detail;
    const trimmed = tag.trim();
    if (!library || !detail || trimmed.length === 0) {
      return;
    }
    try {
      await invokeCommand("add_tag_to_asset", {
        input: {
          libraryPath: library.rootPath,
          assetId: detail.id,
          tag: trimmed,
        },
      });
      await Promise.all([
        refreshGallery(),
        loadAssetDetail(detail.id, selectedAsset?.currentVersionId ?? null),
      ]);
      setStatus("Tag added");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function createAlbum() {
    const name = albumNameInput.trim();
    if (!library || name.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const created = createPreviewAlbum(`album-${crypto.randomUUID()}`, name, "manual", albums.length + 1);
      setAlbums((current) => [created, ...current]);
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      setSelectedAlbumId(created.id);
      setAlbumGallery([]);
      setStatus("Album created");
      setRecoverableError(null);
      return;
    }
    try {
      const created = await invokeCommand<Album>("create_manual_album", {
        input: {
          libraryPath: library.rootPath,
          name,
        },
      });
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      await refreshAlbums();
      setSelectedAlbumId(created.id);
      setAlbumGallery([]);
      setStatus("Album created");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function createSmartAlbum(name: string, smartQueryJson: string) {
    const trimmed = name.trim();
    if (!library || trimmed.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const created = createPreviewAlbum(`album-${crypto.randomUUID()}`, trimmed, "smart", albums.length + 1);
      setAlbums((current) => [created, ...current]);
      setSelectedAlbumId(created.id);
      setAlbumGallery([]);
      return;
    }
    try {
      const created = await invokeCommand<Album>("create_smart_album", {
        input: {
          libraryPath: library.rootPath,
          name: trimmed,
          smartQueryJson,
        },
      });
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      await refreshAlbums();
      setSelectedAlbumId(created.id);
      setAlbumGallery([]);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function openAlbum(albumId: string) {
    setSelectedAlbumId(albumId);
    setActiveView("albums");
  }

  function closeAlbum() {
    setSelectedAlbumId(null);
    setAlbumGallery([]);
    setAlbumAddDrawerOpen(false);
    setAlbumAddSelectionIds([]);
  }

  async function renameAlbum(albumId: string, name: string) {
    const trimmed = name.trim();
    if (!library || trimmed.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => renameAlbumState(current, albumId, trimmed));
      return;
    }
    try {
      await invokeCommand<Album>("rename_album", { input: { albumId, name: trimmed } });
      await refreshAlbums();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function deleteAlbumById(albumId: string) {
    if (!library) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => removeAlbumState(current, albumId));
      if (selectedAlbumId === albumId) {
        closeAlbum();
      }
      return;
    }
    try {
      await invokeCommand("delete_album", { albumId });
      if (selectedAlbumId === albumId) {
        closeAlbum();
      }
      await Promise.all([refreshAlbums(), refreshGallery()]);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function reorderAlbumsByIds(albumIds: string[]) {
    const next = reorderByIds(albums, albumIds);
    setAlbums(next);
    if (!runningInTauri || !library) {
      return;
    }
    try {
      await invokeCommand("reorder_albums", {
        input: {
          libraryPath: library.rootPath,
          albumIds: next.map((album) => album.id),
        },
      });
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshAlbums();
    }
  }

  async function removeAssetFromSelectedAlbum(assetId: string) {
    if (!library || !selectedAlbumId) {
      return;
    }
    if (!runningInTauri) {
      setAlbumGallery((current) => current.filter((asset) => asset.id !== assetId));
      return;
    }
    try {
      await invokeCommand("remove_asset_from_album", {
        input: {
          albumId: selectedAlbumId,
          assetId,
        },
      });
      await Promise.all([refreshAlbums(), refreshSelectedAlbumContents(), refreshGallery()]);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function reorderSelectedAlbumAssets(assetIds: string[]) {
    if (!library || !selectedAlbumId) {
      return;
    }
    setAlbumGallery((current) => reorderByIds(current, assetIds));
    if (!runningInTauri) {
      return;
    }
    try {
      await invokeCommand("reorder_album_items", {
        input: {
          albumId: selectedAlbumId,
          assetIds,
        },
      });
      await refreshSelectedAlbumContents();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshSelectedAlbumContents();
    }
  }

  async function addSelectedGalleryAssetsToAlbum(albumId: string) {
    if (!library || selectedGalleryAssetIds.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => incrementAlbumItemCount(current, albumId, selectedGalleryAssetIds.length));
      if (selectedAlbumId === albumId) {
        setAlbumGallery(filterAlbumAddCandidates(mockGallery, null).filter((asset) => selectedGalleryAssetIds.includes(asset.id)));
      }
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId,
          assetIds: selectedGalleryAssetIds,
        },
      });
      await Promise.all([refreshAlbums(), refreshSelectedAlbumContents(albumId), refreshGallery()]);
      setRecoverableError(null);
      setStatus("Selected assets added to album");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function openAlbumAddDrawer() {
    const nextQuery = defaultAlbumAddSourceQuery();
    setAlbumAddQuery(nextQuery);
    setAlbumAddSelectionIds([]);
    setAlbumAddDrawerOpen(true);
    void refreshAlbumAddCandidates(nextQuery);
  }

  function closeAlbumAddDrawer() {
    setAlbumAddDrawerOpen(false);
    setAlbumAddSelectionIds([]);
    setAlbumAddGallery([]);
  }

  function updateAlbumAddQuery(nextQuery: GalleryQueryState) {
    setAlbumAddQuery(nextQuery);
  }

  async function submitAlbumAddSelection() {
    if (!library || !selectedAlbumId || albumAddSelectionIds.length === 0) {
      return;
    }
    setAlbumAddSubmitting(true);
    if (!runningInTauri) {
      setAlbums((current) => incrementAlbumItemCount(current, selectedAlbumId, albumAddSelectionIds.length));
      setAlbumGallery((current) => [
        ...current,
        ...mockGallery.filter((asset) => albumAddSelectionIds.includes(asset.id)),
      ]);
      closeAlbumAddDrawer();
      setAlbumAddSubmitting(false);
      setStatus("Images added to album");
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId: selectedAlbumId,
          assetIds: albumAddSelectionIds,
        },
      });
      await Promise.all([
        refreshAlbums(),
        refreshSelectedAlbumContents(selectedAlbumId),
        refreshAlbumAddCandidates(),
        refreshGallery(),
      ]);
      setAlbumAddSelectionIds([]);
      setAlbumAddDrawerOpen(false);
      setStatus("Images added to album");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setAlbumAddSubmitting(false);
    }
  }

  async function addSelectedAssetToAlbum(albumId: string) {
    const detail = detailState.detail;
    if (!library || !detail || albumId.length === 0) {
      return;
    }
    if (!runningInTauri) {
      const album = albums.find((item) => item.id === albumId);
      if (!album) {
        return;
      }
      const alreadyInAlbum = detail.albums.some((item) => item.id === albumId);
      setDetailState((current) =>
        current.detail
          ? {
              ...current,
              detail: {
                ...current.detail,
                albums: alreadyInAlbum
                  ? current.detail.albums
                  : [...current.detail.albums, { id: album.id, name: album.name, kind: album.kind }],
              },
            }
          : current,
      );
      if (!alreadyInAlbum) {
        setAlbums((current) => incrementAlbumItemCount(current, albumId, 1));
      }
      setStatus(alreadyInAlbum ? "Asset already in album" : "Asset added to album");
      setRecoverableError(null);
      return;
    }
    try {
      await invokeCommand("add_asset_to_album", {
        input: {
          albumId,
          assetId: detail.id,
        },
      });
      await Promise.all([
        refreshAlbums(),
        refreshGallery(),
        refreshSelectedAlbumContents(albumId),
        loadAssetDetail(detail.id, selectedAsset?.currentVersionId ?? null),
      ]);
      setStatus("Asset added to album");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function promoteFocusedVersionAsAsset(versionId: string | null | undefined) {
    if (!library || !versionId) {
      setRecoverableError("Select an asset version before promoting it.");
      return;
    }
    if (!runningInTauri) {
      setRecoverableError("Promote as new asset requires a real library.");
      return;
    }
    try {
      const promoted = await invokeCommand<PromoteAssetVersionResult>("promote_asset_version", {
        input: {
          libraryPath: library.rootPath,
          sourceVersionId: versionId,
        },
      });
      await refreshGallery();
      setSelectedAssetId(promoted.asset.id);
      await loadAssetDetail(promoted.asset.id, promoted.version.id);
      setInspectorOpen(true);
      setStatus("Version promoted as new asset");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  const detail = detailState.detail;
  const composerInputVersion =
    composerInputVersionId && detail
      ? (detail.versions.find((version) => version.id === composerInputVersionId) ??
        detail.lineage.find((entry) => entry.version.id === composerInputVersionId)?.version ??
        null)
      : null;
  const composerInputVersionName =
    composerInputVersion?.versionName ??
    (composerInputVersionId === selectedAsset?.currentVersionId ? (selectedAsset.currentVersionName ?? null) : null) ??
    (composerInputVersionId ? shortIdentifier(composerInputVersionId) : null);
  const composerInputSourceName = composerInputVersionName ?? composerInputFileName;

  const sidebarSlot = (
    <Sidebar
      library={library}
      libraries={libraries}
      libraryStatus={libraryStatus}
      albums={albums}
      selectedAlbumId={selectedAlbumId}
      albumSearchValue={albumSearchInput}
      settingsSection={settingsSection}
      activeView={activeView}
      reviewCount={pendingSuggestions.length}
      queueCount={queueCount}
      expanded={sidebarExpanded}
      onExpandedChange={setSidebarExpanded}
      onViewChange={changeView}
      onLibraryChange={switchLibrary}
      onAlbumSearchChange={setAlbumSearchInput}
      onCreateAlbumClick={() => {
        setActiveView("albums");
        setAlbumCreateOpen(true);
      }}
      onCloseAlbum={closeAlbum}
      onOpenAlbum={openAlbum}
      onSettingsSectionChange={setSettingsSection}
    />
  );
  const workspaceSlot = (
    <>
        <WorkspaceToolbar
          activeView={activeView}
          query={query}
          itemCount={displayedGallery.length}
          status={status}
          composerOpen={composerOpen}
          availableProviders={availableProviders}
          albums={albums}
          onComposerOpenChange={openComposerForTextGeneration}
          onQueryChange={setQuery}
        />

        <StudioOverviewBand
          assetCount={displayedGallery.length}
          reviewCount={pendingSuggestions.length}
          queueCount={queueCount}
          integrityIssueCount={libraryStatus?.integrityIssueCount ?? 0}
          activeView={activeView}
        />

        {composerOpen && (
          <GenerationComposer
            prompt={prompt}
            provider={provider}
            inputSourceName={composerInputSourceName}
            submitting={generationSubmitting}
            onPromptChange={setPrompt}
            onProviderChange={setProvider}
            onGenerate={() => void startGeneration(composerInputVersionId, composerInputFile)}
          />
        )}

        {recoverableError && (
          <div className="inline-error">
            <span>{recoverableError}</span>
            <button onClick={() => setRecoverableError(null)}>Dismiss</button>
          </div>
        )}

        {activeView === "gallery" && (
          <GalleryWorkspace
            assets={displayedGallery}
            selectedAssetId={selectedAsset?.id ?? ""}
            selectedAssetIds={selectedGalleryAssetIds}
            query={query}
            availableTags={availableTags}
            onSelect={selectGalleryAsset}
            onToggleAssetSelection={(assetId) => setSelectedGalleryAssetIds((current) => toggleSelection(current, assetId))}
            onQueryChange={setQuery}
            onRequestReview={(asset) => void requestAssetReview(asset)}
            onPreviewImage={setLightboxImage}
          />
        )}
        {activeView === "albums" && (
          <AlbumsWorkspace
            albums={albums}
            availableTags={availableTags}
            availableCategories={availableCategories}
            availableProviders={availableProviders}
            selectedAlbumId={selectedAlbumId}
            gallery={albumGallery}
            loading={albumLoading}
            searchValue={albumSearchInput}
            onSearchChange={setAlbumSearchInput}
            newAlbumName={albumNameInput}
            onNewAlbumNameChange={setAlbumNameInput}
            createOpen={albumCreateOpen}
            onCreateOpenChange={setAlbumCreateOpen}
            onCreateAlbum={() => void createAlbum()}
            onCreateSmartAlbum={(name, queryJson) => void createSmartAlbum(name, queryJson)}
            onOpenAlbum={openAlbum}
            onCloseAlbum={closeAlbum}
            onRenameAlbum={(albumId, name) => void renameAlbum(albumId, name)}
            onDeleteAlbum={(albumId) => void deleteAlbumById(albumId)}
            onReorderAlbums={(albumIds) => void reorderAlbumsByIds(albumIds)}
            onRemoveAsset={(assetId) => void removeAssetFromSelectedAlbum(assetId)}
            onReorderAssets={(assetIds) => void reorderSelectedAlbumAssets(assetIds)}
            addDrawerOpen={albumAddDrawerOpen}
            addQuery={albumAddQuery}
            addGallery={albumAddGallery}
            addSelectionIds={albumAddSelectionIds}
            addSubmitting={albumAddSubmitting}
            onOpenAddDrawer={openAlbumAddDrawer}
            onCloseAddDrawer={closeAlbumAddDrawer}
            onAddQueryChange={updateAlbumAddQuery}
            onToggleAddSelection={(assetId) => setAlbumAddSelectionIds((current) => toggleSelection(current, assetId))}
            onSubmitAddSelection={() => void submitAlbumAddSelection()}
            onSelectAsset={selectGalleryAsset}
          />
        )}
        {activeView === "review" && (
          <ReviewWorkspace
            suggestions={pendingSuggestions}
            selectedSuggestion={selectedSuggestion}
            selectedSuggestionIds={selectedSuggestionIds}
            suggestionHistory={suggestionHistory}
            suggestionRegenerating={suggestionRegenerating}
            form={reviewForm}
            onSelect={selectSuggestion}
            onToggleSelected={toggleSuggestionForBatch}
            onFormChange={setReviewForm}
            availableTags={availableTags}
            availableCategories={availableCategories}
            albums={albums}
            tasks={tasks}
            onRestore={restoreReviewForm}
            onRegenerateField={(field) => void regenerateReviewField(field)}
            onRegenerateSuggestion={() => void regenerateFullSuggestion()}
            onPickHistoryField={pickReviewHistoryField}
            onAccept={() => void acceptReviewForm()}
            onBatchAccept={() => void batchAcceptReviewSuggestions()}
            onBatchReject={() => void batchRejectReviewSuggestions()}
            onAddToAlbum={(albumId) => void addReviewSelectionToAlbum(albumId)}
            onOpenTask={(taskId) => {
              selectTask(taskId);
              changeView("queue");
            }}
          />
        )}
        {activeView === "queue" && (
          <div className="workspace-fill">
            <TaskWorkspace
              drafts={taskDrafts}
              tasks={tasks}
              selectedTaskId={selectedTaskId}
              detail={taskDetail}
              loading={tasksLoading}
              daemonOnline={daemonOnline}
              pendingTaskActions={pendingTaskActions}
              activePanel={activeTaskPanel}
              onActivePanelChange={setActiveTaskPanel}
              onDraftsChange={setTaskDrafts}
              onAddDraft={() => setTaskDrafts((current) => [...current, createTaskDraft()])}
              onEnqueue={() => void enqueueTaskDrafts()}
              onRefresh={() => void refreshTasks()}
              onSelectTask={selectTask}
              onMoveTask={(taskId, direction) => void reorderQueuedTask(taskId, direction)}
              onCancel={(taskId) => void runTaskAction("cancel_daemon_task", taskId)}
              onRetry={(taskId) => void runTaskAction("retry_daemon_task", taskId)}
              onDuplicate={(taskId) => void runTaskAction("duplicate_daemon_task", taskId)}
            />
          </div>
        )}
        {activeView === "settings" && (
          <SettingsWorkspace
            library={library}
            libraries={libraries}
            activeSection={settingsSection}
            providerHealth={providerHealth}
            daemonOnline={daemonOnline}
            libraryStatus={libraryStatus}
            onSectionChange={setSettingsSection}
            libraryFolderName={libraryFolderNameInput}
            libraryName={libraryNameInput}
            onLibraryFolderNameChange={setLibraryFolderNameInput}
            onLibraryNameChange={setLibraryNameInput}
            onCreate={createLibrary}
            onOpenExisting={openExistingLibraryFromPrompt}
            onImportZip={() => void importLibraryBackup()}
            onSwitchLibrary={switchLibrary}
            onRenameLibrary={(item) => void renameLibraryAlias(item)}
            onCloseLibrary={(item) => void unregisterLibrary(item)}
            onExportZip={(item) => void exportLibraryBackup(item)}
            onReveal={(item) => void revealLibraryFolder(item)}
            pendingLibraryActions={pendingLibraryActions}
            missingLibraryPaths={missingLibraryPaths}
            logs={appLogs}
            logsLoading={logsLoading}
            selectedLogPath={selectedLogPath}
            selectedLogContent={selectedLogContent}
            logContentLoading={logContentLoading}
            updateState={updateState}
            onRefreshLogs={() => void refreshAppLogs()}
            onSelectLog={(path) => void readAppLog(path)}
            onCheckUpdate={() => void checkForAppUpdate()}
            onInstallUpdate={() => void installAppUpdate()}
            onRestartApp={() => void restartApp()}
          />
        )}
      </>
  );
  const inspectorSlot = (
      <>
        <button className="inspector-toggle" onClick={() => setInspectorOpen(!inspectorOpen)}>
          <Icon name="panelRight" />
          <span>{inspectorOpen ? "Close detail" : "Open detail"}</span>
        </button>
        <Inspector
          asset={selectedAsset}
          detailState={detailState}
          onClose={() => setInspectorOpen(false)}
          onUpdateRating={updateRating}
          onUpdateTitle={(title) => void updateTitle(title)}
          onAddTag={(tag) => void addTagToSelectedAsset(tag)}
          albums={albums}
          onAddToAlbum={(albumId) => void addSelectedAssetToAlbum(albumId)}
          onSelectVersion={selectAssetVersion}
          onPreviewImage={setLightboxImage}
          onGenerateFromReference={openComposerForReferenceGeneration}
          onGenerateVariation={(versionId) => {
            versionId =
              versionId ??
              detail?.focusedVersionId ??
              detail?.currentVersionId ??
              detail?.lineage[0]?.version.id ??
              detail?.versions[0]?.id ??
              selectedAsset?.currentVersionId ??
              null;
            openComposerForVersionGeneration(versionId);
          }}
          onPromoteVersion={(versionId) => void promoteFocusedVersionAsAsset(versionId)}
        />
      </>
  );
  const activityStripSlot = (
    <ActivityStrip>
      <button className="activity-item" onClick={() => changeView("queue")}>
        <Icon name="plus" />
        <span>
          <strong>{runningTaskCount}</strong>
          <small>Running</small>
        </span>
      </button>
      <button className="activity-item" onClick={() => changeView("review")}>
        <Icon name="panelRight" />
        <span>
          <strong>{pendingSuggestions.length}</strong>
          <small>Review</small>
        </span>
      </button>
      <button className={failedTaskCount > 0 ? "activity-item danger" : "activity-item"} onClick={() => changeView("queue")}>
        <Icon name="list" />
        <span>
          <strong>{failedTaskCount}</strong>
          <small>Failed</small>
        </span>
      </button>
    </ActivityStrip>
  );

  return (
    <AppShell
      sidebar={sidebarSlot}
      workspace={workspaceSlot}
      inspector={inspectorSlot}
      sidebarExpanded={sidebarExpanded}
      inspectorExpanded={inspectorOpen}
      activityStrip={activityStripSlot}
    >
      {lightboxImage && <ImageLightbox image={lightboxImage} onClose={() => setLightboxImage(null)} />}
    </AppShell>
  );
}
