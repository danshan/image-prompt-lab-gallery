import React, { useEffect } from "react";
import { Icon } from "../../../studio-icons";
import { convertImagePath } from "../../tauri-adapter";
import { StarRatingDisplay } from "../../components/rating";
import { thumbnailImageStyle, thumbnailStyle } from "../../utils";
import { formatVersionTreeSummary, toggleGalleryTag, type GalleryQueryState } from "../../workflows/gallery";
import type { GalleryAsset, LightboxImage } from "../../types";
import type { Dictionary } from "../../i18n/dictionaries";

export function GalleryWorkspace({
  assets,
  selectedAssetId,
  selectedAssetIds,
  query,
  availableTags,
  onSelect,
  onToggleAssetSelection,
  onClearSelection,
  onQueryChange,
  onRequestReview,
  onPreviewImage,
  dictionary,
}: {
  assets: GalleryAsset[];
  selectedAssetId: string;
  selectedAssetIds: string[];
  query: GalleryQueryState;
  availableTags: string[];
  onSelect: (id: string) => void;
  onToggleAssetSelection: (id: string) => void;
  onClearSelection: () => void;
  onQueryChange: (query: GalleryQueryState) => void;
  onRequestReview: (asset: GalleryAsset) => void;
  onPreviewImage: (image: LightboxImage) => void;
  dictionary: Dictionary;
}) {
  if (assets.length === 0) {
    return <div className="empty-state">{dictionary.workflow.noAssetsMatch}</div>;
  }
  const selectedAssets = assets.filter((asset) => selectedAssetIds.includes(asset.id));
  return (
    <>
      <div className="tag-filter-strip">
        {availableTags.slice(0, 8).map((tag) => (
          <button
            key={tag}
            className={query.tags.includes(tag) ? "tag-chip selected" : "tag-chip"}
            onClick={() => onQueryChange(toggleGalleryTag(query, tag))}
          >
            {tag}
          </button>
        ))}
      </div>
      {selectedAssets.length > 0 && (
        <section className="selection-action-bar" aria-label={dictionary.workflow.selectionActions}>
          <span>{selectedAssets.length} {dictionary.workflow.selected}</span>
          <button className="secondary-button" onClick={() => onRequestReview(selectedAssets[0])}>
            {dictionary.workflow.reviewFirst}
          </button>
          <button onClick={onClearSelection}>{dictionary.workflow.clearSelection}</button>
        </section>
      )}
      <section className="gallery-grid">
        {assets.map((asset, index) => (
          <article
            key={asset.id}
            className={asset.id === selectedAssetId ? "asset-card selected" : "asset-card"}
            role="button"
            tabIndex={0}
            onClick={() => onSelect(asset.id)}
            onKeyDown={(event) => {
              if (event.target !== event.currentTarget) {
                return;
              }
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onSelect(asset.id);
              }
            }}
          >
            <div className="asset-card-main">
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
              <span className="asset-title-row">
                <span className="asset-title">{asset.title ?? dictionary.workflow.untitled}</span>
                <span>{asset.currentVersionTreeName ?? asset.currentVersionName ?? asset.versionLabel ?? "v1"}</span>
              </span>
            </div>
            <span className="asset-card-meta">
              <span className="provider-pill">{asset.provider ?? dictionary.workflow.unknownProvider}</span>
              <StarRatingDisplay rating={asset.rating} />
            </span>
            <span className="card-tags">
              {asset.tags.slice(0, 3).map((tag) => (
                <span key={tag}>{tag}</span>
              ))}
            </span>
            <span className="asset-card-footer">
              {asset.reviewPendingCount > 0 ? <span className="review-badge">{dictionary.workflow.reviewPending}</span> : <span />}
              <span>{formatVersionTreeSummary(asset)}</span>
            </span>
            <span className="asset-card-actions">
              <button
                className="card-review-button"
                onClick={(event) => {
                  event.stopPropagation();
                  onRequestReview(asset);
                }}
              >
                {dictionary.review}
              </button>
              <label className="checkbox-row card-select-row" onClick={(event) => event.stopPropagation()}>
                <input
                  type="checkbox"
                  checked={selectedAssetIds.includes(asset.id)}
                  onChange={(event) => {
                    event.stopPropagation();
                    onToggleAssetSelection(asset.id);
                  }}
                />
                <span>{dictionary.workflow.select}</span>
              </label>
            </span>
          </article>
        ))}
      </section>
    </>
  );
}

export function Thumbnail({ asset, index, onPreview, altLabel, previewLabel, unavailableLabel }: { asset: GalleryAsset; index: number; onPreview?: () => void; altLabel?: string; previewLabel?: string; unavailableLabel?: string }) {
  const style = thumbnailStyle(asset, index);
  const imageStyle = thumbnailImageStyle(asset);
  const image = asset.imagePath ? (
    <img
      alt={asset.title ?? altLabel ?? "Generated image"}
      src={convertImagePath(asset.imagePath)}
      loading="lazy"
      decoding="async"
      style={imageStyle}
    />
  ) : null;
  if (!onPreview) {
    return (
      <span className="thumbnail" style={style}>
        {image}
      </span>
    );
  }
  return (
    <button
      className="thumbnail"
      style={style}
      type="button"
      disabled={!asset.imagePath}
      aria-label={asset.imagePath ? (previewLabel ?? "Open original image preview") : (unavailableLabel ?? "Image preview unavailable")}
      onClick={(event) => {
        event.stopPropagation();
        onPreview();
      }}
    >
      {image}
    </button>
  );
}

export function ImageLightbox({ image, onClose, closeLabel }: { image: LightboxImage; onClose: () => void; closeLabel: string }) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  const closeFromBackdrop = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.target === event.currentTarget) {
      onClose();
    }
  };

  return (
    <div className="image-lightbox" role="dialog" aria-modal="true" aria-label={image.label} onClick={closeFromBackdrop}>
      <button className="image-lightbox-close" aria-label={closeLabel} onClick={onClose}>
        <Icon name="close" />
      </button>
      <div className="image-lightbox-frame">
        <img alt={image.label} src={convertImagePath(image.path)} onClick={(event) => event.stopPropagation()} />
      </div>
    </div>
  );
}
