import React from "react";
import { formatAspectRatio, reviewFormTags, type ReviewFieldName, type ReviewFormState } from "../workbench-state";
import { mockDetail } from "./mock-data";
import type { AssetDetail, FileContext, GalleryAsset, Suggestion } from "./types";

export function validLibraryFolderName(folderName: string) {
  return folderName.length > 0 && folderName !== "." && folderName !== ".." && !folderName.includes("/") && !folderName.includes("\\");
}

export function buildChildPath(parentPath: string, folderName: string) {
  const separator = parentPath.includes("\\") && !parentPath.includes("/") ? "\\" : "/";
  return `${parentPath.replace(/[\\/]+$/, "")}${separator}${folderName}`;
}

export function nextAnimationFrame() {
  return new Promise<void>((resolve) => {
    window.requestAnimationFrame(() => resolve());
  });
}

export function mockDetailFor(asset: GalleryAsset): AssetDetail {
  return {
    ...mockDetail,
    id: asset.id,
    title: asset.title,
    category: asset.category,
    rating: asset.rating,
    status: asset.status,
    provider: asset.provider,
    modelLabel: asset.modelLabel,
    tags: asset.tags,
    reviewPendingCount: asset.reviewPendingCount,
    currentVersionId: asset.currentVersionId,
    currentVersionNumber: asset.currentVersionNumber ?? null,
    currentVersionName: asset.currentVersionName ?? asset.versionLabel,
    file: mockDetail.file
      ? {
          ...mockDetail.file,
          width: asset.width,
          height: asset.height,
        }
      : null,
  };
}

export function suggestionFromAsset(asset: GalleryAsset): Suggestion {
  return {
    id: `suggestion-${asset.id}-${Date.now()}`,
    assetId: asset.id,
    title: asset.title,
    description: null,
    schemaPrompt: schemaPromptFromAsset(asset, asset.prompt ?? asset.title ?? ""),
    tags: asset.tags,
    category: asset.category,
    status: "pending_review",
  };
}

export function reviewFieldContext(
  form: ReviewFormState,
  suggestion: Suggestion,
  asset: GalleryAsset | null | undefined,
) {
  return {
    currentTitle: form.title.trim() || null,
    currentDescription: form.description.trim() || null,
    currentSchemaPrompt: form.schemaPrompt.trim() || null,
    assetTitle: asset?.title ?? suggestion.title,
    assetPrompt: asset?.prompt ?? null,
    tags: reviewFormTags(form),
    category: form.category.trim() || suggestion.category,
    provider: asset?.provider ?? null,
    modelLabel: asset?.modelLabel ?? null,
    width: asset?.width ?? null,
    height: asset?.height ?? null,
  };
}

export function previewGeneratedReviewField(
  field: ReviewFieldName,
  asset: GalleryAsset | null | undefined,
  sourceText: string,
): string {
  if (field === "title") {
    return titleFromPrompt(sourceText);
  }
  if (field === "description") {
    return descriptionFromPrompt(sourceText);
  }
  return schemaPromptFromAsset(asset, sourceText);
}

export function titleFromPrompt(prompt: string | null | undefined): string {
  const words = (prompt ?? "")
    .split(/[^a-zA-Z0-9]+/)
    .map((word) => word.trim().toLowerCase())
    .filter((word) => word.length > 0 && !["a", "an", "and", "the", "of", "with", "to", "for"].includes(word))
    .slice(0, 6)
    .map((word) => word.slice(0, 1).toUpperCase() + word.slice(1));
  return words.join(" ") || "Untitled Review";
}

export function descriptionFromPrompt(prompt: string | null | undefined): string {
  const text = (prompt ?? "").trim();
  if (!text) {
    return "Generated visual asset prepared for metadata review.";
  }
  return `Review draft based on the generation prompt: ${text}`;
}

export function schemaPromptFromAsset(asset: GalleryAsset | null | undefined, sourceText: string): string {
  const aspectRatio = asset?.width && asset.height ? formatAspectRatio(asset.width, asset.height) : "unspecified";
  const schema = {
    GLOBAL_SETTINGS: {
      aspect_ratio: aspectRatio,
      style: "derived from current asset and prompt",
      clarity: "sharp foreground, visible subject detail",
      render_flags: ["sharp_foreground", "micro_texture", "editorial_finish"],
    },
    ENVIRONMENT: {
      background: "preserve the generated image environment cues",
      lighting: "preserve visible lighting direction and contrast",
      atmosphere: ["match final image mood", "avoid unsupported scene changes"],
    },
    CORE_ASSETS: {
      primary_subject: sourceText || asset?.title || "reviewed image asset",
      materials: ["infer visible materials from final image"],
      composition: "preserve generated composition and camera framing",
    },
    OUTPUT: {
      mood: "match accepted visual direction",
      avoid: ["cheap e-commerce banner", "plastic CGI", "fake brand logos"],
    },
  };
  return `// VERSION: 0.1\n// AESTHETIC: review draft\n${JSON.stringify(schema, null, 2)}`;
}

export function thumbnailStyle(asset: GalleryAsset, index: number): React.CSSProperties {
  const styles = [
    "linear-gradient(135deg, #0b0b0b, #052f34 45%, #d98b00)",
    "linear-gradient(135deg, #26465a, #f18f6f 50%, #ffd2a6)",
    "linear-gradient(135deg, #d8d6cd, #f7f3ea 45%, #57756b)",
    "linear-gradient(135deg, #8f2717, #ef7847 45%, #4c153d)",
    "linear-gradient(135deg, #121820, #586878 45%, #c7a36f)",
    "linear-gradient(135deg, #081b2d, #184e63 45%, #e07f3b)",
  ];
  return {
    aspectRatio: thumbnailAspectRatio(asset),
    background: styles[index % styles.length],
  };
}

export function thumbnailImageStyle(asset: GalleryAsset): React.CSSProperties {
  return isOverTallThumbnail(asset) ? { objectPosition: "top center" } : {};
}

export function thumbnailAspectRatio(asset: GalleryAsset) {
  if (!hasValidDimensions(asset)) {
    return "4 / 3";
  }
  if (isOverTallThumbnail(asset)) {
    return "2 / 3";
  }
  return `${asset.width} / ${asset.height}`;
}

export function isOverTallThumbnail(asset: GalleryAsset) {
  if (!hasValidDimensions(asset)) {
    return false;
  }
  return asset.height / asset.width > 3 / 2;
}

export function hasValidDimensions(asset: GalleryAsset): asset is GalleryAsset & { width: number; height: number } {
  return typeof asset.width === "number" && typeof asset.height === "number" && asset.width > 0 && asset.height > 0;
}
