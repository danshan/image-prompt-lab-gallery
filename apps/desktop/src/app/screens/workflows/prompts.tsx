import React from "react";
import { Icon } from "../../../studio-icons";
import { displayDate } from "./common";
import type { PromptDocument, PromptOutputHistoryItem, PromptVersion, RenderPromptRun } from "../../types";
import type { PromptDraftForm, PromptRunForm } from "../../workflows/prompts";

export function PromptWorkspace({
  prompts,
  search,
  selectedPromptId,
  draft,
  versions,
  selectedVersionId,
  history,
  runForm,
  renderResult,
  loading,
  versionsLoading,
  saving,
  running,
  onSearchChange,
  onRefresh,
  onSelectPrompt,
  onDraftChange,
  onNewPrompt,
  onSaveDraft,
  onSaveVersion,
  onSelectVersion,
  onRunFormChange,
  onRender,
  onRun,
}: {
  prompts: PromptDocument[];
  search: string;
  selectedPromptId: string | null;
  draft: PromptDraftForm;
  versions: PromptVersion[];
  selectedVersionId: string | null;
  history: PromptOutputHistoryItem[];
  runForm: PromptRunForm;
  renderResult: RenderPromptRun | null;
  loading: boolean;
  versionsLoading: boolean;
  saving: boolean;
  running: boolean;
  onSearchChange: (value: string) => void;
  onRefresh: () => void;
  onSelectPrompt: (promptId: string) => void;
  onDraftChange: (draft: PromptDraftForm) => void;
  onNewPrompt: () => void;
  onSaveDraft: () => void;
  onSaveVersion: () => void;
  onSelectVersion: (versionId: string) => void;
  onRunFormChange: (form: PromptRunForm) => void;
  onRender: () => void;
  onRun: () => void;
}) {
  const selectedPrompt = prompts.find((prompt) => prompt.id === selectedPromptId) ?? null;
  const selectedVersion = versions.find((version) => version.id === selectedVersionId) ?? null;
  const canSaveDraft = draft.name.trim().length > 0 && draft.body.trim().length > 0 && !saving;
  const canUseVersion = Boolean(selectedVersion) && !running;

  return (
    <section className="prompt-workspace">
      <aside className="prompt-library-panel">
        <div className="panel-header compact">
          <div>
            <h3>Prompt Library</h3>
            <p>{loading ? "Loading" : `${prompts.length} prompt${prompts.length === 1 ? "" : "s"}`}</p>
          </div>
          <button className="icon-button" title="New prompt" aria-label="New prompt" onClick={onNewPrompt}>
            <Icon name="plus" />
          </button>
        </div>
        <div className="prompt-search-row">
          <Icon name="search" />
          <input value={search} onChange={(event) => onSearchChange(event.target.value)} placeholder="Search prompts" />
          <button onClick={onRefresh}>Refresh</button>
        </div>
        <div className="prompt-list">
          {prompts.map((prompt) => (
            <button
              key={prompt.id}
              className={prompt.id === selectedPromptId ? "prompt-list-item active" : "prompt-list-item"}
              onClick={() => onSelectPrompt(prompt.id)}
            >
              <span>
                <strong>{prompt.name}</strong>
                <small>{prompt.latestVersionName ?? "draft only"} · {displayDate(prompt.updatedAt)}</small>
              </span>
              <span className="prompt-status">{prompt.status}</span>
            </button>
          ))}
          {prompts.length === 0 && (
            <div className="empty-panel">
              <strong>No prompts</strong>
              <span>Create a prompt draft or adjust search.</span>
            </div>
          )}
        </div>
      </aside>

      <main className="prompt-editor-panel">
        <div className="prompt-editor-header">
          <label>
            <span>Name</span>
            <input value={draft.name} onChange={(event) => onDraftChange({ ...draft, name: event.target.value })} />
          </label>
          <div className="row-actions">
            <button disabled={!canSaveDraft} onClick={onSaveDraft}>Save draft</button>
            <button className="primary-button" disabled={!canSaveDraft} onClick={onSaveVersion}>Save version</button>
          </div>
        </div>
        <div className="prompt-editor-grid">
          <label className="prompt-body-field">
            <span>Prompt body</span>
            <textarea value={draft.body} onChange={(event) => onDraftChange({ ...draft, body: event.target.value })} />
          </label>
          <label>
            <span>Negative</span>
            <textarea value={draft.negativePrompt} onChange={(event) => onDraftChange({ ...draft, negativePrompt: event.target.value })} />
          </label>
          <label>
            <span>Style</span>
            <textarea value={draft.stylePrompt} onChange={(event) => onDraftChange({ ...draft, stylePrompt: event.target.value })} />
          </label>
          <label>
            <span>Variables schema JSON</span>
            <textarea value={draft.variablesSchemaJson} onChange={(event) => onDraftChange({ ...draft, variablesSchemaJson: event.target.value })} />
          </label>
          <label>
            <span>Default values JSON</span>
            <textarea value={draft.defaultValuesJson} onChange={(event) => onDraftChange({ ...draft, defaultValuesJson: event.target.value })} />
          </label>
          <label>
            <span>Parameter preset JSON</span>
            <textarea value={draft.parameterPresetJson} onChange={(event) => onDraftChange({ ...draft, parameterPresetJson: event.target.value })} />
          </label>
          <label className="prompt-notes-field">
            <span>Notes</span>
            <textarea value={draft.notes} onChange={(event) => onDraftChange({ ...draft, notes: event.target.value })} />
          </label>
        </div>
        <section className="prompt-version-strip">
          <div className="panel-header compact">
            <div>
              <h3>Versions</h3>
              <p>{versionsLoading ? "Loading" : `${versions.length} saved`}</p>
            </div>
          </div>
          <div className="prompt-version-list">
            {versions.map((version) => (
              <button
                key={version.id}
                className={version.id === selectedVersionId ? "prompt-version-item active" : "prompt-version-item"}
                onClick={() => onSelectVersion(version.id)}
              >
                <strong>{version.versionName}</strong>
                <span>{displayDate(version.createdAt)}</span>
              </button>
            ))}
            {versions.length === 0 && <span className="muted-inline">Save a version before running this prompt.</span>}
          </div>
        </section>
      </main>

      <aside className="prompt-run-panel">
        <div className="panel-header compact">
          <div>
            <h3>Run</h3>
            <p>{selectedPrompt?.name ?? "No prompt selected"}</p>
          </div>
          <button className="primary-button" disabled={!canUseVersion} onClick={onRun}>Run</button>
        </div>
        <div className="prompt-run-controls">
          <label>
            <span>Version</span>
            <select
              className="select-control"
              value={selectedVersionId ?? ""}
              onChange={(event) => onSelectVersion(event.target.value)}
            >
              {versions.map((version) => (
                <option key={version.id} value={version.id}>
                  {version.versionName}
                </option>
              ))}
            </select>
          </label>
          <div className="prompt-run-row">
            <label>
              <span>Provider</span>
              <select
                className="select-control"
                value={runForm.provider}
                onChange={(event) => onRunFormChange({ ...runForm, provider: event.target.value })}
              >
                <option value="codex-cli">codex-cli</option>
                <option value="fake">fake</option>
                <option value="grok">grok</option>
              </select>
            </label>
            <label>
              <span>Operation</span>
              <select
                className="select-control"
                value={runForm.operation}
                onChange={(event) => onRunFormChange({ ...runForm, operation: event.target.value === "image_to_image" ? "image_to_image" : "text_to_image" })}
              >
                <option value="text_to_image">text_to_image</option>
                <option value="image_to_image">image_to_image</option>
              </select>
            </label>
          </div>
          <label>
            <span>Run values JSON</span>
            <textarea value={runForm.valuesJson} onChange={(event) => onRunFormChange({ ...runForm, valuesJson: event.target.value })} />
          </label>
          <label>
            <span>Parameter JSON</span>
            <textarea value={runForm.parametersJson} onChange={(event) => onRunFormChange({ ...runForm, parametersJson: event.target.value })} />
          </label>
          <button disabled={!canUseVersion} onClick={onRender}>Render</button>
        </div>
        <section className="prompt-render-preview">
          <div className="prompt-section-title">
            <strong>Rendered</strong>
            <span>{renderResult?.versionName ?? selectedVersion?.versionName ?? "-"}</span>
          </div>
          <pre>{renderResult?.renderedPrompt ?? selectedVersion?.body ?? "Select a version and render."}</pre>
          <small>{renderResult?.renderedNegativePrompt ?? selectedVersion?.negativePrompt ?? "No negative prompt."}</small>
        </section>
        <section className="prompt-history-panel">
          <div className="prompt-section-title">
            <strong>Output history</strong>
            <span>{history.length}</span>
          </div>
          <div className="prompt-history-list">
            {history.map((item) => (
              <article key={item.generationEventId} className="prompt-history-item">
                <strong>{item.status}</strong>
                <span>{item.provider}/{item.providerModel}</span>
                <small>{item.outputVersionId ?? item.assetId ?? item.taskId ?? item.generationEventId}</small>
              </article>
            ))}
            {history.length === 0 && <span className="muted-inline">No linked outputs yet.</span>}
          </div>
        </section>
      </aside>
    </section>
  );
}
