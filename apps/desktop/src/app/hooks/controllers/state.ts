import { useState, type Dispatch, type MutableRefObject, type SetStateAction } from "react";
import {
  defaultGalleryQuery,
  type DetailLoadState,
  type GalleryQueryState,
} from "../../workflows/gallery";
import {
  createReviewFormState,
  type ReviewFormState,
} from "../../workflows/review";
import {
  defaultSettingsSection,
  type SettingsSection,
} from "../../workflows/settings";
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
import {
  initialUpdateState,
} from "../../types";
import type {
  AlbumListItem,
  AppLog,
  AppLogContent,
  AssetDetail,
  DaemonTask,
  DaemonTaskDetail,
  GalleryAsset,
  Library,
  LibraryStatus,
  LightboxImage,
  ProviderHealth,
  Suggestion,
  TaskDraft,
  TaskPanel,
  UpdateState,
} from "../../types";

export function useLibrarySettingsControllerState(runningInTauri: boolean) {
  const [libraries, setLibraries] = useState<Library[]>(runningInTauri ? [] : mockLibraries);
  const [library, setLibrary] = useState<Library | null>(runningInTauri ? null : mockLibrary);
  const [libraryStatus, setLibraryStatus] = useState<LibraryStatus | null>(
    runningInTauri ? null : mockLibraryStatus,
  );
  const [providerHealth, setProviderHealth] = useState<ProviderHealth[]>(mockProviderHealth);
  const [libraryFolderNameInput, setLibraryFolderNameInput] = useState("image-prompt-lab");
  const [libraryNameInput, setLibraryNameInput] = useState("Image Prompt Lab");
  const [settingsSection, setSettingsSection] = useState<SettingsSection>(defaultSettingsSection);
  const [pendingLibraryActions, setPendingLibraryActions] = useState<string[]>([]);
  const [missingLibraryPaths, setMissingLibraryPaths] = useState<string[]>([]);

  return {
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
  };
}

export function useGallerySelectionControllerState(runningInTauri: boolean) {
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
  const [lightboxImage, setLightboxImage] = useState<LightboxImage | null>(null);
  const [sidebarExpanded, setSidebarExpanded] = useState(false);
  const [inspectorOpen, setInspectorOpen] = useState(false);

  return {
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
  };
}

export function useAlbumControllerState(runningInTauri: boolean) {
  const [albums, setAlbums] = useState<AlbumListItem[]>(runningInTauri ? [] : mockAlbumList);
  const [selectedAlbumId, setSelectedAlbumId] = useState<string | null>(null);
  const [albumSearchInput, setAlbumSearchInput] = useState("");
  const [albumNameInput, setAlbumNameInput] = useState("");
  const [albumCreateOpen, setAlbumCreateOpen] = useState(false);
  const [albumLoading, setAlbumLoading] = useState(false);

  return {
    albums,
    setAlbums,
    selectedAlbumId,
    setSelectedAlbumId,
    albumSearchInput,
    setAlbumSearchInput,
    albumNameInput,
    setAlbumNameInput,
    albumCreateOpen,
    setAlbumCreateOpen,
    albumLoading,
    setAlbumLoading,
  };
}

export function useReviewControllerState(runningInTauri: boolean) {
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

  return {
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
  };
}

export function useTaskGenerationControllerState(runningInTauri: boolean) {
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
  const [activeTaskPanel, setActiveTaskPanel] = useState<TaskPanel>("queue");

  return {
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
  };
}

export function useAppOperationsControllerState(runningInTauri: boolean) {
  const [status, setStatus] = useState(runningInTauri ? "Open or create a library" : "Preview mode");
  const [recoverableError, setRecoverableError] = useState<string | null>(null);
  const [appLogs, setAppLogs] = useState<AppLog[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);
  const [selectedLogPath, setSelectedLogPath] = useState<string | null>(null);
  const [selectedLogContent, setSelectedLogContent] = useState<AppLogContent | null>(null);
  const [logContentLoading, setLogContentLoading] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState>(initialUpdateState);

  return {
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
  };
}
