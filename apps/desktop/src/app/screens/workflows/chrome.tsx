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
  availableProviders,
  albums,
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
            <select
              className="select-control"
              value={query.providers[0] ?? ""}
              onChange={(event) =>
                onQueryChange(updateGalleryQuery(query, { providers: event.target.value ? [event.target.value] : [] }))
              }
            >
              <option value="">Any provider</option>
              {availableProviders.map((provider) => (
                <option key={provider} value={provider}>
                  {provider}
                </option>
              ))}
            </select>
            <GalleryAlbumSelector
              albums={albums}
              query={query}
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
          <GalleryFilterChips
            query={query}
            albums={albums}
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
  onQueryChange,
}: {
  albums: AlbumListItem[];
  query: GalleryQueryState;
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const selectedIds = galleryAlbumFilterIds(query);
  const unassigned = query.albumFilter.mode === "unassigned";
  return (
    <details className="filter-popover">
      <summary className={selectedIds.length > 0 || unassigned ? "chip-button active" : "chip-button"}>
        Albums
      </summary>
      <div className="filter-popover-panel">
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={unassigned}
            onChange={() => onQueryChange(unassigned ? clearGalleryAlbumFilter(query) : setGalleryUnassignedAlbumFilter(query))}
          />
          <span>Not in any album</span>
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
        <button onClick={() => onQueryChange(clearGalleryAlbumFilter(query))}>Clear album filter</button>
      </div>
    </details>
  );
}

function GalleryFilterChips({
  query,
  albums,
  onQueryChange,
}: {
  query: GalleryQueryState;
  albums: AlbumListItem[];
  onQueryChange: (query: GalleryQueryState) => void;
}) {
  const selectedIds = galleryAlbumFilterIds(query);
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
          label={`Search: ${query.text.trim()}`}
          ariaLabel="Clear search filter"
          onClick={() => onQueryChange(clearGalleryTextFilter(query))}
        />
      )}
      {query.providers.map((provider) => (
        <FilterChipButton
          key={provider}
          label={`Provider: ${provider}`}
          ariaLabel={`Clear provider filter ${provider}`}
          onClick={() => onQueryChange(clearGalleryProviderFilter(query, provider))}
        />
      ))}
      {query.tags.map((tag) => (
        <FilterChipButton
          key={tag}
          label={`Tag: ${tag}`}
          ariaLabel={`Clear tag filter ${tag}`}
          onClick={() => onQueryChange(clearGalleryTagFilter(query, tag))}
        />
      ))}
      {query.albumFilter.mode === "unassigned" && (
        <FilterChipButton
          label="Not in any album"
          ariaLabel="Clear album filter"
          onClick={() => onQueryChange(clearGalleryAlbumFilter(query))}
        />
      )}
      {selectedIds.map((albumId) => {
        const albumName = albums.find((album) => album.id === albumId)?.name ?? albumId;
        return (
          <FilterChipButton
            key={albumId}
            label={`Album: ${albumName}`}
            ariaLabel={`Clear album filter ${albumName}`}
            onClick={() => onQueryChange(removeGalleryAlbumFilter(query, albumId))}
          />
        );
      })}
      {query.minRating !== null && (
        <FilterChipButton
          label={`${query.minRating}+ stars`}
          ariaLabel="Clear rating filter"
          onClick={() => onQueryChange(clearGalleryMinRatingFilter(query))}
        />
      )}
      {query.reviewStatus === "pending" && (
        <FilterChipButton
          label="Review pending"
          ariaLabel="Clear review filter"
          onClick={() => onQueryChange(clearGalleryReviewFilter(query))}
        />
      )}
      <button onClick={() => onQueryChange(resetGalleryQuery())}>Clear all</button>
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
