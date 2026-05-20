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
import { displayDate, formatBytes } from "./common";
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
