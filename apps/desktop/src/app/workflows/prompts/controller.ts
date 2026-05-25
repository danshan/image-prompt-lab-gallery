import type { Dispatch, SetStateAction } from "react";
import { errorMessage, invokeCommand } from "../../tauri-adapter";
import type {
  DaemonTask,
  Library,
  PromptDocument,
  PromptOutputHistoryItem,
  PromptVersion,
  RenderPromptRun,
  View,
} from "../../types";
import type { TaskPanel } from "../../types";
import {
  createPromptDraftForm,
  createPromptRunForm,
  emptyPromptDraftForm,
  emptyPromptRunForm,
  mockPromptDocuments,
  mockPromptOutputHistory,
  mockPromptVersions,
  promptDocumentMatchesQuery,
  type PromptDraftForm,
  type PromptRunForm,
  type PromptWorkspaceControllerState,
} from "./state";
import { mergeTasks } from "../tasks";

type PromptWorkspaceActionsInput = PromptWorkspaceControllerState & {
  runningInTauri: boolean;
  library: Library | null;
  setTasks: Dispatch<SetStateAction<DaemonTask[]>>;
  setSelectedTaskId: Dispatch<SetStateAction<string | null>>;
  setActiveView: Dispatch<SetStateAction<View>>;
  setActiveTaskPanel: Dispatch<SetStateAction<TaskPanel>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
  refreshTasks: () => Promise<unknown>;
};

export function usePromptWorkspaceActions({
  runningInTauri,
  library,
  prompts,
  setPrompts,
  promptSearch,
  selectedPromptId,
  setSelectedPromptId,
  promptDraftForm,
  setPromptDraftForm,
  promptVersions,
  setPromptVersions,
  selectedPromptVersionId,
  setSelectedPromptVersionId,
  setPromptOutputHistory,
  promptRunForm,
  setPromptRunForm,
  setPromptRenderResult,
  setPromptsLoading,
  setPromptVersionsLoading,
  setPromptSaving,
  setPromptRunning,
  setTasks,
  setSelectedTaskId,
  setActiveView,
  setActiveTaskPanel,
  setStatus,
  setRecoverableError,
  refreshTasks,
}: PromptWorkspaceActionsInput) {
  async function refreshPrompts(nextSearch = promptSearch) {
    if (!runningInTauri || !library) {
      const items = mockPromptDocuments.filter((prompt) => promptDocumentMatchesQuery(prompt, nextSearch));
      setPrompts(items);
      selectPrompt(items.some((item) => item.id === selectedPromptId) ? selectedPromptId : items[0]?.id ?? null, items);
      return;
    }
    setPromptsLoading(true);
    try {
      const items = await invokeCommand<PromptDocument[]>("list_prompt_documents", {
        input: {
          libraryPath: library.rootPath,
          query: nextSearch.trim() || null,
          includeArchived: false,
        },
      });
      setPrompts(items);
      const nextPromptId = items.some((item) => item.id === selectedPromptId) ? selectedPromptId : items[0]?.id ?? null;
      await selectPrompt(nextPromptId, items);
      setRecoverableError(null);
    } catch (error) {
      setPrompts([]);
      setPromptVersions([]);
      setSelectedPromptId(null);
      setSelectedPromptVersionId(null);
      setPromptDraftForm(emptyPromptDraftForm);
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptsLoading(false);
    }
  }

  async function selectPrompt(promptId: string | null, sourcePrompts = prompts) {
    const prompt = sourcePrompts.find((item) => item.id === promptId) ?? null;
    setSelectedPromptId(prompt?.id ?? null);
    setPromptDraftForm(createPromptDraftForm(prompt));
    setPromptRenderResult(null);
    if (!prompt) {
      setPromptVersions([]);
      setSelectedPromptVersionId(null);
      setPromptOutputHistory([]);
      return;
    }
    await refreshPromptVersions(prompt.id);
  }

  async function refreshPromptVersions(promptId = selectedPromptId) {
    if (!promptId) {
      setPromptVersions([]);
      setSelectedPromptVersionId(null);
      setPromptOutputHistory([]);
      return;
    }
    if (!runningInTauri || !library) {
      const versions = mockPromptVersions.filter((version) => version.promptId === promptId);
      setPromptVersions(versions);
      const nextVersionId = versions[0]?.id ?? null;
      selectPromptVersion(nextVersionId, versions);
      return;
    }
    setPromptVersionsLoading(true);
    try {
      const versions = await invokeCommand<PromptVersion[]>("list_prompt_versions", {
        input: {
          libraryPath: library.rootPath,
          promptId,
        },
      });
      setPromptVersions(versions);
      const nextVersionId = versions.some((version) => version.id === selectedPromptVersionId)
        ? selectedPromptVersionId
        : versions[0]?.id ?? null;
      await selectPromptVersion(nextVersionId, versions);
      setRecoverableError(null);
    } catch (error) {
      setPromptVersions([]);
      setSelectedPromptVersionId(null);
      setPromptOutputHistory([]);
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptVersionsLoading(false);
    }
  }

  async function openPromptVersion(promptId: string, versionId: string) {
    setPromptsLoading(true);
    setPromptVersionsLoading(true);
    try {
      if (!runningInTauri || !library) {
        const prompt = mockPromptDocuments.find((item) => item.id === promptId) ?? null;
        const versions = mockPromptVersions.filter((version) => version.promptId === promptId);
        if (prompt && !prompts.some((item) => item.id === prompt.id)) {
          setPrompts((current) => [prompt, ...current]);
        }
        setSelectedPromptId(prompt?.id ?? null);
        setPromptDraftForm(createPromptDraftForm(prompt));
        setPromptVersions(versions);
        await selectPromptVersion(versionId, versions);
        return;
      }
      const documents = await invokeCommand<PromptDocument[]>("list_prompt_documents", {
        input: {
          libraryPath: library.rootPath,
          query: null,
          includeArchived: true,
        },
      });
      const prompt = documents.find((item) => item.id === promptId) ?? null;
      const versions = await invokeCommand<PromptVersion[]>("list_prompt_versions", {
        input: {
          libraryPath: library.rootPath,
          promptId,
        },
      });
      setPrompts(documents);
      setSelectedPromptId(prompt?.id ?? null);
      setPromptDraftForm(createPromptDraftForm(prompt));
      setPromptVersions(versions);
      await selectPromptVersion(versionId, versions);
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptsLoading(false);
      setPromptVersionsLoading(false);
    }
  }

  async function createPrompt(): Promise<PromptDocument | null> {
    const name = promptDraftForm.name.trim() || "Untitled Prompt";
    const body = promptDraftForm.body.trim();
    if (!body) {
      setRecoverableError("Enter a prompt body before creating a prompt.");
      return null;
    }
    setPromptSaving(true);
    try {
      if (!runningInTauri || !library) {
        const created = createPreviewPrompt(promptDraftForm);
        setPrompts((current) => [created, ...current]);
        await selectPrompt(created.id, [created, ...prompts]);
        setStatus("Prompt created");
        setRecoverableError(null);
        return created;
      }
      const created = await invokeCommand<PromptDocument>("create_prompt_document", {
        input: promptDraftInput(library.rootPath, { ...promptDraftForm, name }),
      });
      await refreshPrompts("");
      await selectPrompt(created.id, [created, ...prompts]);
      setStatus("Prompt created");
      setRecoverableError(null);
      return created;
    } catch (error) {
      setRecoverableError(errorMessage(error));
      return null;
    } finally {
      setPromptSaving(false);
    }
  }

  async function saveDraft(): Promise<boolean> {
    const promptId = selectedPromptId;
    const current = prompts.find((item) => item.id === promptId) ?? null;
    if (!promptId || !current) {
      return Boolean(await createPrompt());
    }
    setPromptSaving(true);
    try {
      if (!runningInTauri || !library) {
        const updated: PromptDocument = {
          ...current,
          ...documentFieldsFromForm(promptDraftForm),
          updatedAt: "Now",
        };
        setPrompts((items) => items.map((item) => (item.id === updated.id ? updated : item)));
        setStatus("Prompt draft saved");
        setRecoverableError(null);
        return true;
      }
      const updated = await invokeCommand<PromptDocument>("update_prompt_draft", {
        input: {
          ...promptDraftInput(library.rootPath, promptDraftForm),
          promptId,
        },
      });
      setPrompts((items) => items.map((item) => (item.id === updated.id ? updated : item)));
      setPromptDraftForm(createPromptDraftForm(updated));
      setStatus("Prompt draft saved");
      setRecoverableError(null);
      return true;
    } catch (error) {
      setRecoverableError(errorMessage(error));
      return false;
    } finally {
      setPromptSaving(false);
    }
  }

  async function saveVersion() {
    const promptId = selectedPromptId;
    if (!promptId) {
      setRecoverableError("Select or create a prompt before saving a version.");
      return;
    }
    setPromptSaving(true);
    try {
      if (!runningInTauri || !library) {
        const nextNumber = promptVersions.reduce((max, version) => Math.max(max, version.versionNumber), 0) + 1;
        const version = createPreviewVersion(promptId, promptDraftForm, nextNumber);
        const nextVersions = [version, ...promptVersions];
        setPromptVersions(nextVersions);
        setPrompts((items) =>
          items.map((item) =>
            item.id === promptId
              ? {
                  ...item,
                  latestVersionId: version.id,
                  latestVersionNumber: version.versionNumber,
                  latestVersionName: version.versionName,
                  updatedAt: "Now",
                }
              : item,
          ),
        );
        await selectPromptVersion(version.id, nextVersions);
        setStatus("Prompt version saved");
        setRecoverableError(null);
        return;
      }
      const draftSaved = await saveDraft();
      if (!draftSaved) {
        return;
      }
      const version = await invokeCommand<PromptVersion>("save_prompt_version", {
        input: {
          libraryPath: library.rootPath,
          promptId,
        },
      });
      await refreshPromptVersions(promptId);
      await selectPromptVersion(version.id, [version, ...promptVersions]);
      setStatus("Prompt version saved");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptSaving(false);
    }
  }

  async function selectPromptVersion(versionId: string | null, sourceVersions = promptVersions) {
    const version = sourceVersions.find((item) => item.id === versionId) ?? null;
    setSelectedPromptVersionId(version?.id ?? null);
    setPromptRenderResult(null);
    setPromptRunForm(createPromptRunForm(version));
    if (!version) {
      setPromptOutputHistory([]);
      return;
    }
    await refreshPromptHistory(version.id);
  }

  async function refreshPromptHistory(versionId = selectedPromptVersionId) {
    if (!versionId) {
      setPromptOutputHistory([]);
      return;
    }
    if (!runningInTauri || !library) {
      setPromptOutputHistory(mockPromptOutputHistory.filter((item) => item.generationEventId.includes("botanical")));
      return;
    }
    try {
      const history = await invokeCommand<PromptOutputHistoryItem[]>("list_prompt_output_history", {
        input: {
          libraryPath: library.rootPath,
          promptVersionId: versionId,
        },
      });
      setPromptOutputHistory(history);
      setRecoverableError(null);
    } catch (error) {
      setPromptOutputHistory([]);
      setRecoverableError(errorMessage(error));
    }
  }

  async function renderSelectedPrompt() {
    const version = promptVersions.find((item) => item.id === selectedPromptVersionId) ?? null;
    if (!version) {
      setRecoverableError("Select a saved prompt version before rendering.");
      return null;
    }
    setPromptRunning(true);
    try {
      if (!runningInTauri || !library) {
        const rendered = renderPreviewPrompt(version, promptRunForm.valuesJson);
        setPromptRenderResult(rendered);
        setStatus("Prompt rendered");
        setRecoverableError(null);
        return rendered;
      }
      const rendered = await invokeCommand<RenderPromptRun>("render_prompt_run", {
        input: {
          libraryPath: library.rootPath,
          promptVersionId: version.id,
          valuesJson: promptRunForm.valuesJson,
        },
      });
      setPromptRenderResult(rendered);
      setStatus("Prompt rendered");
      setRecoverableError(null);
      return rendered;
    } catch (error) {
      setRecoverableError(errorMessage(error));
      return null;
    } finally {
      setPromptRunning(false);
    }
  }

  async function runSelectedPrompt() {
    if (!library) {
      setRecoverableError("Open a real library before running a prompt.");
      return;
    }
    const rendered = await renderSelectedPrompt();
    if (!rendered) {
      return;
    }
    if (!runningInTauri) {
      setStatus("Preview prompt rendered");
      setRecoverableError(null);
      return;
    }
    setPromptRunning(true);
    try {
      const created = await invokeCommand<DaemonTask[]>("enqueue_generation_tasks", {
        input: {
          libraryPath: library.rootPath,
          tasks: [
            {
              provider: promptRunForm.provider,
              prompt: rendered.renderedPrompt,
              negativePrompt: rendered.renderedNegativePrompt,
              promptVersionId: rendered.promptVersionId,
              model: promptRunForm.model.trim() || null,
              valuesJson: rendered.valuesJson,
              operation: promptRunForm.operation,
              inputFile: null,
              inputVersionId: null,
              parametersJson: promptRunForm.parametersJson || rendered.parameterPresetJson,
              priority: 0,
              maxAttempts: 3,
            },
          ],
        },
      });
      setTasks((current) => mergeTasks(created, current));
      setSelectedTaskId(created[0]?.id ?? null);
      setActiveView("queue");
      setActiveTaskPanel("queue");
      setStatus("Prompt generation enqueued");
      setRecoverableError(null);
      void refreshTasks();
      void refreshPromptHistory(rendered.promptVersionId);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptRunning(false);
    }
  }

  function newPromptDraft() {
    setSelectedPromptId(null);
    setPromptVersions([]);
    setSelectedPromptVersionId(null);
    setPromptOutputHistory([]);
    setPromptRenderResult(null);
    setPromptDraftForm(emptyPromptDraftForm);
    setPromptRunForm(emptyPromptRunForm);
  }

  async function archivePrompt(promptId = selectedPromptId) {
    if (!promptId) {
      return;
    }
    setPromptSaving(true);
    try {
      if (!runningInTauri || !library) {
        const nextPrompts = prompts.filter((prompt) => prompt.id !== promptId);
        setPrompts(nextPrompts);
        await selectPrompt(nextPrompts[0]?.id ?? null, nextPrompts);
        setStatus("Prompt archived");
        setRecoverableError(null);
        return;
      }
      await invokeCommand<void>("archive_prompt_document", {
        input: {
          libraryPath: library.rootPath,
          promptId,
        },
      });
      await refreshPrompts(promptSearch);
      setStatus("Prompt archived");
      setRecoverableError(null);
    } catch (error) {
      setRecoverableError(errorMessage(error));
    } finally {
      setPromptSaving(false);
    }
  }

  return {
    refreshPrompts,
    selectPrompt,
    createPrompt,
    saveDraft,
    saveVersion,
    selectPromptVersion,
    openPromptVersion,
    renderSelectedPrompt,
    runSelectedPrompt,
    newPromptDraft,
    archivePrompt,
  };
}

function promptDraftInput(libraryPath: string, form: PromptDraftForm) {
  return {
    libraryPath,
    name: form.name.trim() || "Untitled Prompt",
    draftBody: form.body,
    draftNegativePrompt: form.negativePrompt.trim() || null,
    draftStylePrompt: form.stylePrompt.trim() || null,
    variablesSchemaJson: form.variablesSchemaJson.trim() || "[]",
    defaultValuesJson: form.defaultValuesJson.trim() || "{}",
    parameterPresetJson: form.parameterPresetJson.trim() || "{}",
    notes: form.notes.trim() || null,
  };
}

function documentFieldsFromForm(form: PromptDraftForm) {
  return {
    name: form.name.trim() || "Untitled Prompt",
    draftBody: form.body,
    draftNegativePrompt: form.negativePrompt.trim() || null,
    draftStylePrompt: form.stylePrompt.trim() || null,
    variablesSchemaJson: form.variablesSchemaJson.trim() || "[]",
    defaultValuesJson: form.defaultValuesJson.trim() || "{}",
    parameterPresetJson: form.parameterPresetJson.trim() || "{}",
    notes: form.notes.trim() || null,
  };
}

function createPreviewPrompt(form: PromptDraftForm): PromptDocument {
  return {
    id: `prompt-${crypto.randomUUID()}`,
    kind: "image_generation",
    status: "active",
    latestVersionId: null,
    latestVersionNumber: null,
    latestVersionName: null,
    createdAt: "Now",
    updatedAt: "Now",
    archivedAt: null,
    ...documentFieldsFromForm(form),
  };
}

function createPreviewVersion(promptId: string, form: PromptDraftForm, versionNumber: number): PromptVersion {
  return {
    id: `prompt-version-${crypto.randomUUID()}`,
    promptId,
    versionNumber,
    versionName: `v${versionNumber}`,
    body: form.body,
    negativePrompt: form.negativePrompt.trim() || null,
    stylePrompt: form.stylePrompt.trim() || null,
    variablesSchemaJson: form.variablesSchemaJson.trim() || "[]",
    defaultValuesJson: form.defaultValuesJson.trim() || "{}",
    parameterPresetJson: form.parameterPresetJson.trim() || "{}",
    notes: form.notes.trim() || null,
    createdAt: "Now",
  };
}

function renderPreviewPrompt(version: PromptVersion, valuesJson: string): RenderPromptRun {
  const values = parseObject(valuesJson);
  const defaults = parseObject(version.defaultValuesJson);
  const merged = { ...defaults, ...values };
  const replaceVariables = (source: string) =>
    source.replace(/\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}/g, (_match, name: string) =>
      typeof merged[name] === "string" || typeof merged[name] === "number" ? String(merged[name]) : "",
    );
  const body = replaceVariables(version.body);
  const style = version.stylePrompt ? replaceVariables(version.stylePrompt) : "";
  return {
    promptVersionId: version.id,
    promptId: version.promptId,
    versionNumber: version.versionNumber,
    versionName: version.versionName,
    renderedPrompt: [body, style].filter(Boolean).join("\n\n"),
    renderedNegativePrompt: version.negativePrompt ? replaceVariables(version.negativePrompt) : null,
    valuesJson,
    parameterPresetJson: version.parameterPresetJson,
  };
}

function parseObject(value: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(value) as unknown;
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed as Record<string, unknown> : {};
  } catch {
    return {};
  }
}
