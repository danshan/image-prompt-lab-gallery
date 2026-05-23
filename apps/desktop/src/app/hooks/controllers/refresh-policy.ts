import { useEffect, type MutableRefObject } from "react";
import {
  GALLERY_REFRESH_DEBOUNCE_MS,
  TASK_QUEUE_BACKGROUND_POLL_INTERVAL_MS,
  TASK_QUEUE_POLL_INTERVAL_MS,
} from "../../types";
import {
  clearSelectionForLibrarySwitch,
  type DetailLoadState,
  type GalleryQueryState,
} from "../../workflows/gallery";
import { createReviewFormState } from "../../workflows/review";
import type {
  AlbumListItem,
  AssetDetail,
  DaemonTask,
  DaemonTaskDetail,
  GalleryAsset,
  Library,
  Suggestion,
  TaskPanel,
  View,
} from "../../types";

export function useStudioRefreshPolicy({
  runningInTauri,
  library,
  activeView,
  selectedTaskId,
  selectedSuggestion,
  selectedAsset,
  albumAddDrawerOpen,
  albumAddQuery,
  galleryQuery,
  selectedAlbumId,
  selectedAlbumKind,
  albums,
  reviewFormSuggestionId,
  selectedAssetCurrentVersionId,
  metadataPollTimeoutsRef,
  completedTaskKeysRef,
  refreshLibraries,
  checkForAppUpdate,
  refreshGallery,
  refreshSelectedAlbumContents,
  refreshAlbumAddCandidates,
  refreshAlbums,
  refreshSuggestions,
  refreshDaemonHealth,
  refreshTasks,
  loadTaskDetail,
  refreshAppLogs,
  refreshSuggestionHistory,
  loadAssetDetail,
  loadPreviewAssetDetail,
  setTaskDetail,
  setSelectedSuggestionId,
  setReviewForm,
  setSuggestionHistory,
  setDetailState,
}: {
  runningInTauri: boolean;
  library: Library | null;
  activeView: View;
  selectedTaskId: string | null;
  selectedSuggestion: Suggestion | null;
  selectedAsset: GalleryAsset | null;
  albumAddDrawerOpen: boolean;
  albumAddQuery: GalleryQueryState;
  galleryQuery: GalleryQueryState;
  selectedAlbumId: string | null;
  selectedAlbumKind: AlbumListItem["kind"] | undefined;
  albums: AlbumListItem[];
  reviewFormSuggestionId: string | null;
  selectedAssetCurrentVersionId: string | null | undefined;
  metadataPollTimeoutsRef: MutableRefObject<Set<number>>;
  completedTaskKeysRef: MutableRefObject<Set<string>>;
  refreshLibraries: () => Promise<void>;
  checkForAppUpdate: (options?: { silent?: boolean }) => Promise<void>;
  refreshGallery: () => Promise<void>;
  refreshSelectedAlbumContents: (albumId?: string | null) => Promise<void>;
  refreshAlbumAddCandidates: (nextQuery?: GalleryQueryState) => Promise<void>;
  refreshAlbums: () => Promise<void>;
  refreshSuggestions: () => Promise<void>;
  refreshDaemonHealth: () => Promise<void>;
  refreshTasks: (options?: { showLoading?: boolean }) => Promise<DaemonTask[]>;
  loadTaskDetail: (taskId: string) => Promise<void>;
  refreshAppLogs: () => Promise<void>;
  refreshSuggestionHistory: (suggestion: Suggestion) => Promise<void>;
  loadAssetDetail: (assetId: string, versionId: string | null) => Promise<void>;
  loadPreviewAssetDetail: (asset: GalleryAsset) => void;
  setTaskDetail: (detail: DaemonTaskDetail | null) => void;
  setSelectedSuggestionId: (suggestionId: string | null) => void;
  setReviewForm: (form: ReturnType<typeof createReviewFormState> | null) => void;
  setSuggestionHistory: (suggestions: Suggestion[]) => void;
  setDetailState: (state: DetailLoadState<AssetDetail>) => void;
}) {
  useEffect(() => {
    if (runningInTauri) {
      void refreshLibraries();
      void checkForAppUpdate({ silent: true });
    }
  }, [runningInTauri]);

  useEffect(() => {
    if (!runningInTauri || !library) {
      return;
    }
    const timer = window.setTimeout(() => {
      void refreshGallery();
    }, GALLERY_REFRESH_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [runningInTauri, library?.rootPath, galleryQuery]);

  useEffect(() => {
    void refreshSelectedAlbumContents();
  }, [runningInTauri, library?.rootPath, selectedAlbumId, selectedAlbumKind, albums]);

  useEffect(() => {
    if (!albumAddDrawerOpen) {
      return;
    }
    void refreshAlbumAddCandidates(albumAddQuery);
  }, [runningInTauri, library?.rootPath, albumAddDrawerOpen, albumAddQuery, selectedAlbumId]);

  useEffect(() => {
    return () => {
      for (const timer of metadataPollTimeoutsRef.current) {
        window.clearTimeout(timer);
      }
      metadataPollTimeoutsRef.current.clear();
    };
  }, [library?.rootPath]);

  useEffect(() => {
    if (runningInTauri && library) {
      completedTaskKeysRef.current = new Set();
      void refreshAlbums();
      void refreshSuggestions();
      void refreshDaemonHealth();
      void refreshTasks();
    }
  }, [runningInTauri, library?.rootPath]);

  useEffect(() => {
    if (!runningInTauri || !library) {
      return;
    }
    const showLoading = activeView === "queue";
    const intervalMs = showLoading
      ? TASK_QUEUE_POLL_INTERVAL_MS
      : TASK_QUEUE_BACKGROUND_POLL_INTERVAL_MS;

    void refreshTasks({ showLoading });
    const timer = window.setInterval(() => {
      void refreshTasks({ showLoading });
      if (showLoading && selectedTaskId) {
        void loadTaskDetail(selectedTaskId);
      }
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [runningInTauri, library?.rootPath, activeView, selectedTaskId]);

  useEffect(() => {
    if (!selectedTaskId) {
      setTaskDetail(null);
      return;
    }
    void loadTaskDetail(selectedTaskId);
  }, [selectedTaskId]);

  useEffect(() => {
    if (activeView === "settings") {
      void refreshAppLogs();
    }
  }, [activeView, runningInTauri]);

  useEffect(() => {
    if (!selectedSuggestion) {
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setSuggestionHistory([]);
      return;
    }
    if (reviewFormSuggestionId !== selectedSuggestion.id) {
      setSelectedSuggestionId(selectedSuggestion.id);
      setReviewForm(createReviewFormState(selectedSuggestion));
    }
    void refreshSuggestionHistory(selectedSuggestion);
  }, [selectedSuggestion?.id]);

  useEffect(() => {
    if (!selectedAsset) {
      setDetailState(clearSelectionForLibrarySwitch());
      return;
    }
    if (runningInTauri && library) {
      void loadAssetDetail(selectedAsset.id, selectedAsset.currentVersionId);
    } else {
      loadPreviewAssetDetail(selectedAsset);
    }
  }, [runningInTauri, library?.rootPath, selectedAsset?.id, selectedAssetCurrentVersionId]);
}
