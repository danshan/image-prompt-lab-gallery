export {
  compareTaskOrder,
  completedTaskKey,
  formatOperation,
  formatVersionName,
  isRetryableTaskStatus,
  isTerminalFailureStatus,
  mergeTasks,
  statusLabel,
  taskActionKey,
  taskPrompt,
  type TaskOrderState,
} from "../../../studio-orchestration.js";
import { moveItem } from "../shared/state.js";

export type TaskDraftImport = {
  prompt: string;
  provider: string;
  operation: "text_to_image" | "image_to_image";
  inputFile: string;
  parametersJson: string;
  priority: number;
  maxAttempts: number;
};

export type QueueTaskState = {
  id: string;
  status: string;
};

const activeTaskStatuses = new Set(["queued", "running", "retry_waiting", "interrupted_retryable"]);

export function countActiveTasks<TTask extends QueueTaskState>(tasks: TTask[]): number {
  return tasks.filter((task) => activeTaskStatuses.has(task.status)).length;
}

export function parseTaskDraftImport(input: string): TaskDraftImport[] {
  const parsed = JSON.parse(input) as unknown;
  const items = Array.isArray(parsed)
    ? parsed
    : typeof parsed === "object" && parsed && Array.isArray((parsed as { tasks?: unknown }).tasks)
      ? (parsed as { tasks: unknown[] }).tasks
      : [];
  return items.flatMap((item) => {
    if (!item || typeof item !== "object") {
      return [];
    }
    const value = item as Record<string, unknown>;
    if (typeof value.prompt !== "string" || value.prompt.trim().length === 0) {
      return [];
    }
    return [
      {
        prompt: value.prompt,
        provider: typeof value.provider === "string" ? value.provider : "codex-cli",
        operation: value.operation === "image_to_image" ? "image_to_image" : "text_to_image",
        inputFile: typeof value.inputFile === "string" ? value.inputFile : "",
        parametersJson:
          typeof value.parametersJson === "string"
            ? value.parametersJson
            : JSON.stringify(value.parameters ?? {}, null, 2),
        priority: typeof value.priority === "number" ? value.priority : 0,
        maxAttempts: typeof value.maxAttempts === "number" ? value.maxAttempts : 3,
      },
    ];
  });
}

export function moveQueuedTaskOrder<TTask extends QueueTaskState>(
  tasks: TTask[],
  taskId: string,
  direction: -1 | 1,
): string[] {
  const queued = tasks.filter((task) => task.status === "queued");
  const fromIndex = queued.findIndex((task) => task.id === taskId);
  const toIndex = fromIndex + direction;
  if (fromIndex < 0 || toIndex < 0 || toIndex >= queued.length) {
    return queued.map((task) => task.id);
  }
  return moveItem(queued, fromIndex, toIndex).map((task) => task.id);
}
