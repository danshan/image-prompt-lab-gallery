import React, { useEffect, useState } from "react";
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
import {
  addReviewFormTag,
  isReviewFieldGenerating,
  removeReviewFormTag,
  reviewFormTags,
  type ReviewFieldName,
  type ReviewFormState,
} from "../../workflows/review";
import type { Dictionary } from "../../i18n/dictionaries";
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
  dictionary,
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
  dictionary: Dictionary;
}) {
  const [albumToAdd, setAlbumToAdd] = useState("");
  if (suggestions.length === 0) {
    return <div className="empty-state">{dictionary.workflow.noPendingSuggestions}</div>;
  }
  return (
    <section className="review-workspace">
      <div className="workspace-panel review-list-panel">
        <div className="panel-header">
          <div>
            <h3>{dictionary.workflow.pendingReview}</h3>
            <p>{selectedSuggestionIds.length || suggestions.length} {dictionary.workflow.selected} / {suggestions.length} {dictionary.workflow.pendingReview}</p>
          </div>
        </div>
        <div className="row-actions review-actions">
          <button onClick={onBatchAccept}>{dictionary.workflow.acceptSelected}</button>
          <button onClick={onBatchReject}>{dictionary.workflow.rejectSelected}</button>
        </div>
        <div className="add-album-row">
          <select className="select-control" value={albumToAdd} onChange={(event) => setAlbumToAdd(event.target.value)}>
            <option value="">{dictionary.workflow.addSelectedToAlbum}</option>
            {albums
              .filter((album) => album.kind === "manual")
              .map((album) => (
                <option key={album.id} value={album.id}>{album.name}</option>
              ))}
          </select>
          <button disabled={!albumToAdd} onClick={() => onAddToAlbum(albumToAdd)}>{dictionary.workflow.add}</button>
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
                <strong>{suggestion.title ?? dictionary.workflow.untitledSuggestion}</strong>
                <span>{suggestion.category ?? dictionary.workflow.noCategory}</span>
                <small>{suggestion.tags.join(", ") || dictionary.workflow.noTags}</small>
              </button>
            </div>
          ))}
        </div>
      </div>
      <div className="workspace-panel review-detail-panel">
        {!selectedSuggestion || !form ? (
          <div className="empty-state">{dictionary.workflow.selectSuggestion}</div>
        ) : (
          <>
            <div className="panel-header">
              <div>
                <h3>{dictionary.workflow.reviewMetadata}</h3>
                <p>{dictionary.workflow.status}: {selectedSuggestion.status}</p>
              </div>
              <span className="review-badge">{dictionary.workflow.reviewPending}</span>
            </div>
            <ConfidenceSummary confidence={selectedSuggestion.confidence} dictionary={dictionary} />
            <ReviewTaskMirror
              tasks={tasks}
              suggestionId={selectedSuggestion.id}
              onOpenTask={onOpenTask}
            />
            <div className="review-form">
              <label>
                <span>
                  {dictionary.workflow.titleLabel}
                  <ReviewFieldGenerateButton form={form} field="title" onRegenerateField={onRegenerateField} dictionary={dictionary} />
                </span>
                <input
                  value={form.title}
                  disabled={isReviewFieldGenerating(form, "title")}
                  onChange={(event) => onFormChange({ ...form, title: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="title" dictionary={dictionary} />
              </label>
              <label>
                <span>{dictionary.workflow.category}</span>
                <select
                  className="select-control"
                  value={form.category}
                  onChange={(event) => onFormChange({ ...form, category: event.target.value })}
                >
                  <option value="">{dictionary.workflow.noCategory}</option>
                  {availableCategories.map((category) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
                </select>
              </label>
              <label className="wide-field">
                <span>
                  {dictionary.workflow.descriptionLabel}
                  <ReviewFieldGenerateButton form={form} field="description" onRegenerateField={onRegenerateField} dictionary={dictionary} />
                </span>
                <textarea
                  value={form.description}
                  disabled={isReviewFieldGenerating(form, "description")}
                  onChange={(event) => onFormChange({ ...form, description: event.target.value })}
                />
                <ReviewFieldGenerationStatus form={form} field="description" dictionary={dictionary} />
              </label>
              <label className="wide-field">
                <span>
                  {dictionary.workflow.jsonSchemaPrompt}
                  <ReviewFieldGenerateButton form={form} field="schemaPrompt" onRegenerateField={onRegenerateField} dictionary={dictionary} />
                </span>
                <textarea
                  className="schema-prompt-input"
                  value={form.schemaPrompt}
                  disabled={isReviewFieldGenerating(form, "schemaPrompt")}
                  onChange={(event) => onFormChange({ ...form, schemaPrompt: event.target.value })}
                  spellCheck={false}
                />
                <ReviewFieldGenerationStatus form={form} field="schemaPrompt" dictionary={dictionary} />
              </label>
              <div className="wide-field tag-editor-field">
                <span>{dictionary.workflow.tags}</span>
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
                    placeholder={dictionary.workflow.addTag}
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
              <button onClick={onRestore}>{dictionary.workflow.restore}</button>
              <button disabled={suggestionRegenerating} onClick={onRegenerateSuggestion}>
                {suggestionRegenerating ? dictionary.workflow.regenerating : dictionary.workflow.regenerateSuggestion}
              </button>
              <button className="primary-button" onClick={onAccept}>
                {dictionary.workflow.acceptChanges}
              </button>
            </div>
            <SuggestionHistoryTable history={suggestionHistory} onPickField={onPickHistoryField} dictionary={dictionary} />
          </>
        )}
      </div>
    </section>
  );
}

function ConfidenceSummary({ confidence, dictionary }: { confidence?: ConfidenceScore; dictionary: Dictionary }) {
  if (!confidence) {
    return null;
  }
  const fields = [
    [dictionary.workflow.titleLabel, confidence.title],
    [dictionary.workflow.descriptionLabel, confidence.description],
    [dictionary.workflow.schemaLabel, confidence.schemaPrompt],
    [dictionary.workflow.tags, confidence.tags],
    [dictionary.workflow.category, confidence.category],
  ] as const;
  const knownFields = fields.filter(([, score]) => typeof score === "number");
  if (typeof confidence.overall !== "number" && knownFields.length === 0) {
    return null;
  }
  return (
    <div className="confidence-panel">
      <strong>{dictionary.workflow.confidence} {formatScore(confidence.overall, dictionary)}</strong>
      <div className="tag-list">
        {knownFields.map(([label, score]) => (
          <span key={label}>{label}: {formatScore(score, dictionary)}</span>
        ))}
      </div>
    </div>
  );
}

function SuggestionHistoryTable({
  history,
  onPickField,
  dictionary,
}: {
  history: Suggestion[];
  onPickField: (suggestion: Suggestion, field: ReviewFieldName | "tags" | "category") => void;
  dictionary: Dictionary;
}) {
  if (history.length === 0) {
    return <div className="empty-state compact">{dictionary.workflow.noSuggestionHistory}</div>;
  }
  return (
    <div className="history-table">
      <h3>{dictionary.workflow.suggestionHistory}</h3>
      {history.map((suggestion) => (
        <article key={suggestion.id} className="history-row">
          <div>
            <strong>{suggestion.title ?? dictionary.workflow.untitled}</strong>
            <small>{suggestion.status} · {suggestion.createdAt ? displayDate(suggestion.createdAt) : "-"}</small>
          </div>
          <div className="row-actions">
            <button onClick={() => onPickField(suggestion, "title")}>{dictionary.workflow.titleLabel}</button>
            <button onClick={() => onPickField(suggestion, "description")}>{dictionary.workflow.descriptionLabel}</button>
            <button onClick={() => onPickField(suggestion, "schemaPrompt")}>{dictionary.workflow.schemaLabel}</button>
            <button onClick={() => onPickField(suggestion, "tags")}>{dictionary.workflow.tags}</button>
            <button onClick={() => onPickField(suggestion, "category")}>{dictionary.workflow.category}</button>
          </div>
        </article>
      ))}
    </div>
  );
}

function formatScore(score: number | null | undefined, dictionary: Dictionary): string {
  return typeof score === "number" ? `${score}%` : dictionary.workflow.unknown;
}

function ReviewFieldGenerateButton({
  form,
  field,
  onRegenerateField,
  dictionary,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
  onRegenerateField: (field: ReviewFieldName) => void;
  dictionary: Dictionary;
}) {
  const loading = isReviewFieldGenerating(form, field);
  return (
    <button
      type="button"
      className="inline-action"
      disabled={loading}
      onClick={() => onRegenerateField(field)}
    >
      {loading ? dictionary.workflow.generating : dictionary.workflow.regenerate}
    </button>
  );
}

function ReviewFieldGenerationStatus({
  form,
  field,
  dictionary,
}: {
  form: ReviewFormState;
  field: ReviewFieldName;
  dictionary: Dictionary;
}) {
  const state = form.generation[field];
  if (state.error) {
    return <small className="field-status error-text">{dictionary.workflow.generationFailedHint}</small>;
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
