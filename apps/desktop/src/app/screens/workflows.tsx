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
} from "../../workbench-state";
import { Icon } from "../../studio-icons";
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
} from "../../studio-orchestration";
import { convertImagePath, errorMessage, pickImageFile } from "../tauri-adapter";
import { Thumbnail } from "./gallery/GalleryWorkspace";
import { StarRatingControl, StarRatingDisplay } from "../components/rating";
import {
  descriptionFromPrompt,
  previewGeneratedReviewField,
  schemaPromptFromAsset,
  thumbnailAspectRatio,
  titleFromPrompt,
} from "../utils";
import { createTaskDraft } from "../mock-data";
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
} from "../types";

export function StudioOverviewBand({
  assetCount,
  reviewCount,
  queueCount,
  integrityIssueCount,
  activeView,
}: {
  assetCount: number;
  reviewCount: number;
  queueCount: number;
  integrityIssueCount: number;
  activeView: View;
}) {
  const cards = [
    { label: "Assets", value: assetCount, tone: activeView === "gallery" ? "active" : "" },
    { label: "Pending Review", value: reviewCount, tone: reviewCount > 0 ? "warning" : "" },
    { label: "Active Tasks", value: queueCount, tone: queueCount > 0 ? "active" : "" },
    { label: "Integrity Issues", value: integrityIssueCount, tone: integrityIssueCount > 0 ? "danger" : "healthy" },
  ];
  return (
    <section className="studio-overview-band" aria-label="Studio overview">
      {cards.map((card) => (
        <div className={`overview-metric ${card.tone}`} key={card.label}>
          <span>{card.label}</span>
          <strong>{card.value}</strong>
        </div>
      ))}
    </section>
  );
}

export function WorkspaceToolbar({
  activeView,
  query,
  itemCount,
  status,
  composerOpen,
  onComposerOpenChange,
  onQueryChange,
}: {
  activeView: View;
  query: GalleryQueryState;
  itemCount: number;
  status: string;
  composerOpen: boolean;
  onComposerOpenChange: (open: boolean) => void;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const viewLabels: Record<View, { title: string; eyebrow: string }> = {
    gallery: { title: "Gallery", eyebrow: "Image assets" },
    albums: { title: "Albums", eyebrow: "Curation sets" },
    review: { title: "Review Inbox", eyebrow: "Metadata suggestions" },
    queue: { title: "Tasks Queue", eyebrow: "Generation operations" },
    settings: { title: "Settings", eyebrow: "Library administration" },
  };
  const label = viewLabels[activeView];
  const showGalleryControls = activeView === "gallery";

  return (
    <header className="workspace-toolbar">
      <div className="workspace-title-row">
        <div className="workspace-title">
          <span>{label.eyebrow}</span>
          <h1>{label.title}</h1>
        </div>
        <span className="toolbar-status">{status}</span>
        <button className="primary-button" onClick={() => onComposerOpenChange(!composerOpen)}>
          <Icon name="plus" />
          <span>Generate</span>
        </button>
      </div>

      {showGalleryControls && (
        <>
          <div className="search-row">
            <label className="search-box">
              <Icon name="search" />
              <span>Search</span>
              <input
                value={query.text}
                onChange={(event) => onQueryChange(updateGalleryQuery(query, { text: event.target.value }))}
                placeholder="Search prompts, titles, tags, albums..."
              />
            </label>
            <button className="icon-button" aria-label="Grid view">
              <Icon name="grid" />
            </button>
            <button className="icon-button" aria-label="List view">
              <Icon name="list" />
            </button>
          </div>
          <div className="filter-row">
            <SegmentedButton
              label="Provider"
              active={query.providers.length > 0}
              onClick={() => onQueryChange(toggleGalleryProvider(query, "fake"))}
            />
            <select
              className="select-control"
              value={query.minRating ?? ""}
              onChange={(event) =>
                onQueryChange(
                  updateGalleryQuery(query, {
                    minRating: event.target.value ? Number(event.target.value) : null,
                  }),
                )
              }
            >
              <option value="">Rating</option>
              <option value="5">5 stars</option>
              <option value="4">4+ stars</option>
              <option value="3">3+ stars</option>
            </select>
            <select
              className="select-control"
              value={query.reviewStatus}
              onChange={(event) =>
                onQueryChange(updateGalleryQuery(query, { reviewStatus: event.target.value as ReviewStatusFilter }))
              }
            >
              <option value="any">Review</option>
              <option value="pending">Review Pending</option>
            </select>
            <button onClick={() => onQueryChange(resetGalleryQuery())}>Clear All</button>
            <select
              className="select-control sort-select"
              value={query.sort}
              onChange={(event) => onQueryChange(updateGalleryQuery(query, { sort: event.target.value as GallerySort }))}
            >
              <option value="newest">Sort: Newest</option>
              <option value="oldest">Sort: Oldest</option>
              <option value="ratingDesc">Sort: Rating</option>
              <option value="titleAsc">Sort: Title</option>
              <option value="providerAsc">Sort: Provider</option>
            </select>
            <strong>{itemCount} items</strong>
          </div>
        </>
      )}
    </header>
  );
}

function SegmentedButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button className={active ? "chip-button active" : "chip-button"} onClick={onClick}>
      {label}
    </button>
  );
}

export function GenerationComposer({
  prompt,
  provider,
  inputSourceName,
  submitting,
  onPromptChange,
  onProviderChange,
  onGenerate,
}: {
  prompt: string;
  provider: string;
  inputSourceName: string | null;
  submitting: boolean;
  onPromptChange: (value: string) => void;
  onProviderChange: (value: string) => void;
  onGenerate: () => void;
}) {
  const hasInputSource = inputSourceName !== null;

  return (
    <section className="composer">
      <select className="select-control" value={provider} onChange={(event) => onProviderChange(event.target.value)}>
        <option value="codex-cli">codex-cli</option>
        <option value="fake">fake</option>
      </select>
      <input
        value={prompt}
        onChange={(event) => onPromptChange(event.target.value)}
        placeholder={hasInputSource ? `Prompt for image-to-image from ${inputSourceName}` : "Prompt"}
      />
      <button disabled={submitting || prompt.trim().length === 0} onClick={onGenerate}>
        {submitting ? "Enqueueing..." : hasInputSource ? "Generate image" : "Run"}
      </button>
    </section>
  );
}

export function AlbumsWorkspace({
  albums,
  availableTags,
  availableCategories,
  availableProviders,
  selectedAlbumId,
  gallery,
  loading,
  searchValue,
  onSearchChange,
  newAlbumName,
  onNewAlbumNameChange,
  createOpen,
  onCreateOpenChange,
  onCreateAlbum,
  onCreateSmartAlbum,
  onOpenAlbum,
  onCloseAlbum,
  onRenameAlbum,
  onDeleteAlbum,
  onReorderAlbums,
  onRemoveAsset,
  onReorderAssets,
  selectedGalleryAssetCount,
  onBatchAddSelected,
  onSelectAsset,
}: {
  albums: AlbumListItem[];
  availableTags: string[];
  availableCategories: string[];
  availableProviders: string[];
  selectedAlbumId: string | null;
  gallery: GalleryAsset[];
  loading: boolean;
  searchValue: string;
  onSearchChange: (value: string) => void;
  newAlbumName: string;
  onNewAlbumNameChange: (value: string) => void;
  createOpen: boolean;
  onCreateOpenChange: (open: boolean) => void;
  onCreateAlbum: () => void;
  onCreateSmartAlbum: (name: string, queryJson: string) => void;
  onOpenAlbum: (albumId: string) => void;
  onCloseAlbum: () => void;
  onRenameAlbum: (albumId: string, name: string) => void;
  onDeleteAlbum: (albumId: string) => void;
  onReorderAlbums: (albumIds: string[]) => void;
  onRemoveAsset: (assetId: string) => void;
  onReorderAssets: (assetIds: string[]) => void;
  selectedGalleryAssetCount: number;
  onBatchAddSelected: (albumId: string) => void;
  onSelectAsset: (assetId: string) => void;
}) {
  const [newAlbumKind, setNewAlbumKind] = useState<"manual" | "smart">("manual");
  const [smartText, setSmartText] = useState("");
  const [smartTags, setSmartTags] = useState("");
  const [smartProviders, setSmartProviders] = useState("");
  const [smartMinRating, setSmartMinRating] = useState("");
  const [smartReviewPending, setSmartReviewPending] = useState(false);
  const [smartCategory, setSmartCategory] = useState("");
  const [smartStatus, setSmartStatus] = useState("");
  const [smartCreatedAtFrom, setSmartCreatedAtFrom] = useState("");
  const [smartCreatedAtTo, setSmartCreatedAtTo] = useState("");
  const [smartSort, setSmartSort] = useState<GallerySort>("newest");
  const selectedAlbum = albums.find((album) => album.id === selectedAlbumId) ?? null;
  const searchNeedle = searchValue.trim().toLocaleLowerCase();
  const visibleAlbums =
    searchNeedle.length === 0
      ? albums
      : albums.filter((album) => album.name.toLocaleLowerCase().includes(searchNeedle));
  const smartPreviewCount = previewSmartAlbumCount(gallery, {
    text: smartText,
    tags: splitCsv(smartTags),
    providers: splitCsv(smartProviders),
    minRating: smartMinRating.trim() ? Number(smartMinRating.trim()) : null,
    reviewPending: smartReviewPending,
    category: smartCategory,
    status: smartStatus,
    createdAtFrom: smartCreatedAtFrom,
    createdAtTo: smartCreatedAtTo,
  });
  return (
    <section className="albums-workspace">
      <div className="workspace-panel album-list-panel">
        <div className="panel-header">
          <div>
            <h3>Albums</h3>
            <p>{loading ? "Loading albums..." : `${albums.length} album${albums.length === 1 ? "" : "s"}`}</p>
          </div>
        </div>
        <div className="album-search-row">
          <input
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder="Search albums"
          />
          <button className="icon-button" aria-label="Create album" onClick={() => onCreateOpenChange(!createOpen)}>
            <Icon name="plus" />
          </button>
          {createOpen && (
            <div className="album-create-popover">
              <label>
                <span>Album name</span>
                <input
                  value={newAlbumName}
                  autoFocus
                  onChange={(event) => onNewAlbumNameChange(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      onCreateAlbum();
                    }
                    if (event.key === "Escape") {
                      onCreateOpenChange(false);
                    }
                  }}
                  placeholder="New manual album"
                />
              </label>
              <label>
                <span>Kind</span>
                <select className="select-control" value={newAlbumKind} onChange={(event) => setNewAlbumKind(event.target.value as "manual" | "smart")}>
                  <option value="manual">Manual</option>
                  <option value="smart">Smart</option>
                </select>
              </label>
              {newAlbumKind === "smart" && (
                <div className="smart-builder">
                  <label>
                    <span>Text</span>
                    <input value={smartText} onChange={(event) => setSmartText(event.target.value)} />
                  </label>
                  <label>
                    <span>Tags</span>
                    <input list="smart-tag-options" value={smartTags} onChange={(event) => setSmartTags(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-tag-options">
                    {availableTags.map((tag) => <option key={tag} value={tag} />)}
                  </datalist>
                  <label>
                    <span>Providers</span>
                    <input list="smart-provider-options" value={smartProviders} onChange={(event) => setSmartProviders(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-provider-options">
                    {availableProviders.map((provider) => <option key={provider} value={provider} />)}
                  </datalist>
                  <label>
                    <span>Min rating</span>
                    <input value={smartMinRating} onChange={(event) => setSmartMinRating(event.target.value)} inputMode="numeric" />
                  </label>
                  <label>
                    <span>Category</span>
                    <select className="select-control" value={smartCategory} onChange={(event) => setSmartCategory(event.target.value)}>
                      <option value="">Any</option>
                      {availableCategories.map((category) => <option key={category} value={category}>{category}</option>)}
                    </select>
                  </label>
                  <label>
                    <span>Status</span>
                    <input value={smartStatus} onChange={(event) => setSmartStatus(event.target.value)} placeholder="generated, curated..." />
                  </label>
                  <label>
                    <span>Created from</span>
                    <input value={smartCreatedAtFrom} onChange={(event) => setSmartCreatedAtFrom(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>Created to</span>
                    <input value={smartCreatedAtTo} onChange={(event) => setSmartCreatedAtTo(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>Sort</span>
                    <select className="select-control" value={smartSort} onChange={(event) => setSmartSort(event.target.value as GallerySort)}>
                      <option value="newest">Newest</option>
                      <option value="oldest">Oldest</option>
                      <option value="ratingDesc">Rating</option>
                      <option value="titleAsc">Title</option>
                      <option value="providerAsc">Provider</option>
                    </select>
                  </label>
                  <label className="checkbox-row">
                    <input type="checkbox" checked={smartReviewPending} onChange={(event) => setSmartReviewPending(event.target.checked)} />
                    <span>Review pending</span>
                  </label>
                  <small>{smartPreviewCount} visible assets match this builder.</small>
                </div>
              )}
              <div className="row-actions">
                <button onClick={() => onCreateOpenChange(false)}>Cancel</button>
                <button
                  className="primary-button"
                  disabled={newAlbumName.trim().length === 0}
                  onClick={() => {
                    if (newAlbumKind === "manual") {
                      onCreateAlbum();
                      return;
                    }
                    const query = {
                      ...(smartText.trim() ? { text: smartText.trim() } : {}),
                      ...(smartTags.trim() ? { tags: splitCsv(smartTags) } : {}),
                      ...(smartProviders.trim() ? { providers: splitCsv(smartProviders) } : {}),
                      ...(smartMinRating.trim() ? { minRating: Number(smartMinRating.trim()) } : {}),
                      ...(smartReviewPending ? { reviewStatus: "pending" } : {}),
                      ...(smartCategory ? { category: smartCategory } : {}),
                      ...(smartStatus.trim() ? { status: smartStatus.trim() } : {}),
                      ...(smartCreatedAtFrom.trim() ? { createdAtFrom: smartCreatedAtFrom.trim() } : {}),
                      ...(smartCreatedAtTo.trim() ? { createdAtTo: smartCreatedAtTo.trim() } : {}),
                      sort: smartSort,
                    };
                    onCreateSmartAlbum(newAlbumName, JSON.stringify(query));
                  }}
                >
                  Create
                </button>
              </div>
            </div>
          )}
        </div>
        {albums.length === 0 ? (
          <div className="empty-state compact">No albums yet.</div>
        ) : visibleAlbums.length === 0 ? (
          <div className="empty-state compact">No albums match the search.</div>
        ) : (
          <div className="album-list">
            {visibleAlbums.map((album) => (
              <button
                key={album.id}
                className={album.id === selectedAlbumId ? "album-list-item selected" : "album-list-item"}
                draggable
                onDragStart={(event) => event.dataTransfer.setData("text/album-id", album.id)}
                onDragOver={(event) => event.preventDefault()}
                onDrop={(event) => {
                  event.preventDefault();
                  const draggedId = event.dataTransfer.getData("text/album-id");
                  if (!draggedId || draggedId === album.id) {
                    return;
                  }
                  const fromIndex = albums.findIndex((item) => item.id === draggedId);
                  const toIndex = albums.findIndex((item) => item.id === album.id);
                  const next = moveItem(albums, fromIndex, toIndex);
                  if (next !== albums) {
                    onReorderAlbums(next.map((item) => item.id));
                  }
                }}
                onClick={() => onOpenAlbum(album.id)}
              >
                <span>
                  <strong>{album.name}</strong>
                  <small>{album.kind}</small>
                </span>
                <strong>{album.itemCount ?? "-"}</strong>
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="workspace-panel album-detail-panel">
        <div className="panel-header">
          <div>
            <h3>{selectedAlbum?.name ?? "Select an album"}</h3>
            <p>
              {selectedAlbum
                ? `${selectedAlbum.kind} album · ${gallery.length} visible item${gallery.length === 1 ? "" : "s"}`
                : "Open an album to view its assets."}
            </p>
          </div>
          {selectedAlbum && (
            <div className="row-actions">
              {selectedAlbum.kind === "manual" && (
                <button disabled={selectedGalleryAssetCount === 0} onClick={() => onBatchAddSelected(selectedAlbum.id)}>
                  Add selected ({selectedGalleryAssetCount})
                </button>
              )}
              <button onClick={() => {
                const next = window.prompt("Album name", selectedAlbum.name);
                if (next) {
                  onRenameAlbum(selectedAlbum.id, next);
                }
              }}>
                Rename
              </button>
              <button onClick={() => onDeleteAlbum(selectedAlbum.id)}>Delete</button>
              <button onClick={onCloseAlbum}>All assets</button>
            </div>
          )}
        </div>
        {!selectedAlbum ? (
          <div className="empty-state">Choose an album from the list.</div>
        ) : gallery.length === 0 ? (
          <div className="empty-state">This album is empty.</div>
        ) : (
          <section className="gallery-grid compact-grid">
            {gallery.map((asset, index) => (
              <button
                key={asset.id}
                className="asset-card"
                draggable={selectedAlbum.kind === "manual"}
                onDragStart={(event) => event.dataTransfer.setData("text/asset-id", asset.id)}
                onDragOver={(event) => {
                  if (selectedAlbum.kind === "manual") {
                    event.preventDefault();
                  }
                }}
                onDrop={(event) => {
                  if (selectedAlbum.kind !== "manual") {
                    return;
                  }
                  event.preventDefault();
                  const draggedId = event.dataTransfer.getData("text/asset-id");
                  if (!draggedId || draggedId === asset.id) {
                    return;
                  }
                  const fromIndex = gallery.findIndex((item) => item.id === draggedId);
                  const toIndex = gallery.findIndex((item) => item.id === asset.id);
                  const next = moveItem(gallery, fromIndex, toIndex);
                  onReorderAssets(next.map((item) => item.id));
                }}
                onClick={() => onSelectAsset(asset.id)}
              >
                <Thumbnail asset={asset} index={index} />
                <span className="asset-title">{asset.title ?? "Untitled"}</span>
                <span className="provider-pill">{asset.provider ?? "Unknown provider"}</span>
                <StarRatingDisplay rating={asset.rating} />
                {asset.reviewPendingCount > 0 && <span className="review-badge">Review pending</span>}
                {selectedAlbum.kind === "manual" && (
                  <span
                    className="text-button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onRemoveAsset(asset.id);
                    }}
                  >
                    Remove
                  </span>
                )}
              </button>
            ))}
          </section>
        )}
      </div>
    </section>
  );
}

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

function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function previewSmartAlbumCount(
  assets: GalleryAsset[],
  query: {
    text: string;
    tags: string[];
    providers: string[];
    minRating: number | null;
    reviewPending: boolean;
    category: string;
    status: string;
    createdAtFrom: string;
    createdAtTo: string;
  },
): number {
  const text = query.text.trim().toLocaleLowerCase();
  return assets.filter((asset) => {
    if (text && ![asset.title, asset.category, asset.status, asset.provider, asset.modelLabel, asset.prompt, ...asset.tags].some((value) => value?.toLocaleLowerCase().includes(text))) {
      return false;
    }
    if (query.tags.length > 0 && !query.tags.every((tag) => asset.tags.includes(tag))) {
      return false;
    }
    if (query.providers.length > 0 && (!asset.provider || !query.providers.includes(asset.provider))) {
      return false;
    }
    if (query.minRating !== null && (asset.rating ?? 0) < query.minRating) {
      return false;
    }
    if (query.reviewPending && asset.reviewPendingCount === 0) {
      return false;
    }
    if (query.category && asset.category !== query.category) {
      return false;
    }
    if (query.status.trim() && asset.status !== query.status.trim()) {
      return false;
    }
    if (query.createdAtFrom.trim() && asset.createdAt < query.createdAtFrom.trim()) {
      return false;
    }
    if (query.createdAtTo.trim() && asset.createdAt > query.createdAtTo.trim()) {
      return false;
    }
    return true;
  }).length;
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
}) {
  const selectedTask = tasks.find((task) => task.id === selectedTaskId) ?? null;
  return (
    <section className={`task-workspace active-${activePanel}`}>
      <div className="task-panel-tabs" role="tablist" aria-label="Queue panels">
        <button className={activePanel === "compose" ? "active" : ""} onClick={() => onActivePanelChange("compose")}>
          Compose
        </button>
        <button className={activePanel === "queue" ? "active" : ""} onClick={() => onActivePanelChange("queue")}>
          Queue
        </button>
        <button className={activePanel === "detail" ? "active" : ""} onClick={() => onActivePanelChange("detail")}>
          Detail
        </button>
      </div>
      <div className={activePanel === "compose" ? "task-panel-slot task-panel-compose active" : "task-panel-slot task-panel-compose"}>
        <BatchComposer
          drafts={drafts}
          onDraftsChange={onDraftsChange}
          onAddDraft={onAddDraft}
          onEnqueue={onEnqueue}
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
}: {
  drafts: TaskDraft[];
  onDraftsChange: (drafts: TaskDraft[]) => void;
  onAddDraft: () => void;
  onEnqueue: () => void;
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
    const selected = await pickImageFile("Choose Reference Image", draft.inputFile);
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
      setImportError("No valid tasks found in JSON.");
    }
  }
  return (
    <section className="task-panel batch-composer">
      <div className="panel-header">
        <div>
          <h3>Batch Composer</h3>
          <p>{drafts.length} draft{drafts.length === 1 ? "" : "s"}</p>
        </div>
        <button onClick={onAddDraft}>Add task</button>
      </div>
      {drafts.map((draft, index) => (
        <article className="task-draft-card" key={draft.id}>
          <div className="task-draft-header">
            <strong>Task {index + 1}</strong>
            <div className="row-actions">
              <button onClick={() => duplicateDraft(draft)}>Duplicate</button>
              <button onClick={() => removeDraft(draft.id)}>Remove</button>
            </div>
          </div>
          <label>
            <span>Prompt</span>
            <textarea value={draft.prompt} onChange={(event) => updateDraft(draft.id, { prompt: event.target.value })} />
          </label>
          <div className="task-draft-grid">
            <label>
              <span>Provider</span>
              <select className="select-control" value={draft.provider} onChange={(event) => updateDraft(draft.id, { provider: event.target.value })}>
                <option value="codex-cli">codex-cli</option>
                <option value="fake">fake</option>
              </select>
            </label>
            <label>
              <span>Operation</span>
              <select className="select-control" value={draft.operation} onChange={(event) => updateDraft(draft.id, { operation: event.target.value as TaskDraft["operation"] })}>
                <option value="text_to_image">text to image</option>
                <option value="image_to_image">image to image</option>
              </select>
            </label>
            <label>
              <span>Priority</span>
              <input type="number" value={draft.priority} onChange={(event) => updateDraft(draft.id, { priority: Number(event.target.value) })} />
            </label>
            <label>
              <span>Max attempts</span>
              <input type="number" min={1} max={10} value={draft.maxAttempts} onChange={(event) => updateDraft(draft.id, { maxAttempts: Number(event.target.value) })} />
            </label>
            <label>
              <span>Reference file</span>
              <div className="reference-file-control">
                <input value={draft.inputFile} onChange={(event) => updateDraft(draft.id, { inputFile: event.target.value, operation: event.target.value.trim() ? "image_to_image" : draft.operation })} />
                <div className="reference-file-actions">
                  <button type="button" onClick={() => chooseReferenceFile(draft)}>Choose image</button>
                  {draft.inputFile.trim() && <button type="button" onClick={() => clearReferenceFile(draft)}>Clear</button>}
                </div>
              </div>
            </label>
          </div>
          <label>
            <span>Parameters JSON</span>
            <textarea value={draft.parametersJson} onChange={(event) => updateDraft(draft.id, { parametersJson: event.target.value })} />
          </label>
        </article>
      ))}
      <div className="import-json-box">
        <textarea value={importJson} onChange={(event) => setImportJson(event.target.value)} placeholder='[{"prompt":"multi-line prompt","provider":"fake"}]' />
        <button disabled={importJson.trim().length === 0} onClick={importDrafts}>Import JSON</button>
        {importError && <span className="inline-error">{importError}</span>}
      </div>
      <button className="primary-button" disabled={drafts.every((draft) => draft.prompt.trim().length === 0)} onClick={onEnqueue}>
        Enqueue all
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
}) {
  const queuedIds = [...tasks].filter((task) => task.status === "queued").sort(compareTaskOrder).map((task) => task.id);
  const orderedTasks = [...tasks].sort(compareTaskNewestFirst);
  return (
    <section className="task-panel tasks-queue-panel">
      <div className="panel-header">
        <div>
          <h3>Tasks Queue</h3>
          <p>{daemonOnline ? "Daemon connected" : "Daemon offline"}{loading ? " · Refreshing" : ""}</p>
        </div>
        <button onClick={onRefresh}>Refresh</button>
      </div>
      <div className="task-list">
        {tasks.length === 0 ? (
          <div className="empty-state">No tasks yet.</div>
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
                    <button disabled={queuedIds.indexOf(task.id) <= 0} onClick={() => onMoveTask(task.id, -1)}>Up</button>
                    <button disabled={queuedIds.indexOf(task.id) === queuedIds.length - 1} onClick={() => onMoveTask(task.id, 1)}>Down</button>
                  </>
                )}
                {["queued", "running", "retry_waiting", "cancel_requested"].includes(task.status) && (
                  <button disabled={pendingTaskActions.includes(taskActionKey("cancel_daemon_task", task.id))} onClick={() => onCancel(task.id)}>Cancel</button>
                )}
                {isRetryableTaskStatus(task.status) && (
                  <button disabled={pendingTaskActions.includes(taskActionKey("retry_daemon_task", task.id))} onClick={() => onRetry(task.id)}>Retry</button>
                )}
                <button disabled={pendingTaskActions.includes(taskActionKey("duplicate_daemon_task", task.id))} onClick={() => onDuplicate(task.id)}>Duplicate</button>
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
}: {
  task: DaemonTask | null;
  detail: DaemonTaskDetail | null;
  pendingTaskActions: string[];
  onCancel: (taskId: string) => void;
  onRetry: (taskId: string) => void;
  onDuplicate: (taskId: string) => void;
}) {
  if (!task) {
    return <section className="task-panel task-detail-panel empty-state">Select a task to inspect.</section>;
  }
  return (
    <section className="task-panel task-detail-panel">
      <div className="panel-header">
        <div>
          <h3>Task Detail</h3>
          <p>{task.id}</p>
        </div>
        <span className={`status ${task.status}`}>{statusLabel(task.status)}</span>
      </div>
      <div className="row-actions">
        {["queued", "running", "retry_waiting", "cancel_requested"].includes(task.status) && (
          <button disabled={pendingTaskActions.includes(taskActionKey("cancel_daemon_task", task.id))} onClick={() => onCancel(task.id)}>Cancel</button>
        )}
        {isRetryableTaskStatus(task.status) && (
          <button disabled={pendingTaskActions.includes(taskActionKey("retry_daemon_task", task.id))} onClick={() => onRetry(task.id)}>Retry</button>
        )}
        <button disabled={pendingTaskActions.includes(taskActionKey("duplicate_daemon_task", task.id))} onClick={() => onDuplicate(task.id)}>Duplicate</button>
      </div>
      <section className="detail-section">
        <h4>Input Snapshot</h4>
        <pre>{JSON.stringify(task.input ?? {}, null, 2)}</pre>
      </section>
      {task.lastErrorMessage && (
        <section className="detail-section">
          <h4>Error</h4>
          <p className="error-text">{task.lastErrorCode ?? "TaskFailed"}: {task.lastErrorMessage}</p>
        </section>
      )}
      <section className="detail-section">
        <h4>Attempts</h4>
        {(detail?.attempts ?? []).length === 0 ? <p>No attempts yet.</p> : detail?.attempts.map((attempt) => (
          <div className="detail-row" key={attempt.id}>
            <strong>#{attempt.attemptNumber} {attempt.status}</strong>
            <span>{attempt.errorMessage ?? attempt.logPath ?? displayDate(attempt.startedAt)}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>Timeline</h4>
        {(detail?.events ?? []).map((event) => (
          <div className="detail-row" key={event.id}>
            <strong>{event.eventType}</strong>
            <span>{event.message ? `${displayDate(event.createdAt)} · ${event.message}` : displayDate(event.createdAt)}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>Outputs</h4>
        {(detail?.outputs ?? []).length === 0 ? <p>No outputs yet.</p> : detail?.outputs.map((output) => (
          <div className="detail-row" key={output.id}>
            <strong>{output.outputType}</strong>
            <span>{output.targetId}</span>
          </div>
        ))}
      </section>
      <section className="detail-section">
        <h4>Log Tail</h4>
        <pre>{detail?.logTail || "No log content yet."}</pre>
      </section>
    </section>
  );
}

export function SettingsWorkspace({
  library,
  libraries,
  activeSection,
  providerHealth,
  daemonOnline,
  libraryStatus,
  onSectionChange,
  libraryFolderName,
  libraryName,
  onLibraryFolderNameChange,
  onLibraryNameChange,
  onCreate,
  onOpenExisting,
  onImportZip,
  onSwitchLibrary,
  onRenameLibrary,
  onCloseLibrary,
  onExportZip,
  onReveal,
  pendingLibraryActions,
  missingLibraryPaths,
  logs,
  logsLoading,
  selectedLogPath,
  selectedLogContent,
  logContentLoading,
  updateState,
  onRefreshLogs,
  onSelectLog,
  onCheckUpdate,
  onInstallUpdate,
  onRestartApp,
}: {
  library: Library | null;
  libraries: Library[];
  activeSection: SettingsSection;
  providerHealth: ProviderHealth[];
  daemonOnline: boolean;
  libraryStatus: LibraryStatus | null;
  onSectionChange: (section: SettingsSection) => void;
  libraryFolderName: string;
  libraryName: string;
  onLibraryFolderNameChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpenExisting: () => void;
  onImportZip: () => void;
  onSwitchLibrary: (libraryId: string) => void;
  onRenameLibrary: (library: Library) => void;
  onCloseLibrary: (library: Library) => void;
  onExportZip: (library: Library) => void;
  onReveal: (library: Library) => void;
  pendingLibraryActions: string[];
  missingLibraryPaths: string[];
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  updateState: UpdateState;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
}) {
  return (
    <section className="settings-workspace">
      <div className="settings-tabs" role="tablist" aria-label="Settings sections">
        <button className={activeSection === "libraries" ? "active" : ""} onClick={() => onSectionChange("libraries")}>
          Libraries
        </button>
        <button className={activeSection === "providers" ? "active" : ""} onClick={() => onSectionChange("providers")}>
          Providers
        </button>
        <button className={activeSection === "updates" ? "active" : ""} onClick={() => onSectionChange("updates")}>
          Updates
        </button>
        <button className={activeSection === "logs" ? "active" : ""} onClick={() => onSectionChange("logs")}>
          Logs
        </button>
      </div>
      {activeSection === "libraries" ? (
        <SettingsLibrariesView
          library={library}
          libraries={libraries}
          libraryFolderName={libraryFolderName}
          libraryName={libraryName}
          onLibraryFolderNameChange={onLibraryFolderNameChange}
          onLibraryNameChange={onLibraryNameChange}
          onCreate={onCreate}
          onOpenExisting={onOpenExisting}
          onImportZip={onImportZip}
          onSwitchLibrary={onSwitchLibrary}
          onRenameLibrary={onRenameLibrary}
          onCloseLibrary={onCloseLibrary}
          onExportZip={onExportZip}
          onReveal={onReveal}
          pendingLibraryActions={pendingLibraryActions}
          missingLibraryPaths={missingLibraryPaths}
        />
      ) : activeSection === "providers" ? (
        <SettingsProvidersView
          providerHealth={providerHealth}
          daemonOnline={daemonOnline}
          libraryStatus={libraryStatus}
        />
      ) : activeSection === "updates" ? (
        <SettingsUpdatesView
          updateState={updateState}
          onCheckUpdate={onCheckUpdate}
          onInstallUpdate={onInstallUpdate}
          onRestartApp={onRestartApp}
        />
      ) : (
        <SettingsLogsView
          logs={logs}
          logsLoading={logsLoading}
          selectedLogPath={selectedLogPath}
          selectedLogContent={selectedLogContent}
          logContentLoading={logContentLoading}
          onRefreshLogs={onRefreshLogs}
          onSelectLog={onSelectLog}
        />
      )}
    </section>
  );
}

function SettingsLibrariesView({
  library,
  libraries,
  libraryFolderName,
  libraryName,
  onLibraryFolderNameChange,
  onLibraryNameChange,
  onCreate,
  onOpenExisting,
  onImportZip,
  onSwitchLibrary,
  onRenameLibrary,
  onCloseLibrary,
  onExportZip,
  onReveal,
  pendingLibraryActions,
  missingLibraryPaths,
}: {
  library: Library | null;
  libraries: Library[];
  libraryFolderName: string;
  libraryName: string;
  onLibraryFolderNameChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpenExisting: () => void;
  onImportZip: () => void;
  onSwitchLibrary: (libraryId: string) => void;
  onRenameLibrary: (library: Library) => void;
  onCloseLibrary: (library: Library) => void;
  onExportZip: (library: Library) => void;
  onReveal: (library: Library) => void;
  pendingLibraryActions: string[];
  missingLibraryPaths: string[];
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>Libraries</h3>
          <p>
            {libraries.length} registered{library ? `, current: ${library.name}` : ""}
          </p>
        </div>
        <div className="row-actions">
          <button onClick={onOpenExisting}>Open Existing Library</button>
          <button onClick={onImportZip} disabled={pendingLibraryActions.includes("import")}>
            {pendingLibraryActions.includes("import") ? "Importing..." : "Import Zip"}
          </button>
        </div>
      </div>
      <div className="library-create-strip">
        <div className="library-section-heading">
          <h4>Create Library</h4>
          <p>Choose a parent folder after clicking Create. The library folder will be created inside it.</p>
        </div>
        <div className="library-create-controls">
          <label>
            <span>Library name</span>
            <input value={libraryName} onChange={(event) => onLibraryNameChange(event.target.value)} />
          </label>
          <label>
            <span>Folder name</span>
            <input value={libraryFolderName} onChange={(event) => onLibraryFolderNameChange(event.target.value)} />
          </label>
          <button onClick={onCreate}>Create...</button>
        </div>
      </div>
      <div className="library-section-heading library-list-heading">
        <h4>Registered Libraries</h4>
        <p>Switch, rename, close, export, or reveal libraries registered on this machine.</p>
      </div>
      {libraries.length === 0 ? (
        <div className="empty-state compact">No library registered.</div>
      ) : (
        <div className="library-table" role="table" aria-label="Registered libraries">
          <div className="library-table-row header" role="row">
            <span>Name</span>
            <span>Path</span>
            <span>Actions</span>
          </div>
          {libraries.map((item) => {
            const actions = libraryMaintenanceActions(item.rootPath, missingLibraryPaths);
            const isCurrent = library?.id === item.id;
            const busy = (name: string) => pendingLibraryActions.includes(`${name}:${item.id}`);
            return (
              <div
                key={item.id}
                className={isCurrent ? "library-table-row current" : "library-table-row"}
                role="row"
                tabIndex={isCurrent ? -1 : 0}
                aria-label={isCurrent ? `${item.name}, current library` : `Switch to ${item.name}`}
                onClick={() => {
                  if (!isCurrent) {
                    onSwitchLibrary(item.id);
                  }
                }}
                onKeyDown={(event) => {
                  if (!isCurrent && (event.key === "Enter" || event.key === " ")) {
                    event.preventDefault();
                    onSwitchLibrary(item.id);
                  }
                }}
              >
                <span className="library-row-main">
                  <strong>{item.name}</strong>
                  {isCurrent && <small>Current</small>}
                  {!actions.canReveal && <small>Missing on disk</small>}
                </span>
                <span className="mono-line" title={item.rootPath}>
                  {item.rootPath}
                </span>
                <span className="row-actions library-row-actions">
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Rename library"
                    data-tooltip={busy("rename") ? "Renaming..." : "Rename"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onRenameLibrary(item);
                    }}
                    disabled={busy("rename")}
                  >
                    <LibraryActionIcon kind={busy("rename") ? "loading" : "rename"} />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Export library zip"
                    data-tooltip={busy("export") ? "Exporting..." : "Export Zip"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onExportZip(item);
                    }}
                    disabled={!actions.canExport || busy("export")}
                  >
                    <LibraryActionIcon kind={busy("export") ? "loading" : "export"} />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Reveal library in Finder"
                    data-tooltip="Reveal in Finder"
                    onClick={(event) => {
                      event.stopPropagation();
                      onReveal(item);
                    }}
                    disabled={!actions.canReveal || busy("reveal")}
                  >
                    <LibraryActionIcon kind="reveal" />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Close library"
                    data-tooltip={busy("close") ? "Closing..." : "Close"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onCloseLibrary(item);
                    }}
                    disabled={!actions.canClose || busy("close")}
                  >
                    <LibraryActionIcon kind={busy("close") ? "loading" : "close"} />
                  </button>
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function LibraryActionIcon({ kind }: { kind: "rename" | "export" | "reveal" | "close" | "loading" }) {
  if (kind === "loading") {
    return (
      <svg className="button-icon spinner-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 3a9 9 0 1 1-8.2 5.3" />
      </svg>
    );
  }

  return (
    <svg className="button-icon" viewBox="0 0 24 24" aria-hidden="true">
      {kind === "rename" && (
        <>
          <path d="M4 20h4l11-11a2.8 2.8 0 0 0-4-4L4 16v4Z" />
          <path d="m13.5 6.5 4 4" />
        </>
      )}
      {kind === "export" && (
        <>
          <path d="M12 3v11" />
          <path d="m8 10 4 4 4-4" />
          <path d="M5 17v3h14v-3" />
        </>
      )}
      {kind === "reveal" && (
        <>
          <path d="M3 7h7l2 2h9v10H3V7Z" />
          <path d="M15 13h4" />
          <path d="m17 11 2 2-2 2" />
        </>
      )}
      {kind === "close" && (
        <>
          <path d="M6 6l12 12" />
          <path d="M18 6 6 18" />
        </>
      )}
    </svg>
  );
}

function SettingsProvidersView({
  providerHealth,
  daemonOnline,
  libraryStatus,
}: {
  providerHealth: ProviderHealth[];
  daemonOnline: boolean;
  libraryStatus: LibraryStatus | null;
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>Providers & Diagnostics</h3>
          <p>
            Daemon {daemonOnline ? "online" : "offline"} · Integrity{" "}
            {libraryStatus?.integrityStatus ?? "unknown"}
          </p>
        </div>
        <span className={daemonOnline ? "status completed" : "status failed"}>
          {daemonOnline ? "online" : "offline"}
        </span>
      </div>
      <div className="provider-diagnostics-grid">
        {providerHealth.map((provider) => (
          <div className="provider-diagnostic-card" key={provider.provider}>
            <div className="panel-header">
              <div>
                <h4>{provider.displayName}</h4>
                <p>{provider.provider}</p>
              </div>
              <span className={`status ${provider.availability === "available" ? "completed" : "queued"}`}>
                {provider.availability}
              </span>
            </div>
            <div className="meta-grid">
              <span>Credentials</span>
              <strong>{provider.credentialState}</strong>
              <span>Capabilities</span>
              <strong>{provider.supportedOperations.join(", ") || "none"}</strong>
            </div>
            {provider.recoverableError && <p className="error-text">{provider.recoverableError}</p>}
          </div>
        ))}
      </div>
    </div>
  );
}

function SettingsUpdatesView({
  updateState,
  onCheckUpdate,
  onInstallUpdate,
  onRestartApp,
}: {
  updateState: UpdateState;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
}) {
  const update = updateState.availableUpdate;
  const busy = updateState.checking || updateState.installing;
  const statusText =
    updateState.status === "checking"
      ? "Checking"
      : updateState.status === "available"
        ? "Update available"
        : updateState.status === "installing"
          ? "Installing"
          : updateState.status === "pendingRestart"
            ? "Restart required"
            : updateState.status === "error"
              ? "Needs attention"
              : updateState.status === "upToDate"
                ? "Up to date"
                : "Idle";

  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>App Updates</h3>
          <p>
            Current version {updateState.currentVersion}
            {updateState.lastCheckedAt ? ` · checked ${displayDate(updateState.lastCheckedAt)}` : ""}
          </p>
        </div>
        <span className={`status ${updateState.status === "error" ? "failed" : updateState.status === "available" ? "queued" : "completed"}`}>
          {statusText}
        </span>
      </div>
      <div className="updates-panel">
        <div className="meta-grid">
          <span>Current</span>
          <strong>{updateState.currentVersion}</strong>
          <span>Latest</span>
          <strong>{update?.version ?? (updateState.status === "upToDate" ? updateState.currentVersion : "unknown")}</strong>
          <span>Last checked</span>
          <strong>{updateState.lastCheckedAt ? displayDate(updateState.lastCheckedAt) : "never"}</strong>
        </div>
        {update?.body && (
          <div className="update-notes">
            <h4>Release Notes</h4>
            <p>{update.body}</p>
          </div>
        )}
        {updateState.error && <p className="error-text">{updateState.error}</p>}
        <div className="row-actions">
          <button onClick={onCheckUpdate} disabled={busy}>
            {updateState.checking ? "Checking..." : "Check for Updates"}
          </button>
          <button onClick={onInstallUpdate} disabled={busy || !update || updateState.pendingRestart}>
            {updateState.installing ? "Installing..." : "Download and Install"}
          </button>
          <button onClick={onRestartApp} disabled={!updateState.pendingRestart}>
            Restart
          </button>
        </div>
        <p className="settings-note">
          Updates are verified with the Tauri updater public key. Ad-hoc macOS signing is not Apple notarization.
        </p>
      </div>
    </div>
  );
}

function SettingsLogsView({
  logs,
  logsLoading,
  selectedLogPath,
  selectedLogContent,
  logContentLoading,
  onRefreshLogs,
  onSelectLog,
}: {
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
}) {
  return (
    <div className="settings-section settings-logs-panel">
      <div className="panel-header">
        <div>
          <h3>Logs</h3>
          <p>{logsLoading ? "Loading logs..." : `${logs.length} recent log${logs.length === 1 ? "" : "s"}`}</p>
        </div>
        <button onClick={onRefreshLogs} disabled={logsLoading}>
          {logsLoading ? "Refreshing..." : "Refresh"}
        </button>
      </div>
      {logs.length === 0 ? (
        <div className="empty-state compact">No app logs found.</div>
      ) : (
        <div className="logs-browser">
          <div className="logs-list">
            {logs.map((log) => (
              <button
                key={log.path}
                className={log.path === selectedLogPath ? "log-list-item selected" : "log-list-item"}
                onClick={() => onSelectLog(log.path)}
              >
                <span className="log-list-heading">
                  <strong>{log.kind}</strong>
                  <span>{formatBytes(log.sizeBytes)}</span>
                </span>
                <span className="log-list-meta">
                  <span>{displayDate(log.modifiedAt)}</span>
                </span>
              </button>
            ))}
          </div>
          <div className="log-preview">
            {logContentLoading ? (
              <div className="empty-state compact">Loading log preview...</div>
            ) : selectedLogContent ? (
              <>
                <div className="log-preview-meta">
                  <span className="mono-line">{selectedLogContent.path}</span>
                  {selectedLogContent.truncated && <strong>Truncated</strong>}
                </div>
                <pre>{selectedLogContent.content || "Log is empty."}</pre>
              </>
            ) : (
              <div className="empty-state compact">Select a log to preview.</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export function Inspector({
  asset,
  detailState,
  onClose,
  onUpdateRating,
  onUpdateTitle,
  onAddTag,
  albums,
  onAddToAlbum,
  onSelectVersion,
  onPreviewImage,
  onGenerateFromReference,
  onGenerateVariation,
}: {
  asset: GalleryAsset | null;
  detailState: DetailLoadState<AssetDetail>;
  onClose: () => void;
  onUpdateRating: (rating: number) => void;
  onUpdateTitle: (title: string) => void;
  onAddTag: (tag: string) => void;
  albums: AlbumListItem[];
  onAddToAlbum: (albumId: string) => void;
  onSelectVersion: (versionId: string) => void;
  onPreviewImage: (image: LightboxImage) => void;
  onGenerateFromReference: (reference: ReferenceSource) => void;
  onGenerateVariation: (versionId?: string | null) => void;
}) {
  const detail = detailState.detail;
  const [tagInput, setTagInput] = useState("");
  const [tagEditorOpen, setTagEditorOpen] = useState(false);
  const [titleEditing, setTitleEditing] = useState(false);
  const [titleInput, setTitleInput] = useState("");
  const [albumToAdd, setAlbumToAdd] = useState("");
  useEffect(() => {
    setTagInput("");
    setTagEditorOpen(false);
    setTitleEditing(false);
    setTitleInput(detail?.title ?? asset?.title ?? "");
    setAlbumToAdd("");
  }, [detail?.id, detail?.title, asset?.title]);
  const submitTag = () => {
    const trimmed = tagInput.trim();
    if (trimmed.length === 0) {
      return;
    }
    onAddTag(trimmed);
    setTagInput("");
    setTagEditorOpen(false);
  };
  const saveTitle = () => {
    const trimmed = titleInput.trim();
    setTitleEditing(false);
    if (trimmed.length > 0) {
      onUpdateTitle(trimmed);
    }
  };
  if (!asset) {
    return (
      <aside className="inspector">
        <button className="inspector-close" onClick={onClose}>Close</button>
        <h2>Inspector</h2>
        <div className="empty-state compact">No asset selected.</div>
      </aside>
    );
  }
  if (detailState.loading) {
    return (
      <aside className="inspector">
        <button className="inspector-close" onClick={onClose}>Close</button>
        <h2>Inspector</h2>
        <div className="empty-state compact">Loading asset detail...</div>
      </aside>
    );
  }
  if (detailState.error || !detail) {
    return (
      <aside className="inspector">
        <button className="inspector-close" onClick={onClose}>Close</button>
        <h2>Inspector</h2>
        <div className="empty-state compact">{detailState.error ?? "Detail unavailable."}</div>
      </aside>
    );
  }
  return (
    <aside className="inspector">
      <button className="inspector-close" onClick={onClose}>Close</button>
      <section className="inspector-hero">
        {asset.imagePath ? (
          <button
            className="inspector-thumbnail-button"
            aria-label="Open full image preview"
            onClick={() => onPreviewImage({ path: asset.imagePath!, label: asset.title ?? "Generated image" })}
          >
            <Thumbnail asset={asset} index={0} />
          </button>
        ) : (
          <Thumbnail asset={asset} index={0} />
        )}
        <div>
          {titleEditing ? (
            <input
              className="title-input"
              value={titleInput}
              autoFocus
              onChange={(event) => setTitleInput(event.target.value)}
              onBlur={saveTitle}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  saveTitle();
                }
                if (event.key === "Escape") {
                  setTitleInput(detail.title ?? asset.title ?? "");
                  setTitleEditing(false);
                }
              }}
            />
          ) : (
            <h2 className="editable-title" onDoubleClick={() => setTitleEditing(true)}>
              {detail.title ?? asset.title ?? "Untitled"}
            </h2>
          )}
          <StarRatingDisplay rating={detail.rating} showEmpty />
          {detail.reviewPendingCount > 0 && <strong>Review pending</strong>}
          <small>Added: {displayDate(detail.createdAt)}</small>
        </div>
      </section>
      <InspectorSection title="Prompt">
        <p>{detail.prompt ?? "Prompt is unavailable for this version."}</p>
        <button className="text-button">Show full prompt</button>
      </InspectorSection>
      <InspectorSection title="Rating">
        <StarRatingControl rating={detail.rating} onChange={onUpdateRating} />
      </InspectorSection>
      <InspectorSection title="Provider & Model">
        <MetaRow label="Provider" value={detail.provider ?? asset.provider ?? "-"} />
        <MetaRow label="Model" value={detail.modelLabel ?? asset.modelLabel ?? "-"} />
        <MetaRow label="Category" value={detail.category ?? asset.category ?? "-"} />
        <MetaRow label="Parameters" value={detail.parametersJson ?? "-"} />
      </InspectorSection>
      <InspectorSection title="Description">
        <p>{detail.description ?? "No description."}</p>
      </InspectorSection>
      <InspectorSection title="JSON Schema Prompt">
        <pre className="schema-prompt-preview">{detail.schemaPrompt ?? "No schema prompt."}</pre>
      </InspectorSection>
      <InspectorSection title="Tags">
        <div className="tag-list">
          {detail.tags.map((tag) => (
            <span key={tag}>{tag}</span>
          ))}
          {tagEditorOpen && (
            <input
              className="tag-input"
              value={tagInput}
              autoFocus
              onChange={(event) => setTagInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  submitTag();
                }
                if (event.key === "Escape") {
                  setTagInput("");
                  setTagEditorOpen(false);
                }
              }}
              placeholder="Add tag"
            />
          )}
          <button
            className="mini-button"
            aria-label={tagEditorOpen ? "Add tag" : "Open tag editor"}
            disabled={tagEditorOpen && tagInput.trim().length === 0}
            onClick={() => {
              if (tagEditorOpen) {
                submitTag();
              } else {
                setTagEditorOpen(true);
              }
            }}
          >
            <Icon name="plus" />
          </button>
        </div>
      </InspectorSection>
      <InspectorSection title="Albums">
        {detail.albums.length === 0 ? (
          <p>No albums yet.</p>
        ) : (
          detail.albums.map((album) => <MetaRow key={album.id} label={album.kind} value={album.name} />)
        )}
        <div className="add-album-row">
          <select className="select-control" value={albumToAdd} onChange={(event) => setAlbumToAdd(event.target.value)}>
            <option value="">Add to album</option>
            {albums
              .filter((album) => album.kind === "manual")
              .map((album) => (
                <option key={album.id} value={album.id}>
                  {album.name}
                </option>
              ))}
          </select>
          <button
            className="mini-button"
            disabled={albumToAdd.length === 0}
            onClick={() => {
              onAddToAlbum(albumToAdd);
              setAlbumToAdd("");
            }}
          >
            Add
          </button>
        </div>
      </InspectorSection>
      <InspectorSection title="Versions & Lineage">
        <VersionLineagePanel
          detail={detail}
          onSelectVersion={onSelectVersion}
          onPreviewImage={onPreviewImage}
          onGenerateFromReference={onGenerateFromReference}
          onGenerateVariation={onGenerateVariation}
        />
      </InspectorSection>
      <InspectorSection title="File">
        {detail.file ? (
          <>
            <MetaRow label="Filename" value={detail.file.filename} />
            <MetaRow label="Location" value={detail.file.relativeLocation} />
            <MetaRow label="Size" value={formatBytes(detail.file.sizeBytes)} />
            <MetaRow label="Dimensions" value={formatDimensions(detail.file)} />
            <MetaRow label="Aspect Ratio" value={formatAspectRatio(detail.file.width, detail.file.height)} />
            <MetaRow label="Integrity" value={detail.file.integrityStatus} />
            <MetaRow label="Checksum" value={formatChecksum(detail.file)} />
          </>
        ) : (
          <p>File context is unavailable.</p>
        )}
      </InspectorSection>
    </aside>
  );
}

function VersionLineagePanel({
  detail,
  onSelectVersion,
  onPreviewImage,
  onGenerateFromReference,
  onGenerateVariation,
}: {
  detail: AssetDetail;
  onSelectVersion: (versionId: string) => void;
  onPreviewImage: (image: LightboxImage) => void;
  onGenerateFromReference: (reference: ReferenceSource) => void;
  onGenerateVariation: (versionId?: string | null) => void;
}) {
  const current =
    detail.versions.find((version) => version.id === detail.currentVersionId) ??
    detail.lineage[0]?.version ??
    detail.versions[0] ??
    null;
  const lineage = detail.lineage.length > 0 ? detail.lineage : detail.versions.map((version) => ({ version, generationEvent: null }));
  const eventByVersionId = new Map(lineage.map((entry) => [entry.version.id, entry.generationEvent]));
  return (
    <div className="version-lineage-panel">
      <div className="version-current-card">
        <span className="version-current-kicker">Current version</span>
        <strong>{current?.versionName ?? formatVersionName(current?.id ?? detail.id)}</strong>
        <div className="version-current-meta">
          <span>{detail.versions.length} version{detail.versions.length === 1 ? "" : "s"}</span>
          <span>{lineage[0]?.generationEvent?.operationType ? formatOperation(lineage[0].generationEvent.operationType) : "Asset lineage"}</span>
        </div>
      </div>

      <div className="version-browser" aria-label="Asset versions">
        {detail.versions.map((version) => {
          const selected = version.id === detail.currentVersionId;
          const event = eventByVersionId.get(version.id) ?? null;
          return (
            <article className={selected ? "version-browser-row selected" : "version-browser-row"} key={version.id}>
              <button className="version-browser-main" onClick={() => onSelectVersion(version.id)}>
                <span className="version-thumb">
                  <img
                    alt={version.versionName ?? formatVersionName(version.id)}
                    src={convertImagePath(version.filePath)}
                    loading="lazy"
                    decoding="async"
                  />
                </span>
                <span>
                  <strong>{version.versionName ?? formatVersionName(version.id)}</strong>
                  <small>{event?.prompt ?? "Imported or metadata-only version"}</small>
                </span>
              </button>
              <button className="mini-button" onClick={() => onGenerateVariation(version.id)}>
                Generate
              </button>
            </article>
          );
        })}
      </div>

      {detail.sourceReference ? (
        <div className="version-current-card reference-source-card">
          <span className="version-current-kicker">Reference source</span>
          <div className="reference-source-content">
            <button
              className="reference-source-preview"
              aria-label="Open reference source image preview"
              onClick={() =>
                onPreviewImage({
                  path: detail.sourceReference!.filePath,
                  label: detail.sourceReference!.assetTitle ?? "Reference image",
                })
              }
            >
              <img
                alt={detail.sourceReference.assetTitle ?? "Reference image"}
                src={convertImagePath(detail.sourceReference.filePath)}
                loading="lazy"
                decoding="async"
              />
            </button>
            <div>
              <strong>{detail.sourceReference.assetTitle ?? shortIdentifier(detail.sourceReference.assetId)}</strong>
              <div className="version-current-meta">
                <span>{detail.sourceReference.versionName}</span>
                <span>{detail.sourceReference.assetStatus}</span>
              </div>
              <button className="mini-button" onClick={() => onGenerateFromReference(detail.sourceReference!)}>
                Regenerate
              </button>
            </div>
          </div>
        </div>
      ) : null}

      <div className="lineage-timeline" aria-label="Version lineage">
        {lineage.length === 0 ? (
          <p>No lineage available.</p>
        ) : (
          lineage.map((entry, index) => {
            const isCurrent = index === 0;
            return (
              <div className={isCurrent ? "lineage-node current" : "lineage-node"} key={entry.version.id}>
                <span className="lineage-marker" aria-hidden="true" />
                <div className="lineage-node-main">
                  <strong>{isCurrent ? "Current" : "Parent"}</strong>
                  <span>{entry.version.versionName ?? formatVersionName(entry.version.id)}</span>
                  <small>{entry.generationEvent ? `${entry.generationEvent.provider} · ${entry.generationEvent.providerModel}` : "Imported version"}</small>
                </div>
                <code title={entry.version.id}>{shortIdentifier(entry.version.id)}</code>
              </div>
            );
          })
        )}
      </div>

      <button className="variation-button" onClick={() => onGenerateVariation(current?.id ?? detail.currentVersionId)}>
        <Icon name="plus" />
        <span>Generate variation</span>
      </button>
    </div>
  );
}

function InspectorSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="inspector-section">
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="meta-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function displayDate(value: string) {
  if (/^\d+$/.test(value)) {
    return new Date(Number(value)).toLocaleString();
  }
  return value;
}

function formatBytes(value: number | null) {
  if (!value) {
    return "-";
  }
  if (value >= 1024 * 1024 * 1024) {
    return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
  if (value > 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
  }
  return `${Math.round(value / 1024)} KB`;
}

function formatDimensions(file: FileContext) {
  return formatResolution(file.width, file.height);
}

function formatResolution(width: number | null, height: number | null) {
  if (!width || !height) {
    return "Unavailable";
  }
  return `${width} x ${height}`;
}

function formatChecksum(file: FileContext) {
  return `${file.checksumAlgorithm}: ${file.checksum}`;
}
