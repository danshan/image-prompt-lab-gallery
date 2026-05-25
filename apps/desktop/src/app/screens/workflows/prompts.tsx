import React from "react";
import { Icon } from "../../../studio-icons";
import { displayDate } from "./common";
import type { PromptDocument, PromptOutputHistoryItem, PromptVersion, RenderPromptRun } from "../../types";
import type { PromptDraftForm, PromptRunForm } from "../../workflows/prompts";
import type { Dictionary } from "../../i18n/dictionaries";

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
  onArchivePrompt,
  onSelectVersion,
  onRunFormChange,
  onRender,
  onRun,
  dictionary,
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
  onArchivePrompt: (promptId: string) => void;
  onSelectVersion: (versionId: string) => void;
  onRunFormChange: (form: PromptRunForm) => void;
  onRender: () => void;
  onRun: () => void;
  dictionary: Dictionary;
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
            <h3>{dictionary.workflow.promptLibrary}</h3>
            <p>{loading ? dictionary.workflow.loading : `${prompts.length} ${prompts.length === 1 ? dictionary.workflow.promptCountSingular : dictionary.workflow.promptCountPlural}`}</p>
          </div>
          <button className="icon-button" title={dictionary.workflow.newPrompt} aria-label={dictionary.workflow.newPrompt} onClick={onNewPrompt}>
            <Icon name="plus" />
          </button>
        </div>
        <div className="prompt-search-row">
          <Icon name="search" />
          <input value={search} onChange={(event) => onSearchChange(event.target.value)} placeholder={dictionary.workflow.searchPrompts} />
          <button onClick={onRefresh}>{dictionary.workflow.refresh}</button>
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
                <small>{prompt.latestVersionName ?? dictionary.workflow.draftOnly} · {displayDate(prompt.updatedAt)}</small>
              </span>
              <span className="row-actions">
                <span className="prompt-status">{prompt.status}</span>
                <span
                  className="icon-button tooltip-button"
                  role="button"
                  tabIndex={0}
                  aria-label="Archive"
                  data-tooltip="Archive"
                  onClick={(event) => {
                    event.stopPropagation();
                    onArchivePrompt(prompt.id);
                  }}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      event.stopPropagation();
                      onArchivePrompt(prompt.id);
                    }
                  }}
                >
                  <Icon name="close" />
                </span>
              </span>
            </button>
          ))}
          {prompts.length === 0 && (
            <div className="empty-panel">
              <strong>{dictionary.workflow.noPrompts}</strong>
              <span>{dictionary.workflow.noPromptsHint}</span>
            </div>
          )}
        </div>
      </aside>

      <main className="prompt-editor-panel">
        <div className="prompt-editor-header">
          <label>
            <span>{dictionary.workflow.name}</span>
            <input value={draft.name} onChange={(event) => onDraftChange({ ...draft, name: event.target.value })} />
          </label>
          <div className="row-actions">
            <button disabled={!canSaveDraft} onClick={onSaveDraft}>{dictionary.workflow.saveDraft}</button>
            <button className="primary-button" disabled={!canSaveDraft} onClick={onSaveVersion}>{dictionary.workflow.saveVersion}</button>
          </div>
        </div>
        <div className="prompt-editor-grid">
          <label className="prompt-body-field">
            <span>{dictionary.workflow.promptBody}</span>
            <textarea value={draft.body} onChange={(event) => onDraftChange({ ...draft, body: event.target.value })} />
          </label>
          <label>
            <span>{dictionary.workflow.negative}</span>
            <textarea value={draft.negativePrompt} onChange={(event) => onDraftChange({ ...draft, negativePrompt: event.target.value })} />
          </label>
          <label>
            <span>{dictionary.workflow.style}</span>
            <textarea value={draft.stylePrompt} onChange={(event) => onDraftChange({ ...draft, stylePrompt: event.target.value })} />
          </label>
          <label>
            <span>{dictionary.workflow.variablesSchemaJson}</span>
            <textarea value={draft.variablesSchemaJson} onChange={(event) => onDraftChange({ ...draft, variablesSchemaJson: event.target.value })} />
          </label>
          <label>
            <span>{dictionary.workflow.defaultValuesJson}</span>
            <textarea value={draft.defaultValuesJson} onChange={(event) => onDraftChange({ ...draft, defaultValuesJson: event.target.value })} />
          </label>
          <label className="prompt-parameter-preset-field">
            <span>{dictionary.workflow.parameterPresetJson}</span>
            <textarea value={draft.parameterPresetJson} onChange={(event) => onDraftChange({ ...draft, parameterPresetJson: event.target.value })} />
          </label>
          <label className="prompt-notes-field">
            <span>{dictionary.workflow.notes}</span>
            <textarea value={draft.notes} onChange={(event) => onDraftChange({ ...draft, notes: event.target.value })} />
          </label>
        </div>
        <section className="prompt-version-strip">
          <div className="panel-header compact">
            <div>
              <h3>{dictionary.workflow.versions}</h3>
              <p>{versionsLoading ? dictionary.workflow.loading : `${versions.length} ${dictionary.workflow.saved}`}</p>
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
            {versions.length === 0 && <span className="muted-inline">{dictionary.workflow.saveVersionBeforeRun}</span>}
          </div>
        </section>
      </main>

      <aside className="prompt-run-panel">
        <div className="panel-header compact">
          <div>
            <h3>{dictionary.workflow.run}</h3>
            <p>{selectedPrompt?.name ?? dictionary.workflow.noPromptSelected}</p>
          </div>
          <button className="primary-button" disabled={!canUseVersion} onClick={onRun}>{dictionary.workflow.run}</button>
        </div>
        <div className="prompt-run-controls">
          <label>
            <span>{dictionary.workflow.version}</span>
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
              <span>{dictionary.workflow.provider}</span>
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
              <span>{dictionary.workflow.model}</span>
              <input
                value={runForm.model}
                onChange={(event) => onRunFormChange({ ...runForm, model: event.target.value })}
                placeholder={dictionary.workflow.providerDefault}
              />
            </label>
          </div>
          <div className="prompt-run-row">
            <label>
              <span>{dictionary.workflow.operation}</span>
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
            <span>{dictionary.workflow.runValuesJson}</span>
            <textarea value={runForm.valuesJson} onChange={(event) => onRunFormChange({ ...runForm, valuesJson: event.target.value })} />
          </label>
          <label>
            <span>{dictionary.workflow.parameterJson}</span>
            <textarea value={runForm.parametersJson} onChange={(event) => onRunFormChange({ ...runForm, parametersJson: event.target.value })} />
          </label>
          <button disabled={!canUseVersion} onClick={onRender}>{dictionary.workflow.render}</button>
        </div>
        <section className="prompt-render-preview">
          <div className="prompt-section-title">
            <strong>{dictionary.workflow.rendered}</strong>
            <span>{renderResult?.versionName ?? selectedVersion?.versionName ?? "-"}</span>
          </div>
          <pre>{renderResult?.renderedPrompt ?? selectedVersion?.body ?? dictionary.workflow.selectVersionAndRender}</pre>
          <small>{renderResult?.renderedNegativePrompt ?? selectedVersion?.negativePrompt ?? dictionary.workflow.noNegativePrompt}</small>
        </section>
        <section className="prompt-history-panel">
          <div className="prompt-section-title">
            <strong>{dictionary.workflow.outputHistory}</strong>
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
            {history.length === 0 && <span className="muted-inline">{dictionary.workflow.noLinkedOutputs}</span>}
          </div>
        </section>
      </aside>
    </section>
  );
}
