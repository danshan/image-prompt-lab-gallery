import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { reorderByIds } from "../../workflows/shared/state";
import type { ReviewFieldName } from "../../workflows/review";
import {
  createTaskDraft,
  mockTasks,
} from "../../mock-data";
import { METADATA_POLL_INTERVAL_MS } from "../../types";
import {
  nextAnimationFrame,
} from "../../utils";
import {
  errorMessage,
  invokeCommand,
} from "../../tauri-adapter";
import {
  compareTaskOrder,
  completedTaskKey,
  isRetryableTaskStatus,
  isTerminalFailureStatus,
  mergeTasks,
  moveQueuedTaskOrder,
  taskActionKey,
} from "../../workflows/tasks/state";
import type {
  DaemonTask,
  DaemonTaskDetail,
  Library,
  ReferenceSource,
  TaskDraft,
  TaskPanel,
  View,
} from "../../types";

export function useTaskGenerationActions({
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
}: {
  runningInTauri: boolean;
  library: Library | null;
  prompt: string;
  provider: string;
  taskDrafts: TaskDraft[];
  tasks: DaemonTask[];
  selectedTaskId: string | null;
  generationSubmitting: boolean;
  pendingTaskActions: string[];
  completedTaskKeysRef: MutableRefObject<Set<string>>;
  metadataPollTimeoutsRef: MutableRefObject<Set<number>>;
  setDaemonOnline: Dispatch<SetStateAction<boolean>>;
  setTasks: Dispatch<SetStateAction<DaemonTask[]>>;
  setSelectedTaskId: Dispatch<SetStateAction<string | null>>;
  setTaskDetail: Dispatch<SetStateAction<DaemonTaskDetail | null>>;
  setTasksLoading: Dispatch<SetStateAction<boolean>>;
  setPendingTaskActions: Dispatch<SetStateAction<string[]>>;
  setTaskDrafts: Dispatch<SetStateAction<TaskDraft[]>>;
  setPrompt: Dispatch<SetStateAction<string>>;
  setComposerOpen: Dispatch<SetStateAction<boolean>>;
  setComposerInputVersionId: Dispatch<SetStateAction<string | null>>;
  setComposerInputFile: Dispatch<SetStateAction<string>>;
  setComposerInputFileName: Dispatch<SetStateAction<string | null>>;
  setGenerationSubmitting: Dispatch<SetStateAction<boolean>>;
  setActiveView: Dispatch<SetStateAction<View>>;
  setActiveTaskPanel: Dispatch<SetStateAction<TaskPanel>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
  refreshGallery: () => Promise<void>;
  refreshSuggestions: () => Promise<void>;
}) {
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

  return {
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
  };
}
