import React, { useEffect, useState } from "react";
import { Icon } from "../../../studio-icons";
import {
  shortIdentifier,
} from "../../../studio-orchestration";
import { convertImagePath, errorMessage, pickImageFile } from "../../tauri-adapter";
import { Thumbnail } from "../gallery/GalleryWorkspace";
import { StarRatingControl, StarRatingDisplay } from "../../components/rating";
import {
  descriptionFromPrompt,
  previewGeneratedReviewField,
  schemaPromptFromAsset,
  thumbnailAspectRatio,
  titleFromPrompt,
} from "../../utils";
import { createTaskDraft } from "../../mock-data";
import { displayDate } from "./common";
import type {
  Album,
  AlbumListItem,
  AppLog,
  AppLogContent,
  AssetDetail,
  ConfidenceScore,
  DaemonTask,
  DaemonTaskDetail,
  FileContext,
  GalleryAsset,
  GeneratedReviewField,
  LightboxImage,
  Library,
  LibraryStatus,
  ProviderHealth,
  ReferenceSource,
  Suggestion,
  TaskDraft,
  TaskPanel,
  UpdateState,
  View,
} from "../../types";
import {
  compareTaskOrder,
  isRetryableTaskStatus,
  parseTaskDraftImport,
  statusLabel,
  taskActionKey,
  taskPrompt,
} from "../../workflows/tasks";
import type { Dictionary } from "../../i18n/dictionaries";
export function TaskWorkspace({
  drafts,
  tasks,
  selectedTaskId,
  detail,
  loading,
  daemonOnline,
  pendingTaskActions,
  activePanel,
  onActivePanelChange,
  onDraftsChange,
  onAddDraft,
  onEnqueue,
  onRefresh,
  onSelectTask,
  onMoveTask,
  onCancel,
  onRetry,
  onDuplicate,
  dictionary,
}: {
  drafts: TaskDraft[];
  tasks: DaemonTask[];
  selectedTaskId: string | null;
  detail: DaemonTaskDetail | null;
  loading: boolean;
  daemonOnline: boolean;
  pendingTaskActions: string[];
  activePanel: TaskPanel;
  onActivePanelChange: (panel: TaskPanel) => void;
  onDraftsChange: (drafts: TaskDraft[]) => void;
  onAddDraft: () => void;
  onEnqueue: () => void;
  onRefresh: () => void;
  onSelectTask: (taskId: string) => void;
  onMoveTask: (taskId: string, direction: -1 | 1) => void;
  onCancel: (taskId: string) => void;
  onRetry: (taskId: string) => void;
  onDuplicate: (taskId: string) => void;
  dictionary: Dictionary;
}) {
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? null;
  return (
    <section className={`task-workspace active-${activePanel}`}>
      <div className="task-panel-tabs" role="tablist" aria-label={dictionary.workflow.queuePanels}>
        <button className={activePanel === "compose" ? "active" : ""} onClick={() => onActivePanelChange("compose")}>
          {dictionary.workflow.compose}
        </button>
        <button className={activePanel === "queue" ? "active" : ""} onClick={() => onActivePanelChange("queue")}>
          {dictionary.workflow.queuePanel}
        </button>
        <button className={activePanel === "detail" ? "active" : ""} onClick={() => onActivePanelChange("detail")}>
          {dictionary.workflow.detail}
        </button>
      </div>
      <div className={activePanel === "compose" ? "task-panel-slot task-panel-compose active" : "task-panel-slot task-panel-compose"}>
        <BatchComposer
          drafts={drafts}
          onDraftsChange={onDraftsChange}
          onAddDraft={onAddDraft}
          onEnqueue={onEnqueue}
          dictionary={dictionary}
        />
      </div>
      <div className={activePanel === "queue" ? "task-panel-slot task-panel-queue active" : "task-panel-slot task-panel-queue"}>
        <TasksQueue
          tasks={tasks}
          selectedTaskId={selectedTaskId}
          loading={loading}
          daemonOnline={daemonOnline}
          pendingTaskActions={pendingTaskActions}
          onRefresh={onRefresh}
          onSelectTask={onSelectTask}
          onMoveTask={onMoveTask}
          onCancel={onCancel}
          onRetry={onRetry}
          onDuplicate={onDuplicate}
          dictionary={dictionary}
        />
      </div>
      <div className={activePanel === "detail" ? "task-panel-slot task-panel-detail active" : "task-panel-slot task-panel-detail"}>
        <TaskDetailPanel
          task={selectedTask}
          detail={detail}
          pendingTaskActions={pendingTaskActions}
          onCancel={onCancel}
          onRetry={onRetry}
          onDuplicate={onDuplicate}
          dictionary={dictionary}
        />
      </div>
    </section>
  );
}

function BatchComposer({
  drafts,
  onDraftsChange,
  onAddDraft,
  onEnqueue,
  dictionary,
}: {
  drafts: TaskDraft[];
  onDraftsChange: (drafts: TaskDraft[]) => void;
  onAddDraft: () => void;
  onEnqueue: () => void;
  dictionary: Dictionary;
}) {
  const [importJson, setImportJson] = useState("");
  const [importError, setImportError] = useState<string | null>(null);
  function updateDraft(id: string, patch: Partial<TaskDraft>) {
    onDraftsChange(drafts.map((draft) => (draft.id === id ? { ...draft, ...patch } : draft)));
  }
  function duplicateDraft(draft: TaskDraft) {
    onDraftsChange([...drafts, createTaskDraft({ ...draft, id: crypto.randomUUID() })]);
  }
  function removeDraft(id: string) {
    const next = drafts.filter((draft) => draft.id !== id);
    onDraftsChange(next.length > 0 ? next : [createTaskDraft()]);
  }
  async function chooseReferenceFile(draft: TaskDraft) {
    const selected = await pickImageFile(dictionary.workflow.chooseReferenceImage, draft.inputFile);
    if (selected) {
      updateDraft(draft.id, { inputFile: selected, operation: "image_to_image" });
    }
  }
  function clearReferenceFile(draft: TaskDraft) {
    updateDraft(draft.id, {
      inputFile: "",
      operation: draft.operation === "image_to_image" ? "text_to_image" : draft.operation,
    });
  }
  function importDrafts() {
    let imported: TaskDraft[];
    try {
      imported = parseTaskDraftImport(importJson).map((draft) => createTaskDraft(draft));
    } catch (error) {
      setImportError(errorMessage(error));
      return;
    }
    if (imported.length > 0) {
      onDraftsChange(imported);
      setImportJson("");
      setImportError(null);
    } else {
      setImportError(dictionary.workflow.noValidTasksFound);
    }
  }
  return (
    <section className="task-panel batch-composer">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.batchComposer}</h3>
          <p>{drafts.length} {dictionary.workflow.drafts}</p>
        </div>
        <button onClick={onAddDraft}>{dictionary.workflow.addTask}</button>
      </div>
      {drafts.map((draft, index) => (
        <article className="task-draft-card" key={draft.id}>
          <div className="task-draft-header">
            <strong>{dictionary.workflow.task} {index + 1}</strong>
            <div className="row-actions">
              <button onClick={() => duplicateDraft(draft)}>{dictionary.workflow.duplicate}</button>
              <button onClick={() => removeDraft(draft.id)}>{dictionary.workflow.remove}</button>
            </div>
          </div>
          <label>
            <span>{dictionary.workflow.prompt}</span>
            <textarea value={draft.prompt} onChange={(event) => updateDraft(draft.id, { prompt: event.target.value })} />
          </label>
          <div className="task-draft-grid">
            <label>
              <span>{dictionary.workflow.provider}</span>
              <select className="select-control" value={draft.provider} onChange={(event) => updateDraft(draft.id, { provider: event.target.value })}>
                <option value="codex-cli">codex-cli</option>
                <option value="fake">fake</option>
              </select>
            </label>
            <label>
              <span>{dictionary.workflow.operation}</span>
              <select className="select-control" value={draft.operation} onChange={(event) => updateDraft(draft.id, { operation: event.target.value as TaskDraft["operation"] })}>
                <option value="text_to_image">text to image</option>
                <option value="image_to_image">image to image</option>
              </select>
            </label>
            <label>
              <span>{dictionary.workflow.priority}</span>
              <input type="number" value={draft.priority} onChange={(event) => updateDraft(draft.id, { priority: Number(event.target.value) })} />
            </label>
            <label>
              <span>{dictionary.workflow.maxAttempts}</span>
              <input type="number" min={1} max={10} value={draft.maxAttempts} onChange={(event) => updateDraft(draft.id, { maxAttempts: Number(event.target.value) })} />
            </label>
            <label>
              <span>{dictionary.workflow.referenceFile}</span>
              <div className="reference-file-control">
                <input value={draft.inputFile} onChange={(event) => updateDraft(draft.id, { inputFile: event.target.value, operation: event.target.value.trim() ? "image_to_image" : draft.operation })} />
                <div className="reference-file-actions">
                  <button type="button" onClick={() => chooseReferenceFile(draft)}>{dictionary.workflow.chooseImage}</button>
                  {draft.inputFile.trim() && <button type="button" onClick={() => clearReferenceFile(draft)}>{dictionary.workflow.clear}</button>}
                </div>
              </div>
            </label>
          </div>
          <label>
            <span>{dictionary.workflow.parametersJson}</span>
            <textarea value={draft.parametersJson} onChange={(event) => updateDraft(draft.id, { parametersJson: event.target.value })} />
          </label>
        </article>
      ))}
      <div className="import-json-box">
        <textarea value={importJson} onChange={(event) => setImportJson(event.target.value)} placeholder='[{"prompt":"multi-line prompt","provider":"fake"}]' />
        <button disabled={importJson.trim().length === 0} onClick={importDrafts}>{dictionary.workflow.importJson}</button>
        {importError && <span className="inline-error">{importError}</span>}
      </div>
      <button className="primary-button" disabled={drafts.every((draft) => draft.prompt.trim().length === 0)} onClick={onEnqueue}>
        {dictionary.workflow.enqueueAll}
      </button>
    </section>
  );
}

function TasksQueue({
  tasks,
  selectedTaskId,
  loading,
  daemonOnline,
  pendingTaskActions,
  onRefresh,
  onSelectTask,
  onMoveTask,
  onCancel,
  onRetry,
  onDuplicate,
  dictionary,
}: {
  tasks: DaemonTask[];
  selectedTaskId: string | null;
  loading: boolean;
  daemonOnline: boolean;
  pendingTaskActions: string[];
  onRefresh: () => void;
  onSelectTask: (taskId: string) => void;
  onMoveTask: (taskId: string, direction: -1 | 1) => void;
  onCancel: (taskId: string) => void;
  onRetry: (taskId: string) => void;
  onDuplicate: (taskId: string) => void;
  dictionary: Dictionary;
}) {
  const queuedIds = [...tasks].filter((task) => task.status === "queued").sort(compareTaskOrder).map((task) => task.id);
  const orderedTasks = [...tasks].sort(compareTaskNewestFirst);
  return (
    <section className="task-panel tasks-queue-panel">
      <div className="panel-header">
        <div>
          <h3>{dictionary.views.queue.title}</h3>
          <p>{daemonOnline ? dictionary.workflow.daemonConnected : dictionary.workflow.daemonOffline}{loading ? ` · ${dictionary.workflow.refreshing}` : ""}</p>
        </div>
        <button onClick={onRefresh}>{dictionary.workflow.refresh}</button>
      </div>
      <div className="task-list">
        {tasks.length === 0 ? (
          <div className="empty-state">{dictionary.workflow.noTasksYet}</div>
        ) : (
          orderedTasks.map((task) => (
            <article key={task.id} className={task.id === selectedTaskId ? "task-row selected" : "task-row"}>
              <button className="task-row-main" onClick={() => onSelectTask(task.id)}>
                <strong>{task.provider ?? task.taskType}</strong>
                <span>{taskPrompt(task)}</span>
                <small>{task.waitReason ?? `${task.attemptCount}/${task.maxAttempts} attempts`}</small>
              </button>
              <span className={`status ${task.status}`}>{statusLabel(task.status)}</span>
              <div className="task-row-actions">
                {task.status === "queued" && (
                  <>
                    <button disabled={queuedIds.indexOf(task.id) <= 0} onClick={() => onMoveTask(task.id, -1)}>{dictionary.workflow.up}</button>
                    <button disabled={queuedIds.indexOf(task.id) === queuedIds.length - 1} onClick={() => onMoveTask(task.id, 1)}>{dictionary.workflow.down}</button>
                  </>
                )}
                {["queued", "running", "retry_waiting", "cancel_requested"].includes(task.status) && (
                  <button disabled={pendingTaskActions.includes(taskActionKey("cancel_daemon_task", task.id))} onClick={() => onCancel(task.id)}>{dictionary.workflow.cancel}</button>
                )}
                {isRetryableTaskStatus(task.status) && (
                  <button disabled={pendingTaskActions.includes(taskActionKey("retry_daemon_task", task.id))} onClick={() => onRetry(task.id)}>{dictionary.workflow.retry}</button>
                )}
                <button disabled={pendingTaskActions.includes(taskActionKey("duplicate_daemon_task", task.id))} onClick={() => onDuplicate(task.id)}>{dictionary.workflow.duplicate}</button>
              </div>
            </article>
          ))
        )}
      </div>
    </section>
  );
}

function compareTaskNewestFirst(left: DaemonTask, right: DaemonTask) {
  return (
    right.createdAt.localeCompare(left.createdAt) ||
    right.updatedAt.localeCompare(left.updatedAt) ||
    right.queuePosition - left.queuePosition
  );
}

function TaskDetailPanel({
  task,
  detail,
  pendingTaskActions,
  onCancel,
  onRetry,
  onDuplicate,
  dictionary,
}: {
  task: DaemonTask | null;
  detail: DaemonTaskDetail | null;
  pendingTaskActions: string[];
  onCancel: (taskId: string) => void;
  onRetry: (taskId: string) => void;
  onDuplicate: (taskId: string) => void;
  dictionary: Dictionary;
}) {
  if (!task) {
    return <section className="task-panel task-detail-panel empty-state">{dictionary.workflow.selectTask}</section>;
  }
  return (
    <section className="task-panel task-detail-panel">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.taskDetail}</h3>
          <p>{task.id}</p>
        </div>
        <span className={`status ${task.status}`}>{statusLabel(task.status)}</span>
      </div>
      <div className="row-actions">
        {["queued", "running", "retry_waiting", "cancel_requested"].includes(task.status) && (
          <button disabled={pendingTaskActions.includes(taskActionKey("cancel_daemon_task", task.id))} onClick={() => onCancel(task.id)}>{dictionary.workflow.cancel}</button>
        )}
        {isRetryableTaskStatus(task.status) && (
          <button disabled={pendingTaskActions.includes(taskActionKey("retry_daemon_task", task.id))} onClick={() => onRetry(task.id)}>{dictionary.workflow.retry}</button>
        )}
        <button disabled={pendingTaskActions.includes(taskActionKey("duplicate_daemon_task", task.id))} onClick={() => onDuplicate(task.id)}>{dictionary.workflow.duplicate}</button>
      </div>
      <section className="detail-section">
        <h4>{dictionary.workflow.inputSnapshot}</h4>
        <pre>{JSON.stringify(task.input ?? {}, null, 2)}</pre>
      </section>
      {task.lastErrorMessage && (
        <section className="detail-section">
          <h4>{dictionary.workflow.error}</h4>
          <p className="error-text">{task.lastErrorCode ?? dictionary.workflow.taskFailed}: {task.lastErrorMessage}</p>
        </section>
      )}
      <section className="detail-section">
        <h4>{dictionary.workflow.attempts}</h4>
        {(detail?.attempts ?? []).length === 0 ? <p>{dictionary.workflow.noAttemptsYet}</p> : detail?.attempts.map((attempt) => (
          <div className="detail-row" key={attempt.id}>
            <strong>#{attempt.attemptNumber} {attempt.status}</strong>
            <span>{attempt.errorMessage ?? attempt.logPath ?? displayDate(attempt.startedAt)}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>{dictionary.workflow.timeline}</h4>
        {(detail?.events ?? []).map((event) => (
          <div className="detail-row" key={event.id}>
            <strong>{event.eventType}</strong>
            <span>{event.message ? `${displayDate(event.createdAt)} · ${event.message}` : displayDate(event.createdAt)}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>{dictionary.workflow.outputs}</h4>
        {(detail?.outputs ?? []).length === 0 ? <p>{dictionary.workflow.noOutputsYet}</p> : detail?.outputs.map((output) => (
          <div className="detail-row" key={output.id}>
            <strong>{output.outputType}</strong>
            <span>{output.targetId}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>{dictionary.workflow.logTail}</h4>
        <pre>{detail?.logTail || dictionary.workflow.noLogContent}</pre>
      </section>
    </section>
  );
}
