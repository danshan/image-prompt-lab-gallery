import React, { useEffect } from "react";
import { Icon } from "../../../studio-icons";
import { convertImagePath } from "../../tauri-adapter";
import { StarRatingDisplay } from "../../components/rating";
import { thumbnailImageStyle, thumbnailStyle } from "../../utils";
import { toggleGalleryTag, type GalleryQueryState } from "../../workflows/gallery";
import type { GalleryAsset, LightboxImage } from "../../types";

export function GalleryWorkspace({
  assets,
  selectedAssetId,
  selectedAssetIds,
  query,
  availableTags,
  onSelect,
  onToggleAssetSelection,
  onQueryChange,
  onRequestReview,
  onPreviewImage,
}: {
  assets: GalleryAsset[];
  selectedAssetId: string;
  selectedAssetIds: string[];
  query: GalleryQueryState;
  availableTags: string[];
  onSelect: (id: string) => void;
  onToggleAssetSelection: (id: string) => void;
  onQueryChange: (query: GalleryQueryState) => void;
  onRequestReview: (asset: GalleryAsset) => void;
  onPreviewImage: (image: LightboxImage) => void;
}) {
  if (assets.length === 0) {
    return <div className="empty-state">No assets match the current query.</div>;
  }
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
                      label: asset.title ?? "Generated image",
                    });
                  }
                }}
              />
              <span className="asset-title-row">
                <span className="asset-title">{asset.title ?? "Untitled"}</span>
                <span>{asset.currentVersionName ?? asset.versionLabel ?? "v1"}</span>
              </span>
            </div>
            <span className="asset-card-meta">
              <span className="provider-pill">{asset.provider ?? "Unknown provider"}</span>
              <StarRatingDisplay rating={asset.rating} />
            </span>
            <span className="card-tags">
              {asset.tags.slice(0, 3).map((tag) => (
                <span key={tag}>{tag}</span>
              ))}
            </span>
            <span className="asset-card-footer">
              {asset.reviewPendingCount > 0 ? <span className="review-badge">Review pending</span> : <span />}
              <span>{asset.versionCount} version{asset.versionCount === 1 ? "" : "s"}</span>
            </span>
            <span className="asset-card-actions">
              <button
                className="card-review-button"
                onClick={(event) => {
                  event.stopPropagation();
                  onRequestReview(asset);
                }}
              >
                Review
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
                <span>Select</span>
              </label>
            </span>
          </article>
        ))}
      </section>
    </>
  );
}

export function Thumbnail({ asset, index, onPreview }: { asset: GalleryAsset; index: number; onPreview?: () => void }) {
  const style = thumbnailStyle(asset, index);
  const imageStyle = thumbnailImageStyle(asset);
  const image = asset.imagePath ? (
    <img
      alt={asset.title ?? "Generated image"}
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
      aria-label={asset.imagePath ? "Open original image preview" : "Image preview unavailable"}
      onClick={(event) => {
        event.stopPropagation();
        onPreview();
      }}
    >
      {image}
    </button>
  );
}

export function ImageLightbox({ image, onClose }: { image: LightboxImage; onClose: () => void }) {
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
      <button className="image-lightbox-close" aria-label="Close image preview" onClick={onClose}>
        <Icon name="close" />
      </button>
      <div className="image-lightbox-frame">
        <img alt={image.label} src={convertImagePath(image.path)} onClick={(event) => event.stopPropagation()} />
      </div>
    </div>
  );
}
