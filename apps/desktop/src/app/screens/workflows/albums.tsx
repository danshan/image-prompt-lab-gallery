import React, { useEffect, useState } from "react";
import { moveItem } from "../../workflows/shared/state";
import {
  previewSmartAlbumCount,
  smartAlbumQueryJson,
  splitCsv,
  updateGalleryQuery,
  type GallerySort,
} from "../../workflows/albums";
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
  const smartBuilder = {
    text: smartText,
    tags: smartTags,
    providers: smartProviders,
    minRating: smartMinRating,
    reviewPending: smartReviewPending,
    category: smartCategory,
    status: smartStatus,
    createdAtFrom: smartCreatedAtFrom,
    createdAtTo: smartCreatedAtTo,
    sort: smartSort,
  };
  const smartPreviewCount = previewSmartAlbumCount(gallery, smartBuilder);
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
                    onCreateSmartAlbum(newAlbumName, smartAlbumQueryJson(smartBuilder));
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
