import { useState, type Dispatch, type SetStateAction } from "react";
import type { PromptDocument, PromptOutputHistoryItem, PromptVersion, RenderPromptRun } from "../../types.js";

export type PromptDraftForm = {
  name: string;
  body: string;
  negativePrompt: string;
  stylePrompt: string;
  variablesSchemaJson: string;
  defaultValuesJson: string;
  parameterPresetJson: string;
  notes: string;
};

export type PromptRunForm = {
  valuesJson: string;
  provider: string;
  model: string;
  operation: "text_to_image" | "image_to_image";
  parametersJson: string;
};

export type PromptWorkspaceControllerState = {
  prompts: PromptDocument[];
  setPrompts: Dispatch<SetStateAction<PromptDocument[]>>;
  promptSearch: string;
  setPromptSearch: Dispatch<SetStateAction<string>>;
  selectedPromptId: string | null;
  setSelectedPromptId: Dispatch<SetStateAction<string | null>>;
  promptDraftForm: PromptDraftForm;
  setPromptDraftForm: Dispatch<SetStateAction<PromptDraftForm>>;
  promptVersions: PromptVersion[];
  setPromptVersions: Dispatch<SetStateAction<PromptVersion[]>>;
  selectedPromptVersionId: string | null;
  setSelectedPromptVersionId: Dispatch<SetStateAction<string | null>>;
  promptOutputHistory: PromptOutputHistoryItem[];
  setPromptOutputHistory: Dispatch<SetStateAction<PromptOutputHistoryItem[]>>;
  promptRunForm: PromptRunForm;
  setPromptRunForm: Dispatch<SetStateAction<PromptRunForm>>;
  promptRenderResult: RenderPromptRun | null;
  setPromptRenderResult: Dispatch<SetStateAction<RenderPromptRun | null>>;
  promptsLoading: boolean;
  setPromptsLoading: Dispatch<SetStateAction<boolean>>;
  promptVersionsLoading: boolean;
  setPromptVersionsLoading: Dispatch<SetStateAction<boolean>>;
  promptSaving: boolean;
  setPromptSaving: Dispatch<SetStateAction<boolean>>;
  promptRunning: boolean;
  setPromptRunning: Dispatch<SetStateAction<boolean>>;
};

export const emptyPromptDraftForm: PromptDraftForm = {
  name: "",
  body: "",
  negativePrompt: "",
  stylePrompt: "",
  variablesSchemaJson: "[]",
  defaultValuesJson: "{}",
  parameterPresetJson: JSON.stringify({ provider: "codex-cli", operation: "text_to_image" }, null, 2),
  notes: "",
};

export const emptyPromptRunForm: PromptRunForm = {
  valuesJson: "{}",
  provider: "codex-cli",
  model: "",
  operation: "text_to_image",
  parametersJson: JSON.stringify({ provider: "codex-cli", operation: "text_to_image" }, null, 2),
};

export const mockPromptDocuments: PromptDocument[] = [
  {
    id: "prompt-botanical",
    name: "Botanical neon study",
    kind: "image_generation",
    status: "active",
    draftBody: "botanical study of {{subject}}, neon line art glow, dark background, ultra detailed",
    draftNegativePrompt: "muddy contrast, generic clipart",
    draftStylePrompt: "editorial specimen layout, controlled rim light",
    variablesSchemaJson: JSON.stringify([{ name: "subject", required: true }], null, 2),
    defaultValuesJson: JSON.stringify({ subject: "exotic plants and flowers" }, null, 2),
    parameterPresetJson: JSON.stringify({ provider: "fake", operation: "text_to_image", model: "fake-image" }, null, 2),
    notes: "Stable test prompt for line detail and glow control.",
    latestVersionId: "prompt-version-botanical-2",
    latestVersionNumber: 2,
    latestVersionName: "v2",
    createdAt: "Today, 8:40 AM",
    updatedAt: "Today, 9:15 AM",
    archivedAt: null,
  },
  {
    id: "prompt-architecture",
    name: "Solarpunk interior",
    kind: "image_generation",
    status: "active",
    draftBody: "solarpunk atrium interior, daylight, modular architecture, plants, human scale",
    draftNegativePrompt: "empty lobby, overexposed glass",
    draftStylePrompt: "quiet commercial concept art, readable material palette",
    variablesSchemaJson: "[]",
    defaultValuesJson: "{}",
    parameterPresetJson: JSON.stringify({ provider: "codex-cli", operation: "text_to_image" }, null, 2),
    notes: "Use for comparing architectural composition variants.",
    latestVersionId: "prompt-version-architecture-1",
    latestVersionNumber: 1,
    latestVersionName: "v1",
    createdAt: "Yesterday, 7:00 PM",
    updatedAt: "Yesterday, 7:45 PM",
    archivedAt: null,
  },
];

export const mockPromptVersions: PromptVersion[] = [
  {
    id: "prompt-version-botanical-2",
    promptId: "prompt-botanical",
    versionNumber: 2,
    versionName: "v2",
    body: mockPromptDocuments[0].draftBody,
    negativePrompt: mockPromptDocuments[0].draftNegativePrompt,
    stylePrompt: mockPromptDocuments[0].draftStylePrompt,
    variablesSchemaJson: mockPromptDocuments[0].variablesSchemaJson,
    defaultValuesJson: mockPromptDocuments[0].defaultValuesJson,
    parameterPresetJson: mockPromptDocuments[0].parameterPresetJson,
    notes: mockPromptDocuments[0].notes,
    createdAt: "Today, 9:10 AM",
  },
  {
    id: "prompt-version-botanical-1",
    promptId: "prompt-botanical",
    versionNumber: 1,
    versionName: "v1",
    body: "botanical study of exotic plants and flowers, neon line art glow, dark background",
    negativePrompt: "muddy contrast",
    stylePrompt: "centered specimen layout",
    variablesSchemaJson: "[]",
    defaultValuesJson: "{}",
    parameterPresetJson: JSON.stringify({ provider: "fake", operation: "text_to_image" }, null, 2),
    notes: "Original reusable version.",
    createdAt: "Today, 8:55 AM",
  },
  {
    id: "prompt-version-architecture-1",
    promptId: "prompt-architecture",
    versionNumber: 1,
    versionName: "v1",
    body: mockPromptDocuments[1].draftBody,
    negativePrompt: mockPromptDocuments[1].draftNegativePrompt,
    stylePrompt: mockPromptDocuments[1].draftStylePrompt,
    variablesSchemaJson: mockPromptDocuments[1].variablesSchemaJson,
    defaultValuesJson: mockPromptDocuments[1].defaultValuesJson,
    parameterPresetJson: mockPromptDocuments[1].parameterPresetJson,
    notes: mockPromptDocuments[1].notes,
    createdAt: "Yesterday, 7:40 PM",
  },
];

export const mockPromptOutputHistory: PromptOutputHistoryItem[] = [
  {
    generationEventId: "generation-botanical-3",
    assetId: "asset-botanical",
    outputVersionId: "version-botanical-3",
    taskId: "task-botanical-3",
    provider: "fake",
    providerModel: "fake-image",
    status: "completed",
    promptSnapshot: "botanical study of exotic plants and flowers, neon line art glow, dark background, ultra detailed",
    createdAt: "Today, 9:15 AM",
  },
];

export function usePromptWorkspaceControllerState(runningInTauri: boolean): PromptWorkspaceControllerState {
  const [prompts, setPrompts] = useState<PromptDocument[]>(runningInTauri ? [] : mockPromptDocuments);
  const [promptSearch, setPromptSearch] = useState("");
  const [selectedPromptId, setSelectedPromptId] = useState<string | null>(
    runningInTauri ? null : mockPromptDocuments[0]?.id ?? null,
  );
  const [promptDraftForm, setPromptDraftForm] = useState<PromptDraftForm>(
    createPromptDraftForm(runningInTauri ? null : mockPromptDocuments[0] ?? null),
  );
  const [promptVersions, setPromptVersions] = useState<PromptVersion[]>(
    runningInTauri ? [] : mockPromptVersions.filter((version) => version.promptId === mockPromptDocuments[0]?.id),
  );
  const [selectedPromptVersionId, setSelectedPromptVersionId] = useState<string | null>(
    runningInTauri ? null : mockPromptDocuments[0]?.latestVersionId ?? null,
  );
  const [promptOutputHistory, setPromptOutputHistory] = useState<PromptOutputHistoryItem[]>(
    runningInTauri ? [] : mockPromptOutputHistory,
  );
  const [promptRunForm, setPromptRunForm] = useState<PromptRunForm>(
    runningInTauri ? emptyPromptRunForm : createPromptRunForm(mockPromptVersions[0] ?? null),
  );
  const [promptRenderResult, setPromptRenderResult] = useState<RenderPromptRun | null>(null);
  const [promptsLoading, setPromptsLoading] = useState(false);
  const [promptVersionsLoading, setPromptVersionsLoading] = useState(false);
  const [promptSaving, setPromptSaving] = useState(false);
  const [promptRunning, setPromptRunning] = useState(false);

  return {
    prompts,
    setPrompts,
    promptSearch,
    setPromptSearch,
    selectedPromptId,
    setSelectedPromptId,
    promptDraftForm,
    setPromptDraftForm,
    promptVersions,
    setPromptVersions,
    selectedPromptVersionId,
    setSelectedPromptVersionId,
    promptOutputHistory,
    setPromptOutputHistory,
    promptRunForm,
    setPromptRunForm,
    promptRenderResult,
    setPromptRenderResult,
    promptsLoading,
    setPromptsLoading,
    promptVersionsLoading,
    setPromptVersionsLoading,
    promptSaving,
    setPromptSaving,
    promptRunning,
    setPromptRunning,
  };
}

export function createPromptDraftForm(prompt: PromptDocument | null): PromptDraftForm {
  if (!prompt) {
    return emptyPromptDraftForm;
  }
  return {
    name: prompt.name,
    body: prompt.draftBody,
    negativePrompt: prompt.draftNegativePrompt ?? "",
    stylePrompt: prompt.draftStylePrompt ?? "",
    variablesSchemaJson: prompt.variablesSchemaJson || "[]",
    defaultValuesJson: prompt.defaultValuesJson || "{}",
    parameterPresetJson: prompt.parameterPresetJson || "{}",
    notes: prompt.notes ?? "",
  };
}

export function createPromptRunForm(version: PromptVersion | null): PromptRunForm {
  const preset = parseParameterPreset(version?.parameterPresetJson ?? "{}");
  return {
    valuesJson: version?.defaultValuesJson || "{}",
    provider: preset.provider,
    model: preset.model,
    operation: preset.operation,
    parametersJson: version?.parameterPresetJson || "{}",
  };
}

export function promptDocumentMatchesQuery(prompt: PromptDocument, query: string) {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) {
    return true;
  }
  return [prompt.name, prompt.draftBody, prompt.notes ?? ""].some((value) =>
    value.toLocaleLowerCase().includes(needle),
  );
}

export function parseParameterPreset(value: string): Pick<PromptRunForm, "provider" | "model" | "operation"> {
  try {
    const parsed = JSON.parse(value) as { provider?: unknown; model?: unknown; operation?: unknown };
    return {
      provider: typeof parsed.provider === "string" && parsed.provider.trim() ? parsed.provider : "codex-cli",
      model: typeof parsed.model === "string" ? parsed.model : "",
      operation: parsed.operation === "image_to_image" ? "image_to_image" : "text_to_image",
    };
  } catch {
    return { provider: "codex-cli", model: "", operation: "text_to_image" };
  }
}
