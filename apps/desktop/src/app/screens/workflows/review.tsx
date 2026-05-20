import React, { useEffect, useState } from "react";
import {
  addReviewFormTag,
  applySuggestionFieldToReviewForm,
  clearAlbumQuery,
  formatAspectRatio,
  isReviewFieldGenerating,
  libraryMaintenanceActions,
  moveItem,
  parseTaskDraftImport,
  removeReviewFormTag,
  resetGalleryQuery,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleGalleryProvider,
  updateGalleryQuery,
  type GalleryQueryState,
  type GallerySort,
  type DetailLoadState,
  type ReviewFieldName,
  type ReviewFormState,
  type ReviewStatusFilter,
  type SettingsSection,
} from "../../../workbench-state";
import { Icon } from "../../../studio-icons";
import {
  formatOperation,
  formatVersionName,
  isRetryableTaskStatus,
  isTerminalFailureStatus,
  shortIdentifier,
  statusLabel,
  taskActionKey,
  taskPrompt,
  compareTaskOrder,
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
export function ReviewWorkspace({
  suggestions,
  selectedSuggestion,
  selectedSuggestionIds,
  suggestionHistory,
  suggestionRegenerating,
  form,
  onSelect,
  onToggleSelected,
  onFormChange,
  availableTags,
  availableCategories,
  albums,
  tasks,
  onRestore,
  onRegenerateField,
  onRegenerateSuggestion,
  onPickHistoryField,
  onAccept,
  onBatchAccept,
  onBatchReject,
  onAddToAlbum,
  onOpenTask,
}: {
  suggestions: Suggestion[];
  selectedSuggestion: Suggestion | null;
  selectedSuggestionIds: string[];
  suggestionHistory: Suggestion[];
  suggestionRegenerating: boolean;
  form: ReviewFormState | null;
  onSelect: (suggestion: Suggestion) => void;
  onToggleSelected: (suggestionId: string) => void;
  onFormChange: (form: ReviewFormState) => void;
  availableTags: string[];
  availableCategories: string[];
  albums: AlbumListItem[];
  tasks: DaemonTask[];
  onRestore: () => void;
  onRegenerateField: (field: ReviewFieldName) => void;
  onRegenerateSuggestion: () => void;
  onPickHistoryField: (suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") => void;
  onAccept: () => void;
  onBatchAccept: () => void;
  onBatchReject: () => void;
  onAddToAlbum: (albumId: string) => void;
  onOpenTask: (taskId: string) => void;
}) {
  const [albumToAdd, setAlbumToAdd] = useState("");
  if (suggestions.length === 0) {
    return <div className="empty-state">No pending suggestions.</div>;
  }
  return (
    <section className="review-workspace">
      <div className="workspace-panel review-list-panel">
        <div className="panel-header">
          <div>
            <h3>Pending review</h3>
            <p>{selectedSuggestionIds.length || suggestions.length} selected / {suggestions.length} pending</p>
          </div>
        </div>
        <div className="row-actions review-actions">
          <button onClick={onBatchAccept}>Accept selected</button>
          <button onClick={onBatchReject}>Reject selected</button>
        </div>
        <div className="add-album-row">
          <select className="select-control" value={albumToAdd} onChange={(event) => setAlbumToAdd(event.target.value)}>
            <option value="">Add selected to album</option>
            {albums
              .filter((album) => album.kind === "manual")
              .map((album) => (
                <option key={album.id} value={album.id}>{album.name}</option>
              ))}
          </select>
          <button disabled={!albumToAdd} onClick={() => onAddToAlbum(albumToAdd)}>Add</button>
        </div>
        <div className="review-list">
          {suggestions.map((suggestion) => (
            <div
              className={suggestion.id === selectedSuggestion?.id ? "review-list-item selected" : "review-list-item"}
              key={suggestion.id}
            >
              <input
                type="checkbox"
                checked={selectedSuggestionIds.includes(suggestion.id)}
                onChange={() => onToggleSelected(suggestion.id)}
                aria-label={`Select ${suggestion.title ?? suggestion.id}`}
              />
              <button type="button" onClick={() => onSelect(suggestion)}>
                <strong>{suggestion.title ?? "Untitled suggestion"}</strong>
                <span>{suggestion.category ?? "No category"}</span>
                <small>{suggestion.tags.join(", ") || "No tags"}</small>
              </button>
            </div>
          ))}
        </div>
      </div>
      <div className="workspace-panel review-detail-panel">
        {!selectedSuggestion || !form ? (
          <div className="empty-state">Select a suggestion to review.</div>
        ) : (
          <>
            <div className="panel-header">
              <div>
                <h3>Review metadata</h3>
                <p>Status: {selectedSuggestion.status}</p>
              </div>
              <span className="review-badge">Review pending</span>
            </div>
            <ConfidenceSummary confidence={selectedSuggestion.confidence} />
            <ReviewTaskMirror
              tasks={tasks}
              suggestionId={selectedSuggestion.id}
              onOpenTask={onOpenTask}
            />
            <div className="review-form">
              <label>
                <span>
                  Title
                  <ReviewFieldGenerateButton form={form} field="title" onRegenerateField={onRegenerateField} />
                </span>
                <input
                  value={form.title}
                  disabled={isReviewFieldGenerating(form, "title")}
                  onChange={(event) => onFormChange({ ...form, title: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="title" />
              </label>
              <label>
                <span>Category</span>
                <select
                  className="select-control"
                  value={form.category}
                  onChange={(event) => onFormChange({ ...form, category: event.target.value })}
                >
                  <option value="">No category</option>
                  {availableCategories.map((category) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
                </select>
              </label>
              <label className="wide-field">
                <span>
                  Description
                  <ReviewFieldGenerateButton form={form} field="description" onRegenerateField={onRegenerateField} />
                </span>
                <textarea
                  value={form.description}
                  disabled={isReviewFieldGenerating(form, "description")}
                  onChange={(event) => onFormChange({ ...form, description: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="description" />
              </label>
              <label className="wide-field">
                <span>
                  JSON Schema Prompt
                  <ReviewFieldGenerateButton form={form} field="schemaPrompt" onRegenerateField={onRegenerateField} />
                </span>
                <textarea
                  className="schema-prompt-input"
                  value={form.schemaPrompt}
                  disabled={isReviewFieldGenerating(form, "schemaPrompt")}
                  onChange={(event) => onFormChange({ ...form, schemaPrompt: event.target.value })}
                  spellCheck={false}
                />
                <ReviewFieldGenerationStatus form={form} field="schemaPrompt" />
              </label>
              <div className="wide-field tag-editor-field">
                <span>Tags</span>
                <div className="tag-chip-editor">
                  {form.tags.map((tag) => (
                    <button
                      key={tag}
                      className="tag-chip removable"
                      onClick={() => onFormChange(removeReviewFormTag(form, tag))}
                    >
                      {tag} x
                    </button>
                  ))}
                  <input
                    list="review-tag-options"
                    value={form.tagInput}
                    onChange={(event) => onFormChange({ ...form, tagInput: event.target.value })}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        onFormChange(addReviewFormTag(form, form.tagInput));
                      }
                    }}
                    placeholder="Add tag"
                  />
                  <datalist id="review-tag-options">
                    {availableTags.map((tag) => (
                      <option key={tag} value={tag} />
                    ))}
                  </datalist>
                </div>
              </div>
            </div>
            <div className="row-actions review-actions">
              <button onClick={onRestore}>Restore</button>
              <button disabled={suggestionRegenerating} onClick={onRegenerateSuggestion}>
                {suggestionRegenerating ? "Regenerating..." : "Regenerate suggestion"}
              </button>
              <button className="primary-button" onClick={onAccept}>
                Accept changes
              </button>
            </div>
            <SuggestionHistoryTable history={suggestionHistory} onPickField={onPickHistoryField} />
          </>
        )}
      </div>
    </section>
  );
}

function ConfidenceSummary({ confidence }: { confidence?: ConfidenceScore }) {
  if (!confidence) {
    return null;
  }
  const fields = [
    ["Title", confidence.title],
    ["Description", confidence.description],
    ["Schema", confidence.schemaPrompt],
    ["Tags", confidence.tags],
    ["Category", confidence.category],
  ] as const;
  const knownFields = fields.filter(([, score]) => typeof score === "number");
  if (typeof confidence.overall !== "number" && knownFields.length === 0) {
    return null;
  }
  return (
    <div className="confidence-panel">
      <strong>Confidence {formatScore(confidence.overall)}</strong>
      <div className="tag-list">
        {knownFields.map(([label, score]) => (
          <span key={label}>{label}: {formatScore(score)}</span>
        ))}
      </div>
    </div>
  );
}

function SuggestionHistoryTable({
  history,
  onPickField,
}: {
  history: Suggestion[];
  onPickField: (suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") => void;
}) {
  if (history.length === 0) {
    return <div className="empty-state compact">No suggestion history.</div>;
  }
  return (
    <div className="history-table">
      <h3>Suggestion history</h3>
      {history.map((suggestion) => (
        <article key={suggestion.id} className="history-row">
          <div>
            <strong>{suggestion.title ?? "Untitled"}</strong>
            <small>{suggestion.status} · {suggestion.createdAt ? displayDate(suggestion.createdAt) : "-"}</small>
          </div>
          <div className="row-actions">
            <button onClick={() => onPickField(suggestion, "title")}>Title</button>
            <button onClick={() => onPickField(suggestion, "description")}>Description</button>
            <button onClick={() => onPickField(suggestion, "schemaPrompt")}>Schema</button>
            <button onClick={() => onPickField(suggestion, "tags")}>Tags</button>
            <button onClick={() => onPickField(suggestion, "category")}>Category</button>
          </div>
        </article>
      ))}
    </div>
  );
}

function formatScore(score: number | null | undefined): string {
  return typeof score === "number" ? `${score}%` : "unknown";
}

function ReviewFieldGenerateButton({
  form,
  field,
  onRegenerateField,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
  onRegenerateField: (field: ReviewFieldName) => void;
}) {
  const loading = isReviewFieldGenerating(form, field);
  return (
    <button
      type="button"
      className="inline-action"
      disabled={loading}
      onClick={() => onRegenerateField(field)}
    >
      {loading ? "Generating..." : "Regenerate"}
    </button>
  );
}

function ReviewFieldGenerationStatus({
  form,
  field,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
}) {
  const state = form.generation[field];
  if (state.error) {
    return <small className="field-status error-text">Generation failed. Check the message above or Settings Logs.</small>;
  }
  return null;
}

function ReviewTaskMirror({
  tasks,
  suggestionId,
  onOpenTask,
}: {
  tasks: DaemonTask[];
  suggestionId: string;
  onOpenTask: (taskId: string) => void;
}) {
  const related = tasks.filter((task) => {
    const input = task.input ?? {};
    return (
      (task.taskType === "metadata_field_generation" || task.taskType === "metadata_suggestion_generation") &&
      input.suggestionId === suggestionId
    );
  });
  if (related.length === 0) {
    return null;
  }
  return (
    <section className="review-task-mirror">
      {related.slice(0, 3).map((task) => (
        <button key={task.id} onClick={() => onOpenTask(task.id)}>
          <span className={`status ${task.status}`}>{statusLabel(task.status)}</span>
          <span>{task.waitReason ?? task.lastErrorMessage ?? task.taskType}</span>
        </button>
      ))}
    </section>
  );
}
