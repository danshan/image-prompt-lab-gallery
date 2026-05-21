import type { Dispatch, SetStateAction } from "react";
import { clearAlbumQuery, type DetailLoadState, type GalleryQueryState } from "../../workflows/albums";
import { clearLibraryWorkspaceState } from "../../workflows/library/state";
import type { ReviewFormState } from "../../workflows/review";
import {
  mockAlbumList,
  mockGallery,
  mockLibraryStatus,
  mockProviderHealth,
  mockSuggestions,
  mockTasks,
} from "../../mock-data";
import {
  buildChildPath,
  validLibraryFolderName,
} from "../../utils";
import {
  errorMessage,
  invokeCommand,
  pickDirectory,
  pickSaveZipPath,
  pickZipFile,
} from "../../tauri-adapter";
import type {
  AlbumListItem,
  AssetDetail,
  DaemonTask,
  DaemonTaskDetail,
  GalleryAsset,
  Library,
  LibraryBackup,
  LibraryStatus,
  ProviderHealth,
  Suggestion,
} from "../../types";

export function useLibrarySettingsActions({
  runningInTauri,
  library,
  query,
  libraryFolderNameInput,
  libraryNameInput,
  setLibraries,
  setLibrary,
  setLibraryStatus,
  setProviderHealth,
  setGallery,
  setSelectedGalleryAssetIds,
  setSelectedAssetId,
  setDetailState,
  setAlbums,
  setSelectedAlbumId,
  setAlbumSearchInput,
  setAlbumNameInput,
  setAlbumCreateOpen,
  setSuggestions,
  setSelectedSuggestionId,
  setSelectedSuggestionIds,
  setSuggestionHistory,
  setReviewForm,
  setTasks,
  setSelectedTaskId,
  setTaskDetail,
  setQuery,
  setLibraryFolderNameInput,
  setStatus,
  setRecoverableError,
  setPendingLibraryActions,
  setMissingLibraryPaths,
}: {
  runningInTauri: boolean;
  library: Library | null;
  query: GalleryQueryState;
  libraryFolderNameInput: string;
  libraryNameInput: string;
  setLibraries: Dispatch<SetStateAction<Library[]>>;
  setLibrary: Dispatch<SetStateAction<Library | null>>;
  setLibraryStatus: Dispatch<SetStateAction<LibraryStatus | null>>;
  setProviderHealth: Dispatch<SetStateAction<ProviderHealth[]>>;
  setGallery: Dispatch<SetStateAction<GalleryAsset[]>>;
  setSelectedGalleryAssetIds: Dispatch<SetStateAction<string[]>>;
  setSelectedAssetId: Dispatch<SetStateAction<string>>;
  setDetailState: Dispatch<SetStateAction<DetailLoadState<AssetDetail>>>;
  setAlbums: Dispatch<SetStateAction<AlbumListItem[]>>;
  setSelectedAlbumId: Dispatch<SetStateAction<string | null>>;
  setAlbumSearchInput: Dispatch<SetStateAction<string>>;
  setAlbumNameInput: Dispatch<SetStateAction<string>>;
  setAlbumCreateOpen: Dispatch<SetStateAction<boolean>>;
  setSuggestions: Dispatch<SetStateAction<Suggestion[]>>;
  setSelectedSuggestionId: Dispatch<SetStateAction<string | null>>;
  setSelectedSuggestionIds: Dispatch<SetStateAction<string[]>>;
  setSuggestionHistory: Dispatch<SetStateAction<Suggestion[]>>;
  setReviewForm: Dispatch<SetStateAction<ReviewFormState | null>>;
  setTasks: Dispatch<SetStateAction<DaemonTask[]>>;
  setSelectedTaskId: Dispatch<SetStateAction<string | null>>;
  setTaskDetail: Dispatch<SetStateAction<DaemonTaskDetail | null>>;
  setQuery: Dispatch<SetStateAction<GalleryQueryState>>;
  setLibraryFolderNameInput: Dispatch<SetStateAction<string>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
  setPendingLibraryActions: Dispatch<SetStateAction<string[]>>;
  setMissingLibraryPaths: Dispatch<SetStateAction<string[]>>;
}) {
  async function refreshLibraries() {
    try {
      const libraries = await invokeCommand<Library[]>("list_libraries", { includeHidden: false });
      const nextLibrary = libraries[0] ?? null;
      setLibraries(libraries);
      setLibrary(nextLibrary);
      setStatus(nextLibrary ? "Library opened" : "No library registered");
      if (nextLibrary) {
        void refreshLibraryStatus(nextLibrary.rootPath);
      }
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  function setLibraryActionPending(key: string, pending: boolean) {
    setPendingLibraryActions((current) => {
      if (pending) {
        return current.includes(key) ? current : [...current, key];
      }
      return current.filter((item) => item !== key);
    });
  }

  function rememberMissingLibraryPath(rootPath: string) {
    setMissingLibraryPaths((current) => (current.includes(rootPath) ? current : [...current, rootPath]));
  }

  function forgetMissingLibraryPath(rootPath: string) {
    setMissingLibraryPaths((current) => current.filter((path) => path !== rootPath));
  }

  function replaceRegisteredLibrary(updated: Library) {
    setLibraries((current) => current.map((item) => (item.id === updated.id ? updated : item)));
    setLibrary((current) => (current?.id === updated.id ? updated : current));
  }

  function clearCurrentLibraryContext() {
    const cleared = clearLibraryWorkspaceState<AssetDetail>();
    setLibrary(null);
    setLibraryStatus(null);
    setGallery([]);
    setSelectedGalleryAssetIds(cleared.selectedGalleryAssetIds);
    setSelectedAssetId(cleared.selectedAssetId);
    setDetailState(cleared.detailState);
    setAlbums([]);
    setSelectedAlbumId(cleared.selectedAlbumId);
    setAlbumSearchInput("");
    setAlbumNameInput("");
    setAlbumCreateOpen(false);
    setSuggestions([]);
    setSelectedSuggestionId(cleared.selectedSuggestionId);
    setSelectedSuggestionIds(cleared.selectedSuggestionIds);
    setSuggestionHistory([]);
    setReviewForm(cleared.reviewForm);
    setTasks([]);
    setSelectedTaskId(cleared.selectedTaskId);
    setTaskDetail(null);
    setQuery(clearAlbumQuery(query));
    setLibraryFolderNameInput("image-prompt-lab");
    setStatus("No library selected");
    setRecoverableError(null);
  }

  async function refreshLibraryStatus(rootPath: string) {
    try {
      const nextStatus = await invokeCommand<LibraryStatus>("library_status", { rootPath });
      setLibraryStatus(nextStatus);
      if (runningInTauri) {
        const overview = await invokeCommand<{ providerHealth: ProviderHealth[] }>("diagnostics_overview", { rootPath });
        setProviderHealth(overview.providerHealth);
      }
    } catch (error) {
      setLibraryStatus(null);
      setRecoverableError(errorMessage(error));
    }
  }

  async function createLibrary() {
    const folderName = libraryFolderNameInput.trim();
    if (!validLibraryFolderName(folderName)) {
      setStatus("Folder name must not be empty or contain path separators");
      return;
    }
    try {
      const parentPath = await pickDirectory("Choose Library Parent Folder");
      if (!parentPath) {
        return;
      }
      const rootPath = buildChildPath(parentPath, folderName);
      const created = await invokeCommand<Library>("create_library", {
        input: {
          rootPath,
          name: libraryNameInput.trim() || "Image Prompt Lab",
        },
      });
      setLibraries((current) => [created, ...current.filter((item) => item.id !== created.id)]);
      setLibrary(created);
      setLibraryStatus(null);
      setGallery([]);
      setAlbums([]);
      setAlbumSearchInput("");
      setAlbumNameInput("");
      setAlbumCreateOpen(false);
      setSuggestions([]);
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setSelectedAssetId("");
      setStatus("Library created");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function openExistingLibraryFromPrompt() {
    try {
      const selectedPath = await pickDirectory("Open Existing Library");
      if (!selectedPath) {
        return;
      }
      const opened = await invokeCommand<Library>("open_library", {
        rootPath: selectedPath,
      });
      setLibraries((current) => [opened, ...current.filter((item) => item.id !== opened.id)]);
      setLibrary(opened);
      forgetMissingLibraryPath(opened.rootPath);
      setLibraryStatus(null);
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setStatus("Library opened");
    } catch (error) {
      setStatus(errorMessage(error));
    }
  }

  async function renameLibraryAlias(item: Library) {
    const alias = window.prompt("Rename Library", item.name);
    if (alias === null) {
      return;
    }
    const actionKey = `rename:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      const updated = await invokeCommand<Library>("rename_library_alias", {
        input: {
          libraryId: item.id,
          alias,
        },
      });
      replaceRegisteredLibrary(updated);
      setStatus("Library renamed");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function unregisterLibrary(item: Library) {
    const confirmed = window.confirm("Close this library in the app? Files on disk are not deleted.");
    if (!confirmed) {
      return;
    }
    const actionKey = `close:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      await invokeCommand<void>("unregister_library", { libraryId: item.id });
      setLibraries((current) => current.filter((library) => library.id !== item.id));
      if (library?.id === item.id) {
        clearCurrentLibraryContext();
      } else {
        setStatus("Library closed");
      }
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function exportLibraryBackup(item: Library) {
    const defaultPath = `${item.rootPath.replace(/\/$/, "")}.zip`;
    const actionKey = `export:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      const outputZipPath = await pickSaveZipPath("Export Library Zip", defaultPath);
      if (!outputZipPath) {
        return;
      }
      await invokeCommand<void>("export_library_backup_zip", {
        input: {
          libraryPath: item.rootPath,
          outputZipPath,
        },
      });
      setStatus(`Library exported to ${outputZipPath}`);
      forgetMissingLibraryPath(item.rootPath);
    } catch (error) {
      const message = errorMessage(error);
      setStatus(message);
      if (message.includes("not found") || message.includes("missing")) {
        rememberMissingLibraryPath(item.rootPath);
      }
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function importLibraryBackup() {
    const actionKey = "import";
    setLibraryActionPending(actionKey, true);
    try {
      const zipPath = await pickZipFile("Import Library Zip");
      if (!zipPath) {
        return;
      }
      const destinationPath = await pickDirectory("Import Destination Folder");
      if (!destinationPath) {
        return;
      }
      const imported = await invokeCommand<LibraryBackup>("import_library_backup_zip", {
        input: {
          zipPath,
          destinationPath,
        },
      });
      setLibraries((current) => [imported.library, ...current.filter((item) => item.id !== imported.library.id)]);
      setLibrary(imported.library);
      forgetMissingLibraryPath(imported.library.rootPath);
      setLibraryStatus(null);
      setSelectedAlbumId(null);
      setSelectedSuggestionId(null);
      setReviewForm(null);
      setStatus(imported.cloned ? "Library imported as copy" : "Library imported");
    } catch (error) {
      setStatus(errorMessage(error));
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  async function revealLibraryFolder(item: Library) {
    const actionKey = `reveal:${item.id}`;
    setLibraryActionPending(actionKey, true);
    try {
      await invokeCommand<void>("reveal_library_folder", { rootPath: item.rootPath });
      setStatus("Library folder opened");
      forgetMissingLibraryPath(item.rootPath);
    } catch (error) {
      const message = errorMessage(error);
      setStatus(message);
      if (message.includes("missing") || message.includes("not found")) {
        rememberMissingLibraryPath(item.rootPath);
      }
    } finally {
      setLibraryActionPending(actionKey, false);
    }
  }

  return {
    refreshLibraries,
    refreshLibraryStatus,
    createLibrary,
    openExistingLibraryFromPrompt,
    renameLibraryAlias,
    unregisterLibrary,
    exportLibraryBackup,
    importLibraryBackup,
    revealLibraryFolder,
  };
}
