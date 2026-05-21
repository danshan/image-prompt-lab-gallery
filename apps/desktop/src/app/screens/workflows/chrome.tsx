import React, { useEffect, useState } from "react";
import {
  resetGalleryQuery,
  toggleGalleryProvider,
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
