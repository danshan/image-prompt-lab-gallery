import type { GalleryQueryState } from "./workbench-state";

export type TaskOrderState = {
  id: string;
  attemptCount: number;
  queuePosition: number;
  updatedAt: string;
  input: Record<string, unknown> | null;
};

export function galleryQueryInput(libraryPath: string, query: GalleryQueryState) {
  return {
    libraryPath,
    text: query.text.trim() || null,
    providers: query.providers,
    minRating: query.minRating,
    reviewStatus: query.reviewStatus,
    tags: query.tags,
    sort: query.sort,
    albumId: query.albumId,
  };
}

export function mergeTasks<TTask extends TaskOrderState>(nextTasks: TTask[], currentTasks: TTask[]) {
  const byId = new Map(currentTasks.map((task) => [task.id, task]));
  for (const task of nextTasks) {
    byId.set(task.id, task);
  }
  return Array.from(byId.values()).sort(compareTaskOrder);
}

export function completedTaskKey(task: Pick<TaskOrderState, "id" | "attemptCount" | "updatedAt">) {
  return `${task.id}:${task.attemptCount}:${task.updatedAt}`;
}

export function taskActionKey(command: "cancel_daemon_task" | "retry_daemon_task" | "duplicate_daemon_task", taskId: string) {
  return `${command}:${taskId}`;
}

export function isRetryableTaskStatus(status: string) {
  return (
    status === "failed_retryable" ||
    status === "failed_final" ||
    status === "interrupted_retryable" ||
    status === "interrupted_final"
  );
}

export function isTerminalFailureStatus(status: string) {
  return status === "failed_final" || status === "interrupted_final";
}

export function compareTaskOrder<TTask extends Pick<TaskOrderState, "queuePosition" | "updatedAt">>(
  left: TTask,
  right: TTask,
) {
  return left.queuePosition - right.queuePosition || right.updatedAt.localeCompare(left.updatedAt);
}

export function taskPrompt(task: Pick<TaskOrderState, "input">) {
  const input = task.input ?? {};
  const value = input.prompt;
  return typeof value === "string" && value.trim().length > 0 ? value : "No prompt snapshot";
}

export function statusLabel(status: string) {
  return status.replaceAll("_", " ");
}

export function formatOperation(operation: string) {
  return operation.replaceAll("_", " ");
}

export function formatVersionName(versionId: string) {
  const match = versionId.match(/(?:version[-_])?(.+?)(?:[-_](\d+))?$/);
  if (!match) {
    return versionId;
  }
  const stem = match[1]
    .split(/[-_]/)
    .filter(Boolean)
    .map((part) => part[0]?.toUpperCase() + part.slice(1))
    .join(" ");
  return match[2] ? `${stem} v${match[2]}` : stem || versionId;
}

export function shortIdentifier(value: string) {
  if (value.length <= 18) {
    return value;
  }
  return `${value.slice(0, 10)}...${value.slice(-4)}`;
}
