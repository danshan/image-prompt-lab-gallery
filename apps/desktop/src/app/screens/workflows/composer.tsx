import React, { useEffect, useRef, useState } from "react";
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

type ComposerReferenceImage = {
  id: string;
  name: string;
  imagePath: string | null;
};

export function GenerationComposer({
  prompt,
  provider,
  inputSourceName,
  referenceImages,
  submitting,
  dictionary,
  onPromptChange,
  onProviderChange,
  onChooseReferenceImage,
  onClearReferenceImage,
  onClose,
  onGenerate,
}: {
  prompt: string;
  provider: string;
  inputSourceName: string | null;
  referenceImages: ComposerReferenceImage[];
  submitting: boolean;
  dictionary: Dictionary;
  onPromptChange: (value: string) => void;
  onProviderChange: (value: string) => void;
  onChooseReferenceImage: () => void;
  onClearReferenceImage: () => void;
  onClose: () => void;
  onGenerate: () => void;
}) {
  const hasInputSource = inputSourceName !== null;
  const promptRef = useRef<HTMLTextAreaElement | null>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (typeof document === "undefined") {
      return;
    }
    previousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    window.setTimeout(() => promptRef.current?.focus(), 0);
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  useEffect(() => {
    return () => {
      const previousFocus = previousFocusRef.current;
      if (previousFocus && typeof document !== "undefined" && document.contains(previousFocus)) {
        previousFocus.focus();
      }
    };
  }, []);

  return (
    <>
      <button
        className="generation-modal-backdrop open"
        aria-label={dictionary.workflow.closeGenerateModal}
        type="button"
        onClick={onClose}
      />
      <section className="generation-modal" role="dialog" aria-modal="true" aria-labelledby="generation-modal-title">
        <header className="generation-modal-header">
          <div>
            <h2 id="generation-modal-title">{dictionary.generate}</h2>
            <p>
              {hasInputSource
                ? `${dictionary.workflow.referenceSource}: ${inputSourceName}`
                : dictionary.workflow.generateModalDescription}
            </p>
          </div>
          <button className="icon-button" aria-label={dictionary.workflow.closeGenerateModal} onClick={onClose}>
            <Icon name="close" />
          </button>
        </header>
        <div className="generation-modal-body">
          <label className="generation-field">
            <span>{dictionary.workflow.provider}</span>
            <select className="select-control" value={provider} onChange={(event) => onProviderChange(event.target.value)}>
              <option value="codex-cli">codex-cli</option>
              <option value="fake">fake</option>
            </select>
          </label>
          <section className="generation-reference-field" aria-label={dictionary.workflow.referenceImages}>
            <div className="generation-reference-header">
              <span>{dictionary.workflow.referenceImages}</span>
              {referenceImages.length > 0 && <small>{referenceImages.length}</small>}
            </div>
            <div className="generation-reference-strip">
              {referenceImages.length === 0 ? (
                <span className="generation-reference-empty">{dictionary.workflow.noReferenceImages}</span>
              ) : (
                referenceImages.map((reference) => (
                  <figure className="generation-reference-thumb" key={reference.id} title={reference.name}>
                    {reference.imagePath ? (
                      <img src={convertImagePath(reference.imagePath)} alt={reference.name} />
                    ) : (
                      <span>{dictionary.workflow.referenceImage}</span>
                    )}
                    <figcaption>{reference.name}</figcaption>
                  </figure>
                ))
              )}
              <button className="secondary-button generation-reference-add" type="button" onClick={onChooseReferenceImage}>
                <Icon name="plus" />
                <span>{dictionary.workflow.add}</span>
              </button>
              {referenceImages.length > 0 && (
                <button className="secondary-button generation-reference-clear" type="button" onClick={onClearReferenceImage}>
                  {dictionary.workflow.clear}
                </button>
              )}
            </div>
          </section>
          <label className="generation-field">
            <span>{dictionary.workflow.prompt}</span>
            <textarea
              ref={promptRef}
              value={prompt}
              onChange={(event) => onPromptChange(event.target.value)}
              placeholder={hasInputSource ? `Prompt for image-to-image from ${inputSourceName}` : dictionary.workflow.prompt}
            />
          </label>
        </div>
        <footer className="generation-modal-footer">
          <small>{dictionary.workflow.generateReferenceScope}</small>
          <button className="primary-button" disabled={submitting || prompt.trim().length === 0} onClick={onGenerate}>
            {submitting ? dictionary.workflow.enqueueing : hasInputSource ? dictionary.workflow.generateImage : dictionary.workflow.run}
          </button>
        </footer>
      </section>
    </>
  );
}
