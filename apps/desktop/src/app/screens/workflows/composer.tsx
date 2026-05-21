import React, { useEffect, useState } from "react";
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
