import React, { useState } from "react";
import { moveItem } from "../../workflows/shared/state";
import {
  clearGalleryAlbumFilter,
  galleryAlbumFilterIds,
  previewSmartAlbumCount,
  setGalleryUnassignedAlbumFilter,
  smartAlbumQueryJson,
  splitCsv,
  toggleGalleryAlbumFilter,
  updateGalleryQuery,
  type GalleryQueryState,
  type GallerySort,
  type ReviewStatusFilter,
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
import type { Dictionary } from "../../i18n/dictionaries";
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
  addDrawerOpen,
  addQuery,
  addGallery,
  addSelectionIds,
  addSubmitting,
  onOpenAddDrawer,
  onCloseAddDrawer,
  onAddQueryChange,
  onToggleAddSelection,
  onSubmitAddSelection,
  onSelectAsset,
  onPreviewImage,
  dictionary,
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
  addDrawerOpen: boolean;
  addQuery: GalleryQueryState;
  addGallery: GalleryAsset[];
  addSelectionIds: string[];
  addSubmitting: boolean;
  onOpenAddDrawer: () => void;
  onCloseAddDrawer: () => void;
  onAddQueryChange: (query: GalleryQueryState) => void;
  onToggleAddSelection: (assetId: string) => void;
  onSubmitAddSelection: () => void;
  onSelectAsset: (assetId: string) => void;
  onPreviewImage: (image: LightboxImage) => void;
  dictionary: Dictionary;
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
            <h3>{dictionary.workflow.albumsTitle}</h3>
            <p>{loading ? dictionary.workflow.loadingAlbums : `${albums.length} ${albums.length === 1 ? dictionary.workflow.album : dictionary.workflow.albums}`}</p>
          </div>
        </div>
        <div className="album-search-row">
          <input
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder={dictionary.workflow.searchAlbums}
          />
          <button className="icon-button" aria-label={dictionary.workflow.createAlbum} onClick={() => onCreateOpenChange(!createOpen)}>
            <Icon name="plus" />
          </button>
          {createOpen && (
            <div className="album-create-popover">
              <label>
                <span>{dictionary.workflow.albumName}</span>
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
                  placeholder={dictionary.workflow.newManualAlbum}
                />
              </label>
              <label>
                <span>{dictionary.workflow.kind}</span>
                <select className="select-control" value={newAlbumKind} onChange={(event) => setNewAlbumKind(event.target.value as "manual" | "smart")}>
                  <option value="manual">{dictionary.workflow.manual}</option>
                  <option value="smart">{dictionary.workflow.smart}</option>
                </select>
              </label>
              {newAlbumKind === "smart" && (
                <div className="smart-builder">
                  <label>
                    <span>{dictionary.workflow.text}</span>
                    <input value={smartText} onChange={(event) => setSmartText(event.target.value)} />
                  </label>
                  <label>
                    <span>{dictionary.workflow.tags}</span>
                    <input list="smart-tag-options" value={smartTags} onChange={(event) => setSmartTags(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-tag-options">
                    {availableTags.map((tag) => <option key={tag} value={tag} />)}
                  </datalist>
                  <label>
                    <span>{dictionary.workflow.providers}</span>
                    <input list="smart-provider-options" value={smartProviders} onChange={(event) => setSmartProviders(event.target.value)} placeholder="comma separated" />
                  </label>
                  <datalist id="smart-provider-options">
                    {availableProviders.map((provider) => <option key={provider} value={provider} />)}
                  </datalist>
                  <label>
                    <span>{dictionary.workflow.minRating}</span>
                    <input value={smartMinRating} onChange={(event) => setSmartMinRating(event.target.value)} inputMode="numeric" />
                  </label>
                  <label>
                    <span>{dictionary.workflow.category}</span>
                    <select className="select-control" value={smartCategory} onChange={(event) => setSmartCategory(event.target.value)}>
                      <option value="">{dictionary.workflow.any}</option>
                      {availableCategories.map((category) => <option key={category} value={category}>{category}</option>)}
                    </select>
                  </label>
                  <label>
                    <span>{dictionary.workflow.status}</span>
                    <input value={smartStatus} onChange={(event) => setSmartStatus(event.target.value)} placeholder="generated, curated..." />
                  </label>
                  <label>
                    <span>{dictionary.workflow.createdFrom}</span>
                    <input value={smartCreatedAtFrom} onChange={(event) => setSmartCreatedAtFrom(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>{dictionary.workflow.createdTo}</span>
                    <input value={smartCreatedAtTo} onChange={(event) => setSmartCreatedAtTo(event.target.value)} placeholder="unix ms" />
                  </label>
                  <label>
                    <span>{dictionary.workflow.sort}</span>
                    <select className="select-control" value={smartSort} onChange={(event) => setSmartSort(event.target.value as GallerySort)}>
                      <option value="newest">{dictionary.workflow.newest}</option>
                      <option value="oldest">{dictionary.workflow.oldest}</option>
                      <option value="ratingDesc">{dictionary.workflow.rating}</option>
                      <option value="titleAsc">{dictionary.workflow.title}</option>
                      <option value="providerAsc">{dictionary.workflow.provider}</option>
                    </select>
                  </label>
                  <label className="checkbox-row">
                    <input type="checkbox" checked={smartReviewPending} onChange={(event) => setSmartReviewPending(event.target.checked)} />
                    <span>{dictionary.workflow.reviewPending}</span>
                  </label>
                  <small>{smartPreviewCount} visible assets match this builder.</small>
                </div>
              )}
              <div className="row-actions">
                <button onClick={() => onCreateOpenChange(false)}>{dictionary.workflow.cancel}</button>
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
                  {dictionary.workflow.create}
                </button>
              </div>
            </div>
          )}
        </div>
        {albums.length === 0 ? (
          <div className="empty-state compact">{dictionary.workflow.noAlbumsYet}</div>
        ) : visibleAlbums.length === 0 ? (
          <div className="empty-state compact">{dictionary.workflow.noAlbumsMatch}</div>
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
            <h3>{selectedAlbum?.name ?? dictionary.workflow.selectAlbum}</h3>
            <p>
              {selectedAlbum
                ? `${selectedAlbum.kind} album · ${gallery.length} visible item${gallery.length === 1 ? "" : "s"}`
                : dictionary.workflow.openAlbumAssets}
            </p>
          </div>
          {selectedAlbum && (
            <div className="row-actions">
              {selectedAlbum.kind === "manual" && (
                <button className="primary-button" onClick={onOpenAddDrawer}>
                  <Icon name="plus" />
                  <span>{dictionary.workflow.addImages}</span>
                </button>
              )}
              {selectedAlbum.kind === "smart" && <span className="rule-chip">{dictionary.workflow.ruleBased}</span>}
              <button onClick={() => {
                const next = window.prompt(dictionary.workflow.albumName, selectedAlbum.name);
                if (next) {
                  onRenameAlbum(selectedAlbum.id, next);
                }
              }}>
                {dictionary.workflow.rename}
              </button>
              <button onClick={() => onDeleteAlbum(selectedAlbum.id)}>{dictionary.workflow.delete}</button>
              <button onClick={onCloseAlbum}>{dictionary.workflow.allAssets}</button>
            </div>
          )}
        </div>
        {!selectedAlbum ? (
          <div className="empty-state">{dictionary.workflow.chooseAlbum}</div>
        ) : gallery.length === 0 ? (
          <div className="empty-state">{dictionary.workflow.albumEmpty}</div>
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
                <Thumbnail
                  asset={asset}
                  index={index}
                  onPreview={() => {
                    if (asset.imagePath) {
                      onPreviewImage({
                        path: asset.imagePath,
                        label: asset.title ?? dictionary.workflow.generatedImage,
                      });
                    }
                  }}
                  altLabel={dictionary.workflow.generatedImage}
                  previewLabel={dictionary.workflow.openOriginalImagePreview}
                  unavailableLabel={dictionary.workflow.imagePreviewUnavailable}
                />
                <span className="asset-title">{asset.title ?? dictionary.workflow.untitled}</span>
                <span className="provider-pill">{asset.provider ?? dictionary.workflow.unknownProvider}</span>
                <StarRatingDisplay rating={asset.rating} />
                {asset.reviewPendingCount > 0 && <span className="review-badge">{dictionary.workflow.reviewPending}</span>}
                {selectedAlbum.kind === "manual" && (
                  <span
                    className="text-button"
                    onClick={(event) => {
                      event.stopPropagation();
                      onRemoveAsset(asset.id);
                    }}
                  >
                    {dictionary.workflow.remove}
                  </span>
                )}
              </button>
            ))}
          </section>
        )}
      </div>
      {selectedAlbum?.kind === "manual" && addDrawerOpen && (
        <AddImagesDrawer
          query={addQuery}
          assets={addGallery}
          selectedIds={addSelectionIds}
          albums={albums}
          providers={availableProviders}
          submitting={addSubmitting}
          onQueryChange={onAddQueryChange}
          onToggleSelection={onToggleAddSelection}
          onSubmit={onSubmitAddSelection}
          onClose={onCloseAddDrawer}
          dictionary={dictionary}
        />
      )}
    </section>
  );
}

function AddImagesDrawer({
  query,
  assets,
  selectedIds,
  albums,
  providers,
  submitting,
  onQueryChange,
  onToggleSelection,
  onSubmit,
  onClose,
  dictionary,
}: {
  query: GalleryQueryState;
  assets: GalleryAsset[];
  selectedIds: string[];
  albums: AlbumListItem[];
  providers: string[];
  submitting: boolean;
  onQueryChange: (query: GalleryQueryState) => void;
  onToggleSelection: (assetId: string) => void;
  onSubmit: () => void;
  onClose: () => void;
  dictionary: Dictionary;
}) {
  return (
    <aside className="album-add-drawer" aria-label={dictionary.workflow.addImages}>
      <div className="drawer-header">
        <div>
          <h3>{dictionary.workflow.addImages}</h3>
          <p>{assets.length} eligible asset{assets.length === 1 ? "" : "s"}</p>
        </div>
        <button className="icon-button" aria-label={dictionary.closeContext} onClick={onClose}>
          <Icon name="close" />
        </button>
      </div>
      <div className="drawer-filter-row">
        <label className="search-box compact">
          <Icon name="search" />
          <span>{dictionary.galleryControls.searchLabel}</span>
          <input
            value={query.text}
            onChange={(event) => onQueryChange(updateGalleryQuery(query, { text: event.target.value }))}
            placeholder={dictionary.galleryControls.searchPlaceholder}
          />
        </label>
        <select
          className="select-control"
          value={query.providers[0] ?? ""}
          onChange={(event) =>
            onQueryChange(updateGalleryQuery(query, { providers: event.target.value ? [event.target.value] : [] }))
          }
        >
          <option value="">{dictionary.galleryControls.anyProvider}</option>
          {providers.map((provider) => (
            <option key={provider} value={provider}>
              {provider}
            </option>
          ))}
        </select>
        <select
          className="select-control"
          value={query.minRating ?? ""}
          onChange={(event) =>
            onQueryChange(updateGalleryQuery(query, { minRating: event.target.value ? Number(event.target.value) : null }))
          }
        >
          <option value="">{dictionary.workflow.rating}</option>
          <option value="5">5 {dictionary.galleryControls.stars}</option>
          <option value="4">4+ {dictionary.galleryControls.starsPlus}</option>
          <option value="3">3+ {dictionary.galleryControls.starsPlus}</option>
        </select>
        <select
          className="select-control"
          value={query.reviewStatus}
          onChange={(event) =>
            onQueryChange(updateGalleryQuery(query, { reviewStatus: event.target.value as ReviewStatusFilter }))
          }
        >
          <option value="any">{dictionary.galleryControls.review}</option>
          <option value="pending">{dictionary.workflow.reviewPending}</option>
        </select>
        <AlbumSourceFilterSelector albums={albums} query={query} onQueryChange={onQueryChange} dictionary={dictionary} />
      </div>
      <div className="album-add-list">
        {assets.length === 0 ? (
          <div className="empty-state compact">{dictionary.workflow.noAssetsMatch}</div>
        ) : (
          assets.map((asset, index) => (
            <label key={asset.id} className="album-add-item">
              <input
                type="checkbox"
                checked={selectedIds.includes(asset.id)}
                onChange={() => onToggleSelection(asset.id)}
              />
              <Thumbnail asset={asset} index={index} />
              <span>
                <strong>{asset.title ?? dictionary.workflow.untitled}</strong>
                <small>{asset.provider ?? dictionary.workflow.unknownProvider}</small>
              </span>
            </label>
          ))
        )}
      </div>
      <div className="drawer-footer">
        <button onClick={onClose}>{dictionary.workflow.cancel}</button>
        <button
          className="primary-button"
          disabled={selectedIds.length === 0 || submitting}
          onClick={onSubmit}
        >
          {dictionary.workflow.add} ({selectedIds.length})
        </button>
      </div>
    </aside>
  );
}

function AlbumSourceFilterSelector({
  albums,
  query,
  onQueryChange,
  dictionary,
}: {
  albums: AlbumListItem[];
  query: GalleryQueryState;
  onQueryChange: (query: GalleryQueryState) => void;
  dictionary: Dictionary;
}) {
  const selectedIds = galleryAlbumFilterIds(query);
  const unassigned = query.albumFilter.mode === "unassigned";
  const label = unassigned
    ? dictionary.galleryControls.notInAnyAlbum
    : selectedIds.length === 0
      ? dictionary.workflow.any
      : selectedIds.length === 1
        ? (albums.find((album) => album.id === selectedIds[0])?.name ?? `1 ${dictionary.workflow.album}`)
        : `${selectedIds.length} ${dictionary.workflow.albums}`;
  return (
    <details className="filter-popover">
      <summary className={selectedIds.length > 0 || unassigned ? "chip-button active" : "chip-button"}>
        {label}
      </summary>
      <div className="filter-popover-panel">
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={unassigned}
            onChange={() => onQueryChange(unassigned ? clearGalleryAlbumFilter(query) : setGalleryUnassignedAlbumFilter(query))}
          />
          <span>{dictionary.galleryControls.notInAnyAlbum}</span>
        </label>
        {albums.map((album) => (
          <label key={album.id} className="checkbox-row">
            <input
              type="checkbox"
              disabled={unassigned}
              checked={selectedIds.includes(album.id)}
              onChange={() => onQueryChange(toggleGalleryAlbumFilter(query, album.id))}
            />
            <span>{album.name}</span>
          </label>
        ))}
        <button onClick={() => onQueryChange(clearGalleryAlbumFilter(query))}>{dictionary.galleryControls.clearAlbumFilter}</button>
      </div>
    </details>
  );
}
