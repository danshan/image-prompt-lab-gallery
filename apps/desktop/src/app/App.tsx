import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  acceptSuggestionState,
  addReviewFormTag,
  applySuggestionFieldToReviewForm,
  beginDetailLoad,
  beginReviewFieldGeneration,
  buildBatchReviewPayloads,
  clearAlbumQuery,
  clearCurationStateForLibrarySwitch,
  clearLibraryWorkspaceState,
  clearSelectionForLibrarySwitch,
  completeDetailLoad,
  completeReviewFieldGeneration,
  createReviewFormState,
  defaultSettingsSection,
  defaultGalleryQuery,
  failDetailLoad,
  failReviewFieldGeneration,
  formatAspectRatio,
  isReviewFieldGenerating,
  libraryMaintenanceActions,
  markAssetReviewPending,
  moveItem,
  moveQueuedTaskOrder,
  openAlbumQuery,
  parseTaskDraftImport,
  removeSuggestionState,
  removeReviewFormTag,
  reorderByIds,
  resetGalleryQuery,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleGalleryProvider,
  toggleGalleryTag,
  toggleSelection,
  updateGalleryQuery,
  type DetailLoadState,
  type GalleryQueryState,
  type GallerySort,
  type ReviewFieldName,
  type ReviewFormState,
  type ReviewStatusFilter,
  type SettingsSection,
} from "../workbench-state";
import { ActivityStrip, AppShell } from "../studio-shell";
import { Icon } from "../studio-icons";
import { Sidebar } from "../studio-navigation";
import { useGalleryDerivedState } from "./hooks/gallery";
import { useReviewDerivedState } from "./hooks/review";
import { useTaskActivitySummary } from "./hooks/tasks";
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
  mockDetailFor,
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
  GALLERY_REFRESH_DEBOUNCE_MS,
  METADATA_POLL_INTERVAL_MS,
  TASK_QUEUE_BACKGROUND_POLL_INTERVAL_MS,
  TASK_QUEUE_POLL_INTERVAL_MS,
  initialUpdateState,
} from "./types";
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

export function App() {
  const runningInTauri = hasTauriRuntime();
  const [activeView, setActiveView] = useState<View>("gallery");
  const [libraries, setLibraries] = useState<Library[]>(runningInTauri ? [] : mockLibraries);
  const [library, setLibrary] = useState<Library | null>(runningInTauri ? null : mockLibrary);
  const [libraryStatus, setLibraryStatus] = useState<LibraryStatus | null>(
    runningInTauri ? null : mockLibraryStatus,
  );
  const [providerHealth, setProviderHealth] = useState<ProviderHealth[]>(mockProviderHealth);
  const [gallery, setGallery] = useState<GalleryAsset[]>(runningInTauri ? [] : mockGallery);
  const [selectedGalleryAssetIds, setSelectedGalleryAssetIds] = useState<string[]>([]);
  const [query, setQuery] = useState<GalleryQueryState>(defaultGalleryQuery);
  const [selectedAssetId, setSelectedAssetId] = useState(runningInTauri ? "" : mockGallery[0].id);
  const [detailState, setDetailState] = useState<DetailLoadState<AssetDetail>>({
    assetId: runningInTauri ? null : mockDetail.id,
    detail: runningInTauri ? null : mockDetail,
    loading: false,
    error: null,
  });
  const [albums, setAlbums] = useState<AlbumListItem[]>(runningInTauri ? [] : mockAlbumList);
  const [selectedAlbumId, setSelectedAlbumId] = useState<string | null>(null);
  const [albumSearchInput, setAlbumSearchInput] = useState("");
  const [albumNameInput, setAlbumNameInput] = useState("");
  const [albumCreateOpen, setAlbumCreateOpen] = useState(false);
  const [albumLoading, setAlbumLoading] = useState(false);
  const [suggestions, setSuggestions] = useState<Suggestion[]>(runningInTauri ? [] : mockSuggestions);
  const [selectedSuggestionId, setSelectedSuggestionId] = useState<string | null>(
    runningInTauri ? null : mockSuggestions[0]?.id ?? null,
  );
  const [selectedSuggestionIds, setSelectedSuggestionIds] = useState<string[]>([]);
  const [suggestionHistory, setSuggestionHistory] = useState<Suggestion[]>([]);
  const [suggestionRegenerating, setSuggestionRegenerating] = useState(false);
  const [reviewForm, setReviewForm] = useState<ReviewFormState | null>(
    runningInTauri && mockSuggestions[0] ? null : createReviewFormState(mockSuggestions[0]),
  );
  const [taskDrafts, setTaskDrafts] = useState<TaskDraft[]>([createTaskDraft()]);
  const [tasks, setTasks] = useState<DaemonTask[]>(runningInTauri ? [] : mockTasks);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(runningInTauri ? null : mockTasks[0]?.id ?? null);
  const [taskDetail, setTaskDetail] = useState<DaemonTaskDetail | null>(null);
  const [tasksLoading, setTasksLoading] = useState(false);
  const [daemonOnline, setDaemonOnline] = useState(false);
  const [pendingTaskActions, setPendingTaskActions] = useState<string[]>([]);
  const [prompt, setPrompt] = useState("");
  const [provider, setProvider] = useState("codex-cli");
  const [composerOpen, setComposerOpen] = useState(false);
  const [composerInputVersionId, setComposerInputVersionId] = useState<string | null>(null);
  const [composerInputFile, setComposerInputFile] = useState<string>("");
  const [composerInputFileName, setComposerInputFileName] = useState<string | null>(null);
  const [generationSubmitting, setGenerationSubmitting] = useState(false);
  const [status, setStatus] = useState(runningInTauri ? "Open or create a library" : "Preview mode");
  const [recoverableError, setRecoverableError] = useState<string | null>(null);
  const [libraryFolderNameInput, setLibraryFolderNameInput] = useState("image-prompt-lab");
  const [libraryNameInput, setLibraryNameInput] = useState("Image Prompt Lab");
  const [settingsSection, setSettingsSection] = useState<SettingsSection>(defaultSettingsSection);
  const [pendingLibraryActions, setPendingLibraryActions] = useState<string[]>([]);
  const [missingLibraryPaths, setMissingLibraryPaths] = useState<string[]>([]);
  const [appLogs, setAppLogs] = useState<AppLog[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);
  const [selectedLogPath, setSelectedLogPath] = useState<string | null>(null);
  const [selectedLogContent, setSelectedLogContent] = useState<AppLogContent | null>(null);
  const [logContentLoading, setLogContentLoading] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState>(initialUpdateState);
  const [lightboxImage, setLightboxImage] = useState<LightboxImage | null>(null);
  const [sidebarExpanded, setSidebarExpanded] = useState(false);
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [activeTaskPanel, setActiveTaskPanel] = useState<TaskPanel>("queue");
  const logReadRequestRef = useRef<string | null>(null);
  const completedTaskKeysRef = useRef<Set<string>>(new Set());
  const metadataPollTimeoutsRef = useRef<Set<number>>(new Set());

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
  const { pendingSuggestions, selectedSuggestion } = useReviewDerivedState(suggestions, selectedSuggestionId);
  const { queueCount, runningTaskCount, failedTaskCount } = useTaskActivitySummary(tasks);
  const changeView = (view: View) => {
    setActiveView(view);
    setSidebarExpanded(false);
  };
  const selectGalleryAsset = (assetId: string) => {
    setSelectedAssetId(assetId);
    setInspectorOpen(true);
  };
  const selectAssetVersion = (versionId: string) => {
    const detail = detailState.detail;
    if (!detail) {
      return;
    }
    void loadAssetDetail(detail.id, versionId);
  };
  const selectTask = (taskId: string) => {
    setSelectedTaskId(taskId);
    setActiveTaskPanel("detail");
  };

  useEffect(() => {
    if (runningInTauri) {
      void refreshLibraries();
      void checkForAppUpdate({ silent: true });
    }
  }, [runningInTauri]);

  useEffect(() => {
    if (!runningInTauri || !library) {
      return;
    }
    const timer = window.setTimeout(() => {
      void refreshGallery();
    }, GALLERY_REFRESH_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [runningInTauri, library?.rootPath, query]);

  useEffect(() => {
    return () => {
      for (const timer of metadataPollTimeoutsRef.current) {
        window.clearTimeout(timer);
      }
      metadataPollTimeoutsRef.current.clear();
    };
  }, [library?.rootPath]);

  useEffect(() => {
    if (runningInTauri && library) {
      completedTaskKeysRef.current = new Set();
      void refreshAlbums();
      void refreshSuggestions();
      void refreshDaemonHealth();
      void refreshTasks();
    }
  }, [runningInTauri, library?.rootPath]);

  useEffect(() => {
    if (!runningInTauri || !library) {
      return;
    }
    const showLoading = activeView === "queue";
    const intervalMs = showLoading ? TASK_QUEUE_POLL_INTERVAL_MS : TASK_QUEUE_BACKGROUND_POLL_INTERVAL_MS;

    void refreshTasks({ showLoading });
    const timer = window.setInterval(() => {
      void refreshTasks({ showLoading });
      if (showLoading && selectedTaskId) {
        void loadTaskDetail(selectedTaskId);
      }
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [runningInTauri, library?.rootPath, activeView, selectedTaskId]);

  useEffect(() => {
    if (!selectedTaskId) {
      setTaskDetail(null);
      return;
    }
    void loadTaskDetail(selectedTaskId);
  }, [selectedTaskId]);

  useEffect(() => {
    if (activeView === "settings") {
      void refreshAppLogs();
    }
  }, [activeView, runningInTauri]);

  useEffect(() => {
    if (!selectedSuggestion) {
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setSuggestionHistory([]);
      return;
    }
    if (reviewForm?.suggestionId !== selectedSuggestion.id) {
      setSelectedSuggestionId(selectedSuggestion.id);
      setReviewForm(createReviewFormState(selectedSuggestion));
    }
    void refreshSuggestionHistory(selectedSuggestion);
  }, [selectedSuggestion?.id]);

  useEffect(() => {
    if (!selectedAsset) {
      setDetailState({ assetId: null, detail: null, loading: false, error: null });
      return;
    }
    if (runningInTauri && library) {
      void loadAssetDetail(selectedAsset.id, selectedAsset.currentVersionId);
    } else {
      setDetailState(completeDetailLoad(selectedAsset.id, mockDetailFor(selectedAsset)));
    }
  }, [runningInTauri, library?.rootPath, selectedAsset?.id, selectedAsset?.currentVersionId]);

  async function refreshLibraries() {
    try {
      const libraries = await invokeCommand<Library[]>("list_libraries", { includeHidden: false });
      const nextLibrary = libraries[0] ?? null;
      setLibraries(libraries);
      setLibrary(nextLibrary);
      setStatus(nextLibrary ? "Library opened" : "No library registered");
      if (nextLibrary) {
        void refreshLibraryStatus(nextLibrary.rootPath);
      }
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  function setLibraryActionPending(key: string, pending: boolean) {
    setPendingLibraryActions((current) => {
      if (pending) {
        return current.includes(key) ? current : [...current, key];
      }
      return current.filter((item) => item !== key);
    });
  }

  function rememberMissingLibraryPath(rootPath: string) {
    setMissingLibraryPaths((current) => (current.includes(rootPath) ? current : [...current, rootPath]));
  }

  function forgetMissingLibraryPath(rootPath: string) {
    setMissingLibraryPaths((current) => current.filter((path) => path !== rootPath));
  }

  function replaceRegisteredLibrary(updated: Library) {
    setLibraries((current) => current.map((item) => (item.id === updated.id ? updated : item)));
    setLibrary((current) => (current?.id === updated.id ? updated : current));
  }

  function clearCurrentLibraryContext() {
    const cleared = clearLibraryWorkspaceState<AssetDetail>();
    setLibrary(null);
    setLibraryStatus(null);
    setGallery([]);
    setSelectedGalleryAssetIds(cleared.selectedGalleryAssetIds);
    setSelectedAssetId(cleared.selectedAssetId);
    setDetailState(cleared.detailState);
    setAlbums([]);
    setSelectedAlbumId(cleared.selectedAlbumId);
    setAlbumSearchInput("");
    setAlbumNameInput("");
    setAlbumCreateOpen(false);
    setSuggestions([]);
    setSelectedSuggestionId(cleared.selectedSuggestionId);
    setSelectedSuggestionIds(cleared.selectedSuggestionIds);
    setSuggestionHistory([]);
    setReviewForm(cleared.reviewForm);
    setTasks([]);
    setSelectedTaskId(cleared.selectedTaskId);
    setTaskDetail(null);
    setQuery(clearAlbumQuery(query));
    setLibraryFolderNameInput("image-prompt-lab");
    setStatus("No library selected");
    setRecoverableError(null);
  }

  async function refreshLibraryStatus(rootPath: string) {
    try {
      const nextStatus = await invokeCommand<LibraryStatus>("library_status", { rootPath });
      setLibraryStatus(nextStatus);
      if (runningInTauri) {
        const overview = await invokeCommand<{ providerHealth: ProviderHealth[] }>("diagnostics_overview", { rootPath });
        setProviderHealth(overview.providerHealth);
      }
    } catch (error) {
      setLibraryStatus(null);
      setRecoverableError(errorMessage(error));
    }
  }

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

  async function refreshAppLogs() {
    setLogsLoading(true);
    try {
      if (!runningInTauri) {
        setAppLogs([]);
        setSelectedLogPath(null);
        setSelectedLogContent(null);
        setRecoverableError(null);
        return;
      }
      const logs = await invokeCommand<AppLog[]>("list_app_logs");
      setAppLogs(logs);
      setSelectedLogPath((current) => {
        const next = current && logs.some((log) => log.path === current) ? current : logs[0]?.path ?? null;
        if (!next) {
          setSelectedLogContent(null);
        } else if (next !== current) {
          void readAppLog(next);
        }
        return next;
      });
      setRecoverableError(null);
    } catch (error) {
      setAppLogs([]);
      setSelectedLogPath(null);
      setSelectedLogContent(null);
      setRecoverableError(errorMessage(error));
    } finally {
      setLogsLoading(false);
    }
  }

  async function readAppLog(path: string) {
    const requestId = crypto.randomUUID();
    logReadRequestRef.current = requestId;
    setSelectedLogPath(path);
    setLogContentLoading(true);
    try {
      if (!runningInTauri) {
        if (logReadRequestRef.current === requestId) {
          setSelectedLogContent(null);
        }
        setRecoverableError(null);
        return;
      }
      const content = await invokeCommand<AppLogContent>("read_app_log", { input: { path } });
      if (logReadRequestRef.current === requestId && content.path === path) {
        setSelectedLogContent(content);
      }
      setRecoverableError(null);
    } catch (error) {
      if (logReadRequestRef.current === requestId) {
        setSelectedLogContent(null);
      }
      setRecoverableError(errorMessage(error));
    } finally {
      if (logReadRequestRef.current === requestId) {
        setLogContentLoading(false);
      }
    }
  }

  async function checkForAppUpdate({ silent = false }: { silent?: boolean } = {}) {
    setUpdateState((current) => ({
      ...current,
      checking: true,
      error: null,
      status: "checking",
    }));
    try {
      if (!runningInTauri) {
        setUpdateState({
          ...initialUpdateState,
          lastCheckedAt: new Date().toISOString(),
          status: "upToDate",
        });
        return;
      }
      const result = await invokeCommand<UpdateCheck>("check_for_update");
      setUpdateState((current) => ({
        ...current,
        currentVersion: result.currentVersion,
        lastCheckedAt: new Date().toISOString(),
        checking: false,
        availableUpdate: result.update,
        error: null,
        status: result.available ? "available" : "upToDate",
      }));
      if (!silent) {
        setStatus(result.available ? `Update ${result.update?.version ?? ""} available` : "App is up to date");
      }
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        checking: false,
        lastCheckedAt: new Date().toISOString(),
        error: errorMessage(error),
        status: "error",
      }));
      if (!silent) {
        setStatus("Update check failed");
      }
    }
  }

  async function installAppUpdate() {
    setUpdateState((current) => ({
      ...current,
      installing: true,
      error: null,
      status: "installing",
    }));
    try {
      const result = await invokeCommand<{ installed: boolean; version: string | null }>("install_update");
      setUpdateState((current) => ({
        ...current,
        installing: false,
        pendingRestart: result.installed,
        availableUpdate: result.installed ? current.availableUpdate : null,
        error: null,
        status: result.installed ? "pendingRestart" : "upToDate",
      }));
      setStatus(result.installed ? `Update ${result.version ?? ""} installed` : "No update available");
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        installing: false,
        error: errorMessage(error),
        status: "error",
      }));
      setStatus("Update install failed");
    }
  }

  async function restartApp() {
    try {
      await invokeCommand<void>("restart_app");
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        error: errorMessage(error),
        status: "error",
      }));
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
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSelectedSuggestionId(cleared.selectedSuggestionId);
    setSelectedSuggestionIds([]);
    setSuggestionHistory([]);
    setTasks(runningInTauri ? [] : mockTasks);
    setSelectedTaskId(null);
    setTaskDetail(null);
    setReviewForm(cleared.reviewForm);
    setSuggestions(runningInTauri ? [] : mockSuggestions);
    setQuery(clearAlbumQuery(query));
    setRecoverableError(null);
    setStatus(nextLibrary ? "Library switched" : "No library selected");
    if (runningInTauri && nextLibrary) {
      void refreshLibraryStatus(nextLibrary.rootPath);
    } else {
      setLibraryStatus(nextLibrary ? mockLibraryStatus : null);
    }
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

  async function createLibrary() {
    const folderName = libraryFolderNameInput.trim();
    if (!validLibraryFolderName(folderName)) {
      setStatus("Folder name must not be empty or contain path separators");
      return;
    }
    try {
      const parentPath = await pickDirectory("Choose Library Parent Folder");
      if (!parentPath) {
        return;
      }
      const rootPath = buildChildPath(parentPath, folderName);
      const created = await invokeCommand<Library>("create_library", {
        input: {
          rootPath,
          name: libraryNameInput.trim() || "Image Prompt Lab",
        },
      });
      setLibraries((current) => [created, ...current.filter((item) => item.id !== created.id)]);
      setLibrary(created);
      setLibraryStatus(null);
      setGallery([]);
      setAlbums([]);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSuggestions([]);
      setSelectedAlbumId(null);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setSelectedAssetId("");
      setStatus("Library created");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function openExistingLibraryFromPrompt() {
    try {
      const selectedPath = await pickDirectory("Open Existing Library");
      if (!selectedPath) {
        return;
      }
      const opened = await invokeCommand<Library>("open_library", {
        rootPath: selectedPath,
      });
      setLibraries((current) => [opened, ...current.filter((item) => item.id !== opened.id)]);
      setLibrary(opened);
      forgetMissingLibraryPath(opened.rootPath);
      setLibraryStatus(null);
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setStatus("Library opened");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function renameLibraryAlias(item: Library) {
    const alias = window.prompt("Rename Library", item.name);
    if (alias === null) {
      return;
    }
    const actionKey = `rename:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      const updated = await invokeCommand<Library>("rename_library_alias", {
        input: {
          libraryId: item.id,
          alias,
        },
      });
      replaceRegisteredLibrary(updated);
      setStatus("Library renamed");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function unregisterLibrary(item: Library) {
    const confirmed = window.confirm("Close this library in the app? Files on disk are not deleted.");
    if (!confirmed) {
      return;
    }
    const actionKey = `close:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      await invokeCommand<void>("unregister_library", { libraryId: item.id });
      setLibraries((current) => current.filter((library) => library.id !== item.id));
      if (library?.id === item.id) {
        clearCurrentLibraryContext();
      } else {
        setStatus("Library closed");
      }
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function exportLibraryBackup(item: Library) {
    const defaultPath = `${item.rootPath.replace(/\/$/, "")}.zip`;
    const actionKey = `export:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      const outputZipPath = await pickSaveZipPath("Export Library Zip", defaultPath);
      if (!outputZipPath) {
        return;
      }
      await invokeCommand<void>("export_library_backup_zip", {
        input: {
          libraryPath: item.rootPath,
          outputZipPath,
        },
      });
      setStatus(`Library exported to ${outputZipPath}`);
      forgetMissingLibraryPath(item.rootPath);
    } catch (error) {
      const message = errorMessage(error);
      setStatus(message);
      if (message.includes("not found") || message.includes("missing")) {
        rememberMissingLibraryPath(item.rootPath);
      }
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function importLibraryBackup() {
    const actionKey = "import";
    setLibraryActionPending(actionKey, true);
    try {
      const zipPath = await pickZipFile("Import Library Zip");
      if (!zipPath) {
        return;
      }
      const destinationPath = await pickDirectory("Import Destination Folder");
      if (!destinationPath) {
        return;
      }
      const imported = await invokeCommand<LibraryBackup>("import_library_backup_zip", {
        input: {
          zipPath,
          destinationPath,
        },
      });
      setLibraries((current) => [imported.library, ...current.filter((item) => item.id !== imported.library.id)]);
      setLibrary(imported.library);
      forgetMissingLibraryPath(imported.library.rootPath);
      setLibraryStatus(null);
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setStatus(imported.cloned ? "Library imported as copy" : "Library imported");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function revealLibraryFolder(item: Library) {
    const actionKey = `reveal:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      await invokeCommand<void>("reveal_library_folder", { rootPath: item.rootPath });
      setStatus("Library folder opened");
      forgetMissingLibraryPath(item.rootPath);
    } catch (error) {
      const message = errorMessage(error);
      setStatus(message);
      if (message.includes("missing") || message.includes("not found")) {
        rememberMissingLibraryPath(item.rootPath);
      }
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function refreshDaemonHealth() {
    if (!runningInTauri) {
      setDaemonOnline(true);
      return;
    }
    try {
      const online = await invokeCommand<boolean>("daemon_health");
      setDaemonOnline(online);
    } catch (error) {
      setDaemonOnline(false);
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshTasks(options: { showLoading?: boolean } = {}): Promise<DaemonTask[]> {
    const showLoading = options.showLoading ?? true;
    if (!library) {
      setTasks([]);
      setSelectedTaskId(null);
      return [];
    }
    if (!runningInTauri) {
      setTasks(mockTasks);
      return mockTasks;
    }
    if (showLoading) {
      setTasksLoading(true);
    }
    try {
      const nextTasks = await invokeCommand<DaemonTask[]>("list_daemon_tasks", {
        input: { libraryPath: library.rootPath },
      });
      const nextCompletedKeys = nextTasks
        .filter((task) => task.status === "completed")
        .map((task) => completedTaskKey(task));
      const hasNewCompletedTask = nextCompletedKeys.some((key) => !completedTaskKeysRef.current.has(key));
      completedTaskKeysRef.current = new Set(nextCompletedKeys);
      setTasks(nextTasks);
      setSelectedTaskId((current) => {
        if (current && nextTasks.some((task) => task.id === current)) {
          return current;
        }
        return nextTasks[0]?.id ?? null;
      });
      setDaemonOnline(true);
      setRecoverableError(null);
      if (hasNewCompletedTask) {
        void refreshGallery();
        void refreshSuggestions();
      }
      return nextTasks;
    } catch (error) {
      setDaemonOnline(false);
      setRecoverableError(errorMessage(error));
      return [];
    } finally {
      if (showLoading) {
        setTasksLoading(false);
      }
    }
  }

  async function loadTaskDetail(taskId: string) {
    if (!runningInTauri) {
      const task = mockTasks.find((item) => item.id === taskId) ?? null;
      setTaskDetail(task ? { task, attempts: [], events: [], outputs: [], logTail: "", logTailTruncated: false } : null);
      return;
    }
    try {
      const detail = await invokeCommand<DaemonTaskDetail>("get_daemon_task_detail", {
        input: { taskId },
      });
      setTaskDetail(detail);
      setRecoverableError(null);
    } catch (error) {
      setTaskDetail(null);
      setRecoverableError(errorMessage(error));
    }
  }

  function waitForMetadataPollDelay() {
    return new Promise<void>((resolve) => {
      const timer = window.setTimeout(() => {
        metadataPollTimeoutsRef.current.delete(timer);
        resolve();
      }, METADATA_POLL_INTERVAL_MS);
      metadataPollTimeoutsRef.current.add(timer);
    });
  }

  async function waitForMetadataFieldResult(
    taskId: string,
    suggestionId: string,
    field: ReviewFieldName,
    baseRevision: string,
  ) {
    for (let attempt = 0; attempt < 20; attempt += 1) {
      const detail = await invokeCommand<DaemonTaskDetail>("get_daemon_task_detail", {
        input: { taskId },
      });
      setTaskDetail(detail);
      const output = detail.outputs.find(
        (item) => item.outputType === "metadata_field_result" && item.targetId === suggestionId,
      );
      if (output?.payload?.field === field && output.payload.baseRevision === baseRevision) {
        const value = output.payload.value;
        if (typeof value === "string") {
          return value;
        }
      }
      if (isTerminalFailureStatus(detail.task.status)) {
        throw new Error(detail.task.lastErrorMessage ?? "Metadata field generation failed");
      }
      await waitForMetadataPollDelay();
    }
    throw new Error("Metadata field generation timed out");
  }

  async function waitForMetadataSuggestionResult(taskId: string) {
    for (let attempt = 0; attempt < 20; attempt += 1) {
      const detail = await invokeCommand<DaemonTaskDetail>("get_daemon_task_detail", {
        input: { taskId },
      });
      setTaskDetail(detail);
      const output = detail.outputs.find((item) => item.outputType === "metadata_suggestion");
      if (output) {
        return output.targetId;
      }
      if (isTerminalFailureStatus(detail.task.status)) {
        throw new Error(detail.task.lastErrorMessage ?? "Metadata suggestion generation failed");
      }
      await waitForMetadataPollDelay();
    }
    throw new Error("Metadata suggestion generation timed out");
  }

  async function startGeneration(inputVersionId: string | null = null, inputFile = "") {
    if (generationSubmitting) {
      return;
    }
    if (!library || prompt.trim().length === 0) {
      setRecoverableError("Open a real library and enter a prompt before generation.");
      return;
    }
    setGenerationSubmitting(true);
    try {
      const created = await enqueueTaskDrafts([
        createTaskDraft({
          provider,
          prompt,
          operation: inputVersionId || inputFile ? "image_to_image" : "text_to_image",
          inputFile,
          inputVersionId,
        }),
      ]);
      if (created.length === 0) {
        setRecoverableError((current) => current ?? "No generation task was created. Check daemon status and try again.");
        return;
      }
      setPrompt("");
      setComposerOpen(false);
      setComposerInputVersionId(null);
      setComposerInputFile("");
      setComposerInputFileName(null);
      setActiveView("queue");
      setActiveTaskPanel("queue");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setGenerationSubmitting(false);
    }
  }

  function openComposerForTextGeneration(open: boolean) {
    if (open) {
      setComposerInputVersionId(null);
      setComposerInputFile("");
      setComposerInputFileName(null);
      setRecoverableError(null);
    }
    setComposerOpen(open);
  }

  function openComposerForVersionGeneration(versionId: string | null) {
    if (!versionId) {
      setRecoverableError("Select an asset version before generating a new version.");
      return;
    }
    setComposerInputVersionId(versionId);
    setComposerInputFile("");
    setComposerInputFileName(null);
    setRecoverableError(null);
    setComposerOpen(true);
  }

  function openComposerForReferenceGeneration(reference: ReferenceSource) {
    setComposerInputVersionId(null);
    setComposerInputFile(reference.filePath);
    setComposerInputFileName(reference.assetTitle ?? reference.versionName);
    setRecoverableError(null);
    setComposerOpen(true);
  }

  async function enqueueTaskDrafts(drafts: TaskDraft[] = taskDrafts): Promise<DaemonTask[]> {
    if (!library) {
      setRecoverableError("Open a real library before enqueueing tasks.");
      return [];
    }
    const readyDrafts = drafts.filter((draft) => draft.prompt.trim().length > 0);
    if (readyDrafts.length === 0) {
      setRecoverableError("Add at least one task prompt before enqueueing.");
      return [];
    }
    setStatus(`Enqueueing ${readyDrafts.length} task${readyDrafts.length === 1 ? "" : "s"}`);
    try {
      const created = await invokeCommand<DaemonTask[]>("enqueue_generation_tasks", {
        input: {
          libraryPath: library.rootPath,
          tasks: readyDrafts.map((draft) => ({
            provider: draft.provider,
            prompt: draft.prompt,
            negativePrompt: draft.negativePrompt.trim() || null,
            operation: draft.operation,
            inputFile: draft.inputFile.trim() || null,
            inputVersionId: draft.inputVersionId,
            parametersJson: draft.parametersJson,
            priority: draft.priority,
            maxAttempts: draft.maxAttempts,
          })),
        },
      });
      setTasks((current) => mergeTasks(created, current));
      setSelectedTaskId(created[0]?.id ?? selectedTaskId);
      setTaskDrafts((current) => {
        const createdIds = new Set(readyDrafts.map((draft) => draft.id));
        const remaining = current.filter((draft) => !createdIds.has(draft.id));
        return remaining.length > 0 ? remaining : [createTaskDraft()];
      });
      setStatus(`${created.length} task${created.length === 1 ? "" : "s"} enqueued`);
      setRecoverableError(null);
      void refreshTasks();
      return created;
    } catch (error) {
      setRecoverableError(errorMessage(error));
      return [];
    }
  }

  async function reorderQueuedTask(taskId: string, direction: -1 | 1) {
    if (!library) {
      return;
    }
    const queued = tasks.filter((task) => task.status === "queued").sort(compareTaskOrder);
    const nextQueuedIds = moveQueuedTaskOrder(queued, taskId, direction);
    if (nextQueuedIds.join("\0") === queued.map((task) => task.id).join("\0")) {
      return;
    }
    setTasks((current) => reorderByIds(current, nextQueuedIds));
    try {
      await invokeCommand<void>("reorder_daemon_tasks", {
        input: {
          libraryPath: library.rootPath,
          taskIds: nextQueuedIds,
        },
      });
      await refreshTasks();
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshTasks();
    }
  }

  async function runTaskAction(command: "cancel_daemon_task" | "retry_daemon_task" | "duplicate_daemon_task", taskId: string) {
    const actionKey = taskActionKey(command, taskId);
    if (pendingTaskActions.includes(actionKey)) {
      return;
    }
    setPendingTaskActions((current) => (current.includes(actionKey) ? current : [...current, actionKey]));
    try {
      const task = await invokeCommand<DaemonTask>(command, { input: { taskId } });
      setTasks((current) => mergeTasks([task], current));
      setSelectedTaskId(task.id);
      await refreshTasks();
      await loadTaskDetail(task.id);
      if (task.status === "completed") {
        await Promise.all([refreshGallery(), refreshSuggestions()]);
      }
    } catch (error) {
      const nextTasks = await refreshTasks();
      if (selectedTaskId) {
        await loadTaskDetail(selectedTaskId);
      }
      const latestTask = nextTasks.find((task) => task.id === taskId);
      if (command === "retry_daemon_task" && latestTask && !isRetryableTaskStatus(latestTask.status)) {
        setRecoverableError(null);
        return;
      }
      setRecoverableError(errorMessage(error));
    } finally {
      setPendingTaskActions((current) => current.filter((key) => key !== actionKey));
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
      const created: AlbumListItem = {
        id: `album-${crypto.randomUUID()}`,
        name,
        kind: "manual",
        itemCount: 0,
        sortOrder: albums.length + 1,
      };
      setAlbums((current) => [created, ...current]);
      setAlbumNameInput("");
      setAlbumSearchInput("");
      setAlbumCreateOpen(false);
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
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
      setQuery((current) => openAlbumQuery(current, created.id));
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
      const created: AlbumListItem = {
        id: `album-${crypto.randomUUID()}`,
        name: trimmed,
        kind: "smart",
        itemCount: 0,
        sortOrder: albums.length + 1,
      };
      setAlbums((current) => [created, ...current]);
      setSelectedAlbumId(created.id);
      setQuery((current) => openAlbumQuery(current, created.id));
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
      setQuery((current) => openAlbumQuery(current, created.id));
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  function openAlbum(albumId: string) {
    setSelectedAlbumId(albumId);
    setQuery((current) => updateGalleryQuery(openAlbumQuery(current, albumId), { sort: "albumOrder" }));
    setActiveView("albums");
  }

  function closeAlbum() {
    setSelectedAlbumId(null);
    setQuery((current) => clearAlbumQuery(current));
  }

  async function renameAlbum(albumId: string, name: string) {
    const trimmed = name.trim();
    if (!library || trimmed.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) => current.map((album) => (album.id === albumId ? { ...album, name: trimmed } : album)));
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
      setAlbums((current) => current.filter((album) => album.id !== albumId));
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
      setGallery((current) => current.filter((asset) => asset.id !== assetId));
      return;
    }
    try {
      await invokeCommand("remove_asset_from_album", {
        input: {
          albumId: selectedAlbumId,
          assetId,
        },
      });
      await Promise.all([refreshAlbums(), refreshGallery()]);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function reorderSelectedAlbumAssets(assetIds: string[]) {
    if (!library || !selectedAlbumId) {
      return;
    }
    setGallery((current) => reorderByIds(current, assetIds));
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
      await refreshGallery();
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
      await refreshGallery();
    }
  }

  async function addSelectedGalleryAssetsToAlbum(albumId: string) {
    if (!library || selectedGalleryAssetIds.length === 0) {
      return;
    }
    if (!runningInTauri) {
      setAlbums((current) =>
        current.map((album) =>
          album.id === albumId
            ? { ...album, itemCount: (album.itemCount ?? 0) + selectedGalleryAssetIds.length }
            : album,
        ),
      );
      return;
    }
    try {
      await invokeCommand("batch_add_assets_to_album", {
        input: {
          albumId,
          assetIds: selectedGalleryAssetIds,
        },
      });
      await Promise.all([refreshAlbums(), refreshGallery()]);
      setRecoverableError(null);
      setStatus("Selected assets added to album");
    } catch (error) {
      setRecoverableError(errorMessage(error));
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
        setAlbums((current) =>
          current.map((item) =>
            item.id === albumId ? { ...item, itemCount: (item.itemCount ?? 0) + 1 } : item,
          ),
        );
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
        loadAssetDetail(detail.id, selectedAsset?.currentVersionId ?? null),
      ]);
      setStatus("Asset added to album");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

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
      setSuggestions((current) => removeSuggestionState(current, suggestion.id));
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
    if (!reviewForm || !selectedSuggestion) {
      return;
    }
    if (isReviewFieldGenerating(reviewForm, field)) {
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
        current
          ? completeReviewFieldGeneration(current, suggestionId, field, requestId, value, null)
          : current,
      );
      return;
    }
    if (!library) {
      setReviewForm((current) =>
        current
          ? failReviewFieldGeneration(
              current,
              suggestionId,
              field,
              requestId,
              "Open a real library before regenerating review metadata.",
            )
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
        current
          ? completeReviewFieldGeneration(
              current,
              suggestionId,
              field,
              requestId,
              result,
              null,
            )
          : current,
      );
      setRecoverableError(null);
      void refreshTasks();
    } catch (error) {
      const message = errorMessage(error);
      setReviewForm((current) =>
        current
          ? failReviewFieldGeneration(current, suggestionId, field, requestId, message, null)
          : current,
      );
      setRecoverableError(message);
    }
  }

  async function regenerateFullSuggestion() {
    if (!selectedSuggestion || !reviewForm) {
      return;
    }
    if (suggestionRegenerating) {
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
      activeView={activeView}
      reviewCount={pendingSuggestions.length}
      queueCount={queueCount}
      expanded={sidebarExpanded}
      onExpandedChange={setSidebarExpanded}
      onViewChange={changeView}
      onLibraryChange={switchLibrary}
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
            gallery={displayedGallery}
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
            selectedGalleryAssetCount={selectedGalleryAssetIds.length}
            onBatchAddSelected={(albumId) => void addSelectedGalleryAssetsToAlbum(albumId)}
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
            versionId = versionId ?? detail?.currentVersionId ?? detail?.lineage[0]?.version.id ?? detail?.versions[0]?.id ?? selectedAsset?.currentVersionId ?? null;
            openComposerForVersionGeneration(versionId);
          }}
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
