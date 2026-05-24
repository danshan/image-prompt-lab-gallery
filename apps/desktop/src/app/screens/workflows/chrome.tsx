import React, { useEffect, useState } from "react";
import {
  clearGalleryAlbumFilter,
  clearGalleryMinRatingFilter,
  clearGalleryProviderFilter,
  clearGalleryReviewFilter,
  clearGalleryTagFilter,
  clearGalleryTextFilter,
  galleryAlbumFilterIds,
  removeGalleryAlbumFilter,
  resetGalleryQuery,
  setGalleryUnassignedAlbumFilter,
  toggleGalleryAlbumFilter,
  updateGalleryQuery,
  type GalleryQueryState,
  type GallerySort,
  type ReviewStatusFilter,
} from "../../workflows/gallery";
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

export function StudioOverviewBand({
  assetCount,
  reviewCount,
  queueCount,
  integrityIssueCount,
  activeView,
  dictionary,
}: {
  assetCount: number;
  reviewCount: number;
  queueCount: number;
  integrityIssueCount: number;
  activeView: View;
  dictionary: Dictionary;
}) {
  const cards = [
    { label: dictionary.overview.assets, value: assetCount, tone: activeView === "gallery" ? "active" : "" },
    { label: dictionary.overview.pendingReview, value: reviewCount, tone: reviewCount > 0 ? "warning" : "" },
    { label: dictionary.overview.activeTasks, value: queueCount, tone: queueCount > 0 ? "active" : "" },
    { label: dictionary.overview.integrityIssues, value: integrityIssueCount, tone: integrityIssueCount > 0 ? "danger" : "healthy" },
  ];
  return (
    <section className="studio-overview-band" aria-label={dictionary.overview.label}>
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
  availableProviders,
  albums,
  dictionary,
  onComposerOpenChange,
  onQueryChange,
}: {
  activeView: View;
  query: GalleryQueryState;
  itemCount: number;
  status: string;
  composerOpen: boolean;
  availableProviders: string[];
  albums: AlbumListItem[];
  dictionary: Dictionary;
  onComposerOpenChange: (open: boolean) => void;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const label = dictionary.views[activeView];
  const galleryCopy = dictionary.galleryControls;
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
          <span>{dictionary.generate}</span>
        </button>
      </div>

      {showGalleryControls && (
        <>
          <div className="search-row">
            <label className="search-box">
              <Icon name="search" />
              <span>{galleryCopy.searchLabel}</span>
              <input
                value={query.text}
                onChange={(event) => onQueryChange(updateGalleryQuery(query, { text: event.target.value }))}
                placeholder={galleryCopy.searchPlaceholder}
              />
            </label>
            <button className="icon-button" aria-label={galleryCopy.gridView}>
              <Icon name="grid" />
            </button>
            <button className="icon-button" aria-label={galleryCopy.listView}>
              <Icon name="list" />
            </button>
          </div>
          <div className="filter-row">
            <select
              className="select-control"
              value={query.providers[0] ?? ""}
              onChange={(event) =>
                onQueryChange(updateGalleryQuery(query, { providers: event.target.value ? [event.target.value] : [] }))
              }
            >
              <option value="">{galleryCopy.anyProvider}</option>
              {availableProviders.map((provider) => (
                <option key={provider} value={provider}>
                  {provider}
                </option>
              ))}
            </select>
            <GalleryAlbumSelector
              albums={albums}
              query={query}
              dictionary={dictionary}
              onQueryChange={onQueryChange}
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
              <option value="">{galleryCopy.rating}</option>
              <option value="5">5 {galleryCopy.stars}</option>
              <option value="4">4+ {galleryCopy.starsPlus}</option>
              <option value="3">3+ {galleryCopy.starsPlus}</option>
            </select>
            <select
              className="select-control"
              value={query.reviewStatus}
              onChange={(event) =>
                onQueryChange(updateGalleryQuery(query, { reviewStatus: event.target.value as ReviewStatusFilter }))
              }
            >
              <option value="any">{galleryCopy.review}</option>
              <option value="pending">{galleryCopy.reviewPending}</option>
            </select>
            <button onClick={() => onQueryChange(resetGalleryQuery())}>{galleryCopy.clearAll}</button>
            <select
              className="select-control sort-select"
              value={query.sort}
              onChange={(event) => onQueryChange(updateGalleryQuery(query, { sort: event.target.value as GallerySort }))}
            >
              <option value="newest">{galleryCopy.sortNewest}</option>
              <option value="oldest">{galleryCopy.sortOldest}</option>
              <option value="ratingDesc">{galleryCopy.sortRating}</option>
              <option value="titleAsc">{galleryCopy.sortTitle}</option>
              <option value="providerAsc">{galleryCopy.sortProvider}</option>
            </select>
            <strong>{itemCount} {galleryCopy.items}</strong>
          </div>
          <GalleryFilterChips
            query={query}
            albums={albums}
            dictionary={dictionary}
            onQueryChange={onQueryChange}
          />
        </>
      )}
    </header>
  );
}

function GalleryAlbumSelector({
  albums,
  query,
  dictionary,
  onQueryChange,
}: {
  albums: AlbumListItem[];
  query: GalleryQueryState;
  dictionary: Dictionary;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const selectedIds = galleryAlbumFilterIds(query);
  const unassigned = query.albumFilter.mode === "unassigned";
  const galleryCopy = dictionary.galleryControls;
  return (
    <details className="filter-popover">
      <summary className={selectedIds.length > 0 || unassigned ? "chip-button active" : "chip-button"}>
        {galleryCopy.albums}
      </summary>
      <div className="filter-popover-panel">
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={unassigned}
            onChange={() => onQueryChange(unassigned ? clearGalleryAlbumFilter(query) : setGalleryUnassignedAlbumFilter(query))}
          />
          <span>{galleryCopy.notInAnyAlbum}</span>
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
        <button onClick={() => onQueryChange(clearGalleryAlbumFilter(query))}>{galleryCopy.clearAlbumFilter}</button>
      </div>
    </details>
  );
}

function GalleryFilterChips({
  query,
  albums,
  dictionary,
  onQueryChange,
}: {
  query: GalleryQueryState;
  albums: AlbumListItem[];
  dictionary: Dictionary;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const selectedIds = galleryAlbumFilterIds(query);
  const galleryCopy = dictionary.galleryControls;
  const hasFilters =
    query.text.trim().length > 0 ||
    query.providers.length > 0 ||
    query.minRating !== null ||
    query.reviewStatus !== "any" ||
    query.tags.length > 0 ||
    query.albumFilter.mode !== "any";
  if (!hasFilters) {
    return null;
  }
  return (
    <div className="active-filter-row">
      {query.text.trim().length > 0 && (
        <FilterChipButton
          label={`${galleryCopy.searchChip}: ${query.text.trim()}`}
          ariaLabel={galleryCopy.clearSearchFilter}
          onClick={() => onQueryChange(clearGalleryTextFilter(query))}
        />
      )}
      {query.providers.map((provider) => (
        <FilterChipButton
          key={provider}
          label={`${galleryCopy.providerChip}: ${provider}`}
          ariaLabel={`${galleryCopy.clearProviderFilter} ${provider}`}
          onClick={() => onQueryChange(clearGalleryProviderFilter(query, provider))}
        />
      ))}
      {query.tags.map((tag) => (
        <FilterChipButton
          key={tag}
          label={`${galleryCopy.tagChip}: ${tag}`}
          ariaLabel={`${galleryCopy.clearTagFilter} ${tag}`}
          onClick={() => onQueryChange(clearGalleryTagFilter(query, tag))}
        />
      ))}
      {query.albumFilter.mode === "unassigned" && (
        <FilterChipButton
          label={galleryCopy.notInAnyAlbum}
          ariaLabel={galleryCopy.clearAlbumFilter}
          onClick={() => onQueryChange(clearGalleryAlbumFilter(query))}
        />
      )}
      {selectedIds.map((albumId) => {
        const albumName = albums.find((album) => album.id === albumId)?.name ?? albumId;
        return (
          <FilterChipButton
            key={albumId}
            label={`${galleryCopy.albumChip}: ${albumName}`}
            ariaLabel={`${galleryCopy.clearAlbumFilter} ${albumName}`}
            onClick={() => onQueryChange(removeGalleryAlbumFilter(query, albumId))}
          />
        );
      })}
      {query.minRating !== null && (
        <FilterChipButton
          label={`${query.minRating}+ ${galleryCopy.starsPlus}`}
          ariaLabel={galleryCopy.clearRatingFilter}
          onClick={() => onQueryChange(clearGalleryMinRatingFilter(query))}
        />
      )}
      {query.reviewStatus === "pending" && (
        <FilterChipButton
          label={galleryCopy.reviewPending}
          ariaLabel={galleryCopy.clearReviewFilter}
          onClick={() => onQueryChange(clearGalleryReviewFilter(query))}
        />
      )}
      <button onClick={() => onQueryChange(resetGalleryQuery())}>{galleryCopy.clearAll}</button>
    </div>
  );
}

function FilterChipButton({
  label,
  ariaLabel,
  onClick,
}: {
  label: string;
  ariaLabel: string;
  onClick: () => void;
}) {
  return (
    <button className="filter-chip" aria-label={ariaLabel} onClick={onClick}>
      <span>{label}</span>
      <Icon name="close" />
    </button>
  );
}
