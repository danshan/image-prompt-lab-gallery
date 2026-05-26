import React, { useEffect, useMemo, useRef, useState } from "react";
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
import {
  responsiveModeForWidth,
  sidebarCollapsedByDefaultForMode,
} from "./shell/state.js";
import { AppShell } from "../studio-shell";
import { useThemePreference } from "./design-system/theme.js";
import { useLocalePreference } from "./i18n/use-locale.js";
import { CommandBar, ContextDrawer, WorkspaceSidebar } from "./shell/desktop-shell.js";
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
  SchedulesWorkspace,
  type ScheduleDraft,
  TaskWorkspace,
  WorkspaceToolbar,
  PromptWorkspace,
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
  settingsSections,
  useAppOperationsActions,
  useAppOperationsControllerState,
  useLibrarySettingsActions,
  useLibrarySettingsControllerState,
  type SettingsSection,
} from "./workflows/settings";
import {
  usePromptWorkspaceActions,
  usePromptWorkspaceControllerState,
} from "./workflows/prompts";
import { initialUpdateState } from "./types";
import type {
  Album,
  AlbumListItem,
  AppLog,
  AppLogContent,
  ArchivedContent,
  AssetDetail,
  AssetView,
  AutomationDaemonStatus,
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
  MergeLibrarySummary,
  PermanentDeleteSummary,
  PromptVersion,
  PromoteAssetVersionResult,
  ProviderHealth,
  ReferenceSource,
  ScheduledGenerationJob,
  ScheduledGenerationRun,
  Suggestion,
  TaskDraft,
  TaskQueueSettings,
  TaskPanel,
  UpdateCheck,
  UpdateState,
  View,
  ConfidenceScore,
} from "./types";

const views: View[] = ["gallery", "albums", "prompts", "schedules", "review", "queue", "settings"];

function initialViewFromUrl(): View {
  if (typeof window === "undefined") {
    return "gallery";
  }
  const view = new URLSearchParams(window.location.search).get("view");
  return views.includes(view as View) ? (view as View) : "gallery";
}

function initialDrawerOpenFromUrl(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return new URLSearchParams(window.location.search).get("drawer") === "1";
}

function initialSettingsSectionFromUrl(): SettingsSection | null {
  if (typeof window === "undefined") {
    return null;
  }
  const section = new URLSearchParams(window.location.search).get("settings");
  return settingsSections.includes(section as SettingsSection) ? (section as SettingsSection) : null;
}

function scheduleMutationInputFromDraft(draft: ScheduleDraft, libraryPath: string) {
  const tags = draft.tags.split(",").map((tag) => tag.trim()).filter(Boolean);
  const scheduleRule = draft.scheduleKind === "interval_minutes"
    ? { kind: "interval_minutes", minutes: draft.minutes, hours: null, timezoneId: null, localTimeHhMm: null }
    : draft.scheduleKind === "interval_hours"
      ? { kind: "interval_hours", minutes: null, hours: draft.hours, timezoneId: null, localTimeHhMm: null }
      : { kind: "daily_time", minutes: null, hours: null, timezoneId: "UTC", localTimeHhMm: draft.localTimeHhMm };
  return {
    libraryPath,
    name: draft.name,
    promptMode: draft.promptMode,
    fixedPrompt: draft.promptMode === "fixed" ? draft.fixedPrompt : null,
    negativePrompt: null,
    basePrompt: draft.promptMode === "dynamic" ? draft.basePrompt : null,
    dynamicPrompt: draft.promptMode === "dynamic" ? draft.dynamicPrompt : null,
    parameters: {},
    scheduleRule,
    targetAlbumId: draft.targetAlbumId,
    tags,
    imageProvider: draft.imageProvider,
    imageModel: draft.imageModel || (draft.imageProvider === "fake" ? "fake" : "codex"),
    promptExpanderProvider: draft.promptMode === "dynamic" ? draft.promptExpanderProvider : null,
    promptExpanderModel: draft.promptMode === "dynamic" ? draft.promptExpanderModel : null,
    nextRunAt: String(Date.now()),
  };
}

function scheduleDraftFromJob(job: ScheduledGenerationJob): ScheduleDraft {
  return {
    name: job.name,
    promptMode: job.promptMode,
    fixedPrompt: job.fixedPrompt ?? "",
    basePrompt: job.basePrompt ?? "",
    dynamicPrompt: job.dynamicPrompt ?? "",
    targetAlbumId: job.targetAlbumId,
    tags: job.tags.join(", "),
    imageProvider: job.imageProvider,
    imageModel: job.imageModel,
    promptExpanderProvider: job.promptExpanderProvider ?? "codex-cli",
    promptExpanderModel: job.promptExpanderModel ?? "codex",
    scheduleKind: job.scheduleRule.kind,
    minutes: job.scheduleRule.minutes ?? 30,
    hours: job.scheduleRule.hours ?? 6,
    localTimeHhMm: job.scheduleRule.localTimeHhMm ?? "09:00",
  };
}

export function StudioAppController() {
  const runningInTauri = hasTauriRuntime();
  const [activeView, setActiveView] = useState<View>(initialViewFromUrl);
  const [sidebarManuallySet, setSidebarManuallySet] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    if (typeof window === "undefined") {
      return false;
    }
    return sidebarCollapsedByDefaultForMode(responsiveModeForWidth(window.innerWidth));
  });
  const { theme, toggleTheme } = useThemePreference();
  const { locale, dictionary, toggleLocale } = useLocalePreference();
  useEffect(() => {
    if (typeof window === "undefined" || sidebarManuallySet) {
      return;
    }
    const syncSidebarForViewport = () => {
      setSidebarCollapsed(sidebarCollapsedByDefaultForMode(responsiveModeForWidth(window.innerWidth)));
    };
    syncSidebarForViewport();
    window.addEventListener("resize", syncSidebarForViewport);
    return () => window.removeEventListener("resize", syncSidebarForViewport);
  }, [sidebarManuallySet]);
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
  const initialSettingsSection = useMemo(initialSettingsSectionFromUrl, []);
  useEffect(() => {
    if (initialSettingsSection) {
      setSettingsSection(initialSettingsSection);
    }
  }, [initialSettingsSection, setSettingsSection]);
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
    inspectorOpen,
    setInspectorOpen,
  } = useGallerySelectionControllerState(runningInTauri);
  const initialDrawerOpen = useMemo(initialDrawerOpenFromUrl, []);
  useEffect(() => {
    if (initialDrawerOpen) {
      setInspectorOpen(true);
    }
  }, [initialDrawerOpen, setInspectorOpen]);
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
  const [scheduledJobs, setScheduledJobs] = useState<ScheduledGenerationJob[]>([]);
  const [scheduledRuns, setScheduledRuns] = useState<ScheduledGenerationRun[]>([]);
  const [selectedScheduleJobId, setSelectedScheduleJobId] = useState<string | null>(null);
  const [schedulesLoading, setSchedulesLoading] = useState(false);
  const [automationDaemonStatus, setAutomationDaemonStatus] = useState<AutomationDaemonStatus | null>(null);
  const [automationDaemonLoading, setAutomationDaemonLoading] = useState(false);
  const [taskQueueSettings, setTaskQueueSettings] = useState<TaskQueueSettings | null>(null);
  const [taskQueueSettingsInput, setTaskQueueSettingsInput] = useState("");
  const [taskQueueSettingsLoading, setTaskQueueSettingsLoading] = useState(false);
  const [taskQueueSettingsSaving, setTaskQueueSettingsSaving] = useState(false);
  const [taskQueueSettingsError, setTaskQueueSettingsError] = useState<string | null>(null);
  const promptWorkspaceState = usePromptWorkspaceControllerState(runningInTauri);
  const [archivedContent, setArchivedContent] = useState<ArchivedContent[]>([]);
  const [archivedLoading, setArchivedLoading] = useState(false);
  const [permanentDeleteSummary, setPermanentDeleteSummary] = useState<PermanentDeleteSummary | null>(null);
  const [mergeSummary, setMergeSummary] = useState<MergeLibrarySummary | null>(null);
  const [mergeSourcePath, setMergeSourcePath] = useState("");
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
    archiveAssets,
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
    refreshPrompts,
    selectPrompt,
    createPrompt,
    saveDraft: savePromptDraft,
    saveVersion: savePromptVersion,
    selectPromptVersion,
    openPromptVersion,
    renderSelectedPrompt,
    runSelectedPrompt,
    newPromptDraft,
    archivePrompt,
  } = usePromptWorkspaceActions({
    runningInTauri,
    library,
    ...promptWorkspaceState,
    setTasks,
    setSelectedTaskId,
    setActiveView,
    setActiveTaskPanel,
    setStatus,
    setRecoverableError,
    refreshTasks,
  });

  async function refreshArchivedContent(itemType: "asset" | "prompt" | null = null) {
    if (!runningInTauri || !library) {
      setArchivedContent([]);
      return;
    }
    setArchivedLoading(true);
    try {
      const items = await invokeCommand<ArchivedContent[]>("list_archived_content", {
        input: {
          libraryPath: library.rootPath,
          itemType,
        },
      });
      setArchivedContent(items);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setArchivedLoading(false);
    }
  }

  async function restoreArchivedItem(item: ArchivedContent) {
    if (!library) {
      return;
    }
    try {
      const command = item.itemType === "asset" ? "restore_asset" : "restore_prompt_document";
      await invokeCommand<void>(command, {
        input: item.itemType === "asset"
          ? { libraryPath: library.rootPath, assetId: item.id }
          : { libraryPath: library.rootPath, promptId: item.id },
      });
      await refreshArchivedContent();
      void refreshGallery();
      void refreshPrompts();
      setStatus("Archived item restored");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function dryRunPermanentDelete(item: ArchivedContent) {
    if (!library) {
      return;
    }
    try {
      const summary = await invokeCommand<PermanentDeleteSummary>("dry_run_permanent_delete_archived_content", {
        input: {
          libraryPath: library.rootPath,
          itemType: item.itemType,
          id: item.id,
        },
      });
      setPermanentDeleteSummary(summary);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function confirmPermanentDelete(summary: PermanentDeleteSummary) {
    if (!library) {
      return;
    }
    try {
      await invokeCommand<PermanentDeleteSummary>("permanent_delete_archived_content", {
        input: {
          libraryPath: library.rootPath,
          itemType: summary.itemType,
          id: summary.itemId,
        },
      });
      setPermanentDeleteSummary(null);
      await refreshArchivedContent();
      setStatus("Archived item deleted");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function chooseMergeSource() {
    const selected = await pickDirectory("Choose Source Library");
    if (selected) {
      setMergeSourcePath(selected);
    }
  }

  async function dryRunMergeLibrary() {
    if (!library || !mergeSourcePath) {
      return;
    }
    try {
      const summary = await invokeCommand<MergeLibrarySummary>("dry_run_merge_library", {
        input: {
          targetLibraryPath: library.rootPath,
          sourceLibraryPath: mergeSourcePath,
        },
      });
      setMergeSummary(summary);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function applyMergeLibrary() {
    if (!library || !mergeSourcePath) {
      return;
    }
    try {
      const summary = await invokeCommand<MergeLibrarySummary>("merge_library", {
        input: {
          targetLibraryPath: library.rootPath,
          sourceLibraryPath: mergeSourcePath,
        },
      });
      setMergeSummary(summary);
      void refreshGallery();
      void refreshPrompts();
      setStatus("Library merged");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshSchedules() {
    if (!runningInTauri || !library) {
      setScheduledJobs([]);
      setScheduledRuns([]);
      return;
    }
    setSchedulesLoading(true);
    try {
      const jobs = await invokeCommand<ScheduledGenerationJob[]>("list_scheduled_generation_jobs", {
        input: { libraryPath: library.rootPath },
      });
      setScheduledJobs(jobs);
      const selectedJobId = selectedScheduleJobId && jobs.some((job) => job.id === selectedScheduleJobId)
        ? selectedScheduleJobId
        : jobs[0]?.id ?? null;
      setSelectedScheduleJobId(selectedJobId);
      if (selectedJobId) {
        const runs = await invokeCommand<ScheduledGenerationRun[]>("list_scheduled_generation_runs", {
          input: { jobId: selectedJobId },
        });
        setScheduledRuns(runs);
      } else {
        setScheduledRuns([]);
      }
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setSchedulesLoading(false);
    }
  }

  async function selectScheduleJob(jobId: string) {
    setSelectedScheduleJobId(jobId);
    if (!runningInTauri) {
      return;
    }
    try {
      const runs = await invokeCommand<ScheduledGenerationRun[]>("list_scheduled_generation_runs", {
        input: { jobId },
      });
      setScheduledRuns((current) => [
        ...current.filter((run) => run.jobId !== jobId),
        ...runs,
      ]);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function createScheduleJob(draft: ScheduleDraft) {
    if (!library) {
      setRecoverableError("Open a real library before creating schedules.");
      return;
    }
    try {
      const job = await invokeCommand<ScheduledGenerationJob>("create_scheduled_generation_job", {
        input: scheduleMutationInputFromDraft(draft, library.rootPath),
      });
      setScheduledJobs((current) => [job, ...current.filter((item) => item.id !== job.id)]);
      await selectScheduleJob(job.id);
      setStatus("Schedule created");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function updateScheduleJob(jobId: string, draft: ScheduleDraft) {
    if (!library) {
      setRecoverableError("Open a real library before updating schedules.");
      return;
    }
    try {
      const job = await invokeCommand<ScheduledGenerationJob>("update_scheduled_generation_job", {
        input: {
          ...scheduleMutationInputFromDraft(draft, library.rootPath),
          jobId,
        },
      });
      setScheduledJobs((current) => current.map((item) => (item.id === job.id ? job : item)));
      setStatus("Schedule updated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function duplicateScheduleJob(job: ScheduledGenerationJob) {
    if (!library) {
      setRecoverableError("Open a real library before duplicating schedules.");
      return;
    }
    const draft = scheduleDraftFromJob(job);
    try {
      const duplicated = await invokeCommand<ScheduledGenerationJob>("create_scheduled_generation_job", {
        input: scheduleMutationInputFromDraft({ ...draft, name: `${draft.name} copy` }, library.rootPath),
      });
      setScheduledJobs((current) => [duplicated, ...current]);
      await selectScheduleJob(duplicated.id);
      setStatus("Schedule duplicated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function deleteScheduleJob(jobId: string) {
    try {
      await invokeCommand<void>("delete_scheduled_generation_job", {
        input: { jobId },
      });
      setScheduledJobs((current) => current.filter((job) => job.id !== jobId));
      setScheduledRuns((current) => current.filter((run) => run.jobId !== jobId));
      setSelectedScheduleJobId((current) => (current === jobId ? null : current));
      setStatus("Schedule deleted");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function runScheduleNow(jobId: string) {
    try {
      const run = await invokeCommand<ScheduledGenerationRun>("run_scheduled_generation_now", {
        input: { jobId },
      });
      setScheduledRuns((current) => [run, ...current.filter((item) => item.id !== run.id)]);
      await refreshTasks();
      setStatus("Schedule run queued");
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function toggleScheduleJob(job: ScheduledGenerationJob) {
    const command = job.status === "active" ? "disable_scheduled_generation_job" : "enable_scheduled_generation_job";
    try {
      const updated = await invokeCommand<ScheduledGenerationJob>(command, {
        input: { jobId: job.id },
      });
      setScheduledJobs((current) => current.map((item) => (item.id === updated.id ? updated : item)));
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

  async function refreshAutomationDaemonStatus() {
    if (!runningInTauri) {
      setAutomationDaemonStatus(null);
      return;
    }
    setAutomationDaemonLoading(true);
    try {
      const daemonStatus = await invokeCommand<AutomationDaemonStatus>("automation_daemon_status");
      setAutomationDaemonStatus(daemonStatus);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setAutomationDaemonLoading(false);
    }
  }

  async function runAutomationDaemonAction(
    command: "start_automation_daemon" | "stop_automation_daemon" | "restart_automation_daemon" | "repair_automation_daemon",
  ) {
    setAutomationDaemonLoading(true);
    try {
      const daemonStatus = await invokeCommand<AutomationDaemonStatus>(command);
      setAutomationDaemonStatus(daemonStatus);
      setStatus("Automation daemon updated");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setAutomationDaemonLoading(false);
    }
  }

  async function refreshTaskQueueSettings() {
    if (!runningInTauri) {
      setTaskQueueSettingsLoading(false);
      setTaskQueueSettings(null);
      setTaskQueueSettingsInput("");
      setTaskQueueSettingsError("This action requires the Tauri desktop runtime. Start with npm run tauri dev.");
      return;
    }
    setTaskQueueSettingsLoading(true);
    setTaskQueueSettingsError(null);
    try {
      const settings = await invokeCommand<TaskQueueSettings>("get_task_queue_settings");
      setTaskQueueSettings(settings);
      setTaskQueueSettingsInput(String(settings.maxParallelTasks));
      setRecoverableError(null);
    } catch (error) {
      const message = errorMessage(error);
      setTaskQueueSettingsError(message);
      setRecoverableError(message);
    } finally {
      setTaskQueueSettingsLoading(false);
    }
  }

  async function saveTaskQueueSettings() {
    const parsed = Number(taskQueueSettingsInput);
    if (!Number.isInteger(parsed)) {
      setTaskQueueSettingsError(dictionary.workflow.taskQueueInvalidInteger);
      return;
    }
    if (taskQueueSettings && (parsed < taskQueueSettings.minParallelTasks || parsed > taskQueueSettings.maxParallelTasksLimit)) {
      setTaskQueueSettingsError(
        `${dictionary.workflow.taskQueueRangePrefix} ${taskQueueSettings.minParallelTasks}-${taskQueueSettings.maxParallelTasksLimit}.`,
      );
      return;
    }
    setTaskQueueSettingsSaving(true);
    setTaskQueueSettingsError(null);
    try {
      const settings = await invokeCommand<TaskQueueSettings>("update_task_queue_settings", {
        input: { maxParallelTasks: parsed },
      });
      setTaskQueueSettings(settings);
      setTaskQueueSettingsInput(String(settings.maxParallelTasks));
      setStatus(dictionary.workflow.taskQueueSaved);
      setRecoverableError(null);
    } catch (error) {
      const message = errorMessage(error);
      setTaskQueueSettingsError(message);
      setRecoverableError(message);
    } finally {
      setTaskQueueSettingsSaving(false);
    }
  }

  async function setLibraryAutomationEnabled(targetLibrary: Library, enabled: boolean) {
    try {
      const updated = await invokeCommand<Library>("set_library_automation_enabled", {
        input: { libraryId: targetLibrary.id, enabled },
      });
      setLibraries((current) => current.map((item) => (item.id === updated.id ? updated : item)));
      if (library?.id === updated.id) {
        setLibrary(updated);
      }
      setStatus(enabled ? "Library automation enabled" : "Library automation disabled");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    }
  }

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

  useEffect(() => {
    if (activeView === "prompts") {
      void refreshPrompts();
    }
  }, [activeView, library?.rootPath, promptWorkspaceState.promptSearch]);

  useEffect(() => {
    if (activeView === "schedules") {
      void refreshSchedules();
    }
  }, [activeView, library?.rootPath]);

  useEffect(() => {
    if (activeView === "settings" && settingsSection === "automation") {
      void refreshAutomationDaemonStatus();
    }
    if (activeView === "settings" && settingsSection === "taskQueue") {
      void refreshTaskQueueSettings();
    }
  }, [activeView, settingsSection]);

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
    promptWorkspaceState.setPrompts([]);
    promptWorkspaceState.setSelectedPromptId(null);
    promptWorkspaceState.setPromptVersions([]);
    promptWorkspaceState.setSelectedPromptVersionId(null);
    promptWorkspaceState.setPromptOutputHistory([]);
    promptWorkspaceState.setPromptRenderResult(null);
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
      setRecoverableError(dictionary.workflow.selectAssetVersionBeforePromote);
      return;
    }
    if (!runningInTauri) {
      setRecoverableError(dictionary.workflow.promoteRequiresRealLibrary);
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

  async function openPromptVersionFromInspector(promptId: string, versionId: string) {
    changeView("prompts");
    await openPromptVersion(promptId, versionId);
  }

  async function savePromptSnapshotFromInspector(detail: AssetDetail) {
    if (!library) {
      setRecoverableError(dictionary.workflow.openRealLibraryBeforePromptSnapshot);
      return;
    }
    if (!runningInTauri) {
      setRecoverableError(dictionary.workflow.saveAsPromptRequiresRealLibrary);
      return;
    }
    if (!detail.promptGenerationEventId) {
      setRecoverableError(dictionary.workflow.noGenerationEventForPromptSnapshot);
      return;
    }
    const name = detail.title?.trim() || "Saved Prompt";
    try {
      const version = await invokeCommand<PromptVersion>("save_generation_prompt_as_prompt", {
        input: {
          libraryPath: library.rootPath,
          generationEventId: detail.promptGenerationEventId,
          name,
        },
      });
      changeView("prompts");
      await openPromptVersion(version.promptId, version.id);
      setStatus("Prompt snapshot saved");
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
  const composerReferenceImages =
    composerInputSourceName || composerInputFile
      ? [
          {
            id: composerInputVersionId ?? composerInputFile,
            name: composerInputSourceName ?? composerInputFileName ?? shortIdentifier(composerInputFile),
            imagePath: composerInputFile || selectedAsset?.imagePath || null,
          },
        ]
      : [];
  async function chooseComposerReferenceImage() {
    const file = await pickImageFile(dictionary.workflow.chooseReferenceImage, composerInputFile);
    if (!file) {
      return;
    }
    setComposerInputVersionId(null);
    setComposerInputFile(file);
    setComposerInputFileName(file.split(/[\\/]/).pop() ?? file);
    setRecoverableError(null);
  }

  const commandBarSlot = (
    <CommandBar
      dictionary={dictionary}
      locale={locale}
      theme={theme}
      library={library}
      libraries={libraries}
      status={status}
      assetCount={gallery.length}
      reviewCount={pendingSuggestions.length}
      queueCount={queueCount}
      runningTaskCount={runningTaskCount}
      failedTaskCount={failedTaskCount}
      onGenerate={() => openComposerForTextGeneration(true)}
      onSwitchLibrary={switchLibrary}
      onThemeToggle={toggleTheme}
      onLocaleToggle={toggleLocale}
      onViewChange={changeView}
    />
  );
  const workspaceSidebarSlot = (
    <WorkspaceSidebar
      activeView={activeView}
      collapsed={sidebarCollapsed}
      counts={{
        gallery: gallery.length,
        albums: albums.length,
        prompts: promptWorkspaceState.prompts.length,
        schedules: scheduledJobs.length,
        review: pendingSuggestions.length,
        queue: queueCount,
      }}
      dictionary={dictionary}
      locale={locale}
      theme={theme}
      onCollapsedChange={(collapsed) => {
        setSidebarManuallySet(true);
        setSidebarCollapsed(collapsed);
      }}
      onGenerate={() => openComposerForTextGeneration(true)}
      onLocaleToggle={toggleLocale}
      onThemeToggle={toggleTheme}
      onViewChange={changeView}
    />
  );
  const workspaceSlot = (
    <>
        <header className="workflow-header">
          <div className="workflow-title">
            <span>{dictionary.currentLibrary}: {library?.name ?? dictionary.noLibrary}</span>
            <h1>{dictionary.views[activeView].title}</h1>
            <p>{dictionary.views[activeView].description}</p>
          </div>
          <button className="context-open-button" onClick={() => setInspectorOpen(true)}>
            {dictionary.openContext}
          </button>
        </header>
        <WorkspaceToolbar
          activeView={activeView}
          query={query}
          itemCount={displayedGallery.length}
          status={status}
          availableProviders={availableProviders}
          albums={albums}
          dictionary={dictionary}
          onQueryChange={setQuery}
        />

        {recoverableError && (
          <div className="inline-error">
            <span>{recoverableError}</span>
            <button onClick={() => setRecoverableError(null)}>{dictionary.dismiss}</button>
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
            onClearSelection={() => setSelectedGalleryAssetIds([])}
            onArchiveAsset={(assetId) => {
              void archiveAssets([assetId]);
              setSelectedGalleryAssetIds((current) => current.filter((id) => id !== assetId));
            }}
            onArchiveSelected={(assetIds) => {
              void archiveAssets(assetIds);
              setSelectedGalleryAssetIds([]);
            }}
            onQueryChange={setQuery}
            onRequestReview={(asset) => void requestAssetReview(asset)}
            onPreviewImage={setLightboxImage}
            dictionary={dictionary}
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
            onPreviewImage={setLightboxImage}
            dictionary={dictionary}
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
            dictionary={dictionary}
          />
        )}
        {activeView === "prompts" && (
          <PromptWorkspace
            prompts={promptWorkspaceState.prompts}
            search={promptWorkspaceState.promptSearch}
            selectedPromptId={promptWorkspaceState.selectedPromptId}
            draft={promptWorkspaceState.promptDraftForm}
            versions={promptWorkspaceState.promptVersions}
            selectedVersionId={promptWorkspaceState.selectedPromptVersionId}
            history={promptWorkspaceState.promptOutputHistory}
            runForm={promptWorkspaceState.promptRunForm}
            renderResult={promptWorkspaceState.promptRenderResult}
            loading={promptWorkspaceState.promptsLoading}
            versionsLoading={promptWorkspaceState.promptVersionsLoading}
            saving={promptWorkspaceState.promptSaving}
            running={promptWorkspaceState.promptRunning}
            onSearchChange={promptWorkspaceState.setPromptSearch}
            onRefresh={() => void refreshPrompts()}
            onSelectPrompt={(promptId) => void selectPrompt(promptId)}
            onDraftChange={promptWorkspaceState.setPromptDraftForm}
            onNewPrompt={newPromptDraft}
            onSaveDraft={() => void savePromptDraft()}
            onSaveVersion={() => void savePromptVersion()}
            onArchivePrompt={(promptId) => void archivePrompt(promptId)}
            onSelectVersion={(versionId) => void selectPromptVersion(versionId)}
            onRunFormChange={promptWorkspaceState.setPromptRunForm}
            onRender={() => void renderSelectedPrompt()}
            onRun={() => void runSelectedPrompt()}
            dictionary={dictionary}
          />
        )}
        {activeView === "schedules" && (
          <SchedulesWorkspace
            library={library}
            albums={albums}
            jobs={scheduledJobs}
            runs={scheduledRuns}
            selectedJobId={selectedScheduleJobId}
            defaultProvider={provider}
            loading={schedulesLoading}
            onSelectJob={(jobId) => void selectScheduleJob(jobId)}
            onRefresh={() => void refreshSchedules()}
            onCreateJob={(draft) => void createScheduleJob(draft)}
            onUpdateJob={(jobId, draft) => void updateScheduleJob(jobId, draft)}
            onDuplicateJob={(job) => void duplicateScheduleJob(job)}
            onDeleteJob={(jobId) => void deleteScheduleJob(jobId)}
            onRunNow={(jobId) => void runScheduleNow(jobId)}
            onToggleJob={(job) => void toggleScheduleJob(job)}
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
              dictionary={dictionary}
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
            archivedContent={archivedContent}
            archivedLoading={archivedLoading}
            permanentDeleteSummary={permanentDeleteSummary}
            mergeSourcePath={mergeSourcePath}
            mergeSummary={mergeSummary}
            onRefreshArchived={() => void refreshArchivedContent()}
            onRestoreArchived={(item) => void restoreArchivedItem(item)}
            onDryRunPermanentDelete={(item) => void dryRunPermanentDelete(item)}
            onConfirmPermanentDelete={(summary) => void confirmPermanentDelete(summary)}
            onChooseMergeSource={() => void chooseMergeSource()}
            onMergeSourcePathChange={setMergeSourcePath}
            onDryRunMerge={() => void dryRunMergeLibrary()}
            onApplyMerge={() => void applyMergeLibrary()}
            logs={appLogs}
            logsLoading={logsLoading}
            selectedLogPath={selectedLogPath}
            selectedLogContent={selectedLogContent}
            logContentLoading={logContentLoading}
            updateState={updateState}
            automationDaemonStatus={automationDaemonStatus}
            automationDaemonLoading={automationDaemonLoading}
            taskQueueSettings={taskQueueSettings}
            taskQueueSettingsInput={taskQueueSettingsInput}
            taskQueueSettingsLoading={taskQueueSettingsLoading}
            taskQueueSettingsSaving={taskQueueSettingsSaving}
            taskQueueSettingsError={taskQueueSettingsError}
            onTaskQueueSettingsInputChange={setTaskQueueSettingsInput}
            onRefreshTaskQueueSettings={() => void refreshTaskQueueSettings()}
            onSaveTaskQueueSettings={() => void saveTaskQueueSettings()}
            onRefreshLogs={() => void refreshAppLogs()}
            onSelectLog={(path) => void readAppLog(path)}
            onCheckUpdate={() => void checkForAppUpdate()}
            onInstallUpdate={() => void installAppUpdate()}
            onRestartApp={() => void restartApp()}
            onRefreshAutomationDaemon={() => void refreshAutomationDaemonStatus()}
            onStartAutomationDaemon={() => void runAutomationDaemonAction("start_automation_daemon")}
            onStopAutomationDaemon={() => void runAutomationDaemonAction("stop_automation_daemon")}
            onRestartAutomationDaemon={() => void runAutomationDaemonAction("restart_automation_daemon")}
            onRepairAutomationDaemon={() => void runAutomationDaemonAction("repair_automation_daemon")}
            onSetLibraryAutomationEnabled={(item, enabled) => void setLibraryAutomationEnabled(item, enabled)}
            dictionary={dictionary}
          />
        )}
      </>
  );
  const inspectorSlot = (
      <ContextDrawer
        open={inspectorOpen}
        title={selectedAsset?.title ?? dictionary.openContext}
        subtitle={selectedAsset?.currentVersionName ?? selectedAsset?.versionLabel ?? dictionary.views[activeView].title}
        closeLabel={dictionary.closeContext}
        onClose={() => setInspectorOpen(false)}
      >
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
          onOpenPromptVersion={(promptId, versionId) => void openPromptVersionFromInspector(promptId, versionId)}
          onSavePromptSnapshot={(assetDetail) => void savePromptSnapshotFromInspector(assetDetail)}
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
          dictionary={dictionary}
        />
      </ContextDrawer>
  );

  return (
    <AppShell
      commandBar={commandBarSlot}
      workspaceSidebar={workspaceSidebarSlot}
      workspace={workspaceSlot}
      contextDrawer={inspectorSlot}
      drawerOpen={inspectorOpen}
      sidebarCollapsed={sidebarCollapsed}
    >
      {composerOpen && (
        <GenerationComposer
          prompt={prompt}
          provider={provider}
          inputSourceName={composerInputSourceName}
          referenceImages={composerReferenceImages}
          submitting={generationSubmitting}
          dictionary={dictionary}
          onPromptChange={setPrompt}
          onProviderChange={setProvider}
          onChooseReferenceImage={() => void chooseComposerReferenceImage()}
          onClearReferenceImage={() => {
            setComposerInputVersionId(null);
            setComposerInputFile("");
            setComposerInputFileName(null);
          }}
          onClose={() => setComposerOpen(false)}
          onGenerate={() => void startGeneration(composerInputVersionId, composerInputFile)}
        />
      )}
      {lightboxImage && (
        <ImageLightbox
          image={lightboxImage}
          closeLabel={dictionary.workflow.closeImagePreview}
          onClose={() => setLightboxImage(null)}
        />
      )}
    </AppShell>
  );
}
