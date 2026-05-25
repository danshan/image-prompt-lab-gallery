import type { ScheduleRule } from "../../types.js";

export type ScheduleDraft = {
  name: string;
  promptMode: "fixed" | "dynamic";
  fixedPrompt: string;
  basePrompt: string;
  dynamicPrompt: string;
  targetAlbumId: string;
  tags: string;
  imageProvider: string;
  imageModel: string;
  promptExpanderProvider: string;
  promptExpanderModel: string;
  scheduleKind: ScheduleRule["kind"];
  minutes: number;
  hours: number;
  localTimeHhMm: string;
};

export function defaultScheduleDraftForProvider(defaultProvider: string, targetAlbumId = ""): ScheduleDraft {
  const provider = defaultProvider || "codex-cli";
  return {
    name: "Scheduled image",
    promptMode: "fixed",
    fixedPrompt: "",
    basePrompt: "",
    dynamicPrompt: "",
    targetAlbumId,
    tags: "",
    imageProvider: provider,
    imageModel: provider === "fake" ? "fake" : "codex",
    promptExpanderProvider: provider === "fake" ? "fake" : "codex-cli",
    promptExpanderModel: provider === "fake" ? "fake" : "codex",
    scheduleKind: "interval_hours",
    minutes: 30,
    hours: 6,
    localTimeHhMm: "09:00",
  };
}
