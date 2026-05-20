import React, { useEffect, useState } from "react";
import {
  addReviewFormTag,
  applySuggestionFieldToReviewForm,
  clearAlbumQuery,
  formatAspectRatio,
  isReviewFieldGenerating,
  libraryMaintenanceActions,
  moveItem,
  parseTaskDraftImport,
  removeReviewFormTag,
  resetGalleryQuery,
  reviewFormTags,
  selectedOrCurrentIds,
  toggleGalleryProvider,
  updateGalleryQuery,
  type GalleryQueryState,
  type GallerySort,
  type DetailLoadState,
  type ReviewFieldName,
  type ReviewFormState,
  type ReviewStatusFilter,
  type SettingsSection,
} from "../../../workbench-state";
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
import { displayDate, formatBytes } from "./common";
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
export function SettingsWorkspace({
  library,
  libraries,
  activeSection,
  providerHealth,
  daemonOnline,
  libraryStatus,
  onSectionChange,
  libraryFolderName,
  libraryName,
  onLibraryFolderNameChange,
  onLibraryNameChange,
  onCreate,
  onOpenExisting,
  onImportZip,
  onSwitchLibrary,
  onRenameLibrary,
  onCloseLibrary,
  onExportZip,
  onReveal,
  pendingLibraryActions,
  missingLibraryPaths,
  logs,
  logsLoading,
  selectedLogPath,
  selectedLogContent,
  logContentLoading,
  updateState,
  onRefreshLogs,
  onSelectLog,
  onCheckUpdate,
  onInstallUpdate,
  onRestartApp,
}: {
  library: Library | null;
  libraries: Library[];
  activeSection: SettingsSection;
  providerHealth: ProviderHealth[];
  daemonOnline: boolean;
  libraryStatus: LibraryStatus | null;
  onSectionChange: (section: SettingsSection) => void;
  libraryFolderName: string;
  libraryName: string;
  onLibraryFolderNameChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpenExisting: () => void;
  onImportZip: () => void;
  onSwitchLibrary: (libraryId: string) => void;
  onRenameLibrary: (library: Library) => void;
  onCloseLibrary: (library: Library) => void;
  onExportZip: (library: Library) => void;
  onReveal: (library: Library) => void;
  pendingLibraryActions: string[];
  missingLibraryPaths: string[];
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  updateState: UpdateState;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
}) {
  return (
    <section className="settings-workspace">
      <div className="settings-tabs" role="tablist" aria-label="Settings sections">
        <button className={activeSection === "libraries" ? "active" : ""} onClick={() => onSectionChange("libraries")}>
          Libraries
        </button>
        <button className={activeSection === "providers" ? "active" : ""} onClick={() => onSectionChange("providers")}>
          Providers
        </button>
        <button className={activeSection === "updates" ? "active" : ""} onClick={() => onSectionChange("updates")}>
          Updates
        </button>
        <button className={activeSection === "logs" ? "active" : ""} onClick={() => onSectionChange("logs")}>
          Logs
        </button>
      </div>
      {activeSection === "libraries" ? (
        <SettingsLibrariesView
          library={library}
          libraries={libraries}
          libraryFolderName={libraryFolderName}
          libraryName={libraryName}
          onLibraryFolderNameChange={onLibraryFolderNameChange}
          onLibraryNameChange={onLibraryNameChange}
          onCreate={onCreate}
          onOpenExisting={onOpenExisting}
          onImportZip={onImportZip}
          onSwitchLibrary={onSwitchLibrary}
          onRenameLibrary={onRenameLibrary}
          onCloseLibrary={onCloseLibrary}
          onExportZip={onExportZip}
          onReveal={onReveal}
          pendingLibraryActions={pendingLibraryActions}
          missingLibraryPaths={missingLibraryPaths}
        />
      ) : activeSection === "providers" ? (
        <SettingsProvidersView
          providerHealth={providerHealth}
          daemonOnline={daemonOnline}
          libraryStatus={libraryStatus}
        />
      ) : activeSection === "updates" ? (
        <SettingsUpdatesView
          updateState={updateState}
          onCheckUpdate={onCheckUpdate}
          onInstallUpdate={onInstallUpdate}
          onRestartApp={onRestartApp}
        />
      ) : (
        <SettingsLogsView
          logs={logs}
          logsLoading={logsLoading}
          selectedLogPath={selectedLogPath}
          selectedLogContent={selectedLogContent}
          logContentLoading={logContentLoading}
          onRefreshLogs={onRefreshLogs}
          onSelectLog={onSelectLog}
        />
      )}
    </section>
  );
}

function SettingsLibrariesView({
  library,
  libraries,
  libraryFolderName,
  libraryName,
  onLibraryFolderNameChange,
  onLibraryNameChange,
  onCreate,
  onOpenExisting,
  onImportZip,
  onSwitchLibrary,
  onRenameLibrary,
  onCloseLibrary,
  onExportZip,
  onReveal,
  pendingLibraryActions,
  missingLibraryPaths,
}: {
  library: Library | null;
  libraries: Library[];
  libraryFolderName: string;
  libraryName: string;
  onLibraryFolderNameChange: (value: string) => void;
  onLibraryNameChange: (value: string) => void;
  onCreate: () => void;
  onOpenExisting: () => void;
  onImportZip: () => void;
  onSwitchLibrary: (libraryId: string) => void;
  onRenameLibrary: (library: Library) => void;
  onCloseLibrary: (library: Library) => void;
  onExportZip: (library: Library) => void;
  onReveal: (library: Library) => void;
  pendingLibraryActions: string[];
  missingLibraryPaths: string[];
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>Libraries</h3>
          <p>
            {libraries.length} registered{library ? `, current: ${library.name}` : ""}
          </p>
        </div>
        <div className="row-actions">
          <button onClick={onOpenExisting}>Open Existing Library</button>
          <button onClick={onImportZip} disabled={pendingLibraryActions.includes("import")}>
            {pendingLibraryActions.includes("import") ? "Importing..." : "Import Zip"}
          </button>
        </div>
      </div>
      <div className="library-create-strip">
        <div className="library-section-heading">
          <h4>Create Library</h4>
          <p>Choose a parent folder after clicking Create. The library folder will be created inside it.</p>
        </div>
        <div className="library-create-controls">
          <label>
            <span>Library name</span>
            <input value={libraryName} onChange={(event) => onLibraryNameChange(event.target.value)} />
          </label>
          <label>
            <span>Folder name</span>
            <input value={libraryFolderName} onChange={(event) => onLibraryFolderNameChange(event.target.value)} />
          </label>
          <button onClick={onCreate}>Create...</button>
        </div>
      </div>
      <div className="library-section-heading library-list-heading">
        <h4>Registered Libraries</h4>
        <p>Switch, rename, close, export, or reveal libraries registered on this machine.</p>
      </div>
      {libraries.length === 0 ? (
        <div className="empty-state compact">No library registered.</div>
      ) : (
        <div className="library-table" role="table" aria-label="Registered libraries">
          <div className="library-table-row header" role="row">
            <span>Name</span>
            <span>Path</span>
            <span>Actions</span>
          </div>
          {libraries.map((item) => {
            const actions = libraryMaintenanceActions(item.rootPath, missingLibraryPaths);
            const isCurrent = library?.id === item.id;
            const busy = (name: string) => pendingLibraryActions.includes(`${name}:${item.id}`);
            return (
              <div
                key={item.id}
                className={isCurrent ? "library-table-row current" : "library-table-row"}
                role="row"
                tabIndex={isCurrent ? -1 : 0}
                aria-label={isCurrent ? `${item.name}, current library` : `Switch to ${item.name}`}
                onClick={() => {
                  if (!isCurrent) {
                    onSwitchLibrary(item.id);
                  }
                }}
                onKeyDown={(event) => {
                  if (!isCurrent && (event.key === "Enter" || event.key === " ")) {
                    event.preventDefault();
                    onSwitchLibrary(item.id);
                  }
                }}
              >
                <span className="library-row-main">
                  <strong>{item.name}</strong>
                  {isCurrent && <small>Current</small>}
                  {!actions.canReveal && <small>Missing on disk</small>}
                </span>
                <span className="mono-line" title={item.rootPath}>
                  {item.rootPath}
                </span>
                <span className="row-actions library-row-actions">
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Rename library"
                    data-tooltip={busy("rename") ? "Renaming..." : "Rename"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onRenameLibrary(item);
                    }}
                    disabled={busy("rename")}
                  >
                    <LibraryActionIcon kind={busy("rename") ? "loading" : "rename"} />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Export library zip"
                    data-tooltip={busy("export") ? "Exporting..." : "Export Zip"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onExportZip(item);
                    }}
                    disabled={!actions.canExport || busy("export")}
                  >
                    <LibraryActionIcon kind={busy("export") ? "loading" : "export"} />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Reveal library in Finder"
                    data-tooltip="Reveal in Finder"
                    onClick={(event) => {
                      event.stopPropagation();
                      onReveal(item);
                    }}
                    disabled={!actions.canReveal || busy("reveal")}
                  >
                    <LibraryActionIcon kind="reveal" />
                  </button>
                  <button
                    className="icon-button tooltip-button"
                    aria-label="Close library"
                    data-tooltip={busy("close") ? "Closing..." : "Close"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onCloseLibrary(item);
                    }}
                    disabled={!actions.canClose || busy("close")}
                  >
                    <LibraryActionIcon kind={busy("close") ? "loading" : "close"} />
                  </button>
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function LibraryActionIcon({ kind }: { kind: "rename" | "export" | "reveal" | "close" | "loading" }) {
  if (kind === "loading") {
    return (
      <svg className="button-icon spinner-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 3a9 9 0 1 1-8.2 5.3" />
      </svg>
    );
  }

  return (
    <svg className="button-icon" viewBox="0 0 24 24" aria-hidden="true">
      {kind === "rename" && (
        <>
          <path d="M4 20h4l11-11a2.8 2.8 0 0 0-4-4L4 16v4Z" />
          <path d="m13.5 6.5 4 4" />
        </>
      )}
      {kind === "export" && (
        <>
          <path d="M12 3v11" />
          <path d="m8 10 4 4 4-4" />
          <path d="M5 17v3h14v-3" />
        </>
      )}
      {kind === "reveal" && (
        <>
          <path d="M3 7h7l2 2h9v10H3V7Z" />
          <path d="M15 13h4" />
          <path d="m17 11 2 2-2 2" />
        </>
      )}
      {kind === "close" && (
        <>
          <path d="M6 6l12 12" />
          <path d="M18 6 6 18" />
        </>
      )}
    </svg>
  );
}

function SettingsProvidersView({
  providerHealth,
  daemonOnline,
  libraryStatus,
}: {
  providerHealth: ProviderHealth[];
  daemonOnline: boolean;
  libraryStatus: LibraryStatus | null;
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>Providers & Diagnostics</h3>
          <p>
            Daemon {daemonOnline ? "online" : "offline"} · Integrity{" "}
            {libraryStatus?.integrityStatus ?? "unknown"}
          </p>
        </div>
        <span className={daemonOnline ? "status completed" : "status failed"}>
          {daemonOnline ? "online" : "offline"}
        </span>
      </div>
      <div className="provider-diagnostics-grid">
        {providerHealth.map((provider) => (
          <div className="provider-diagnostic-card" key={provider.provider}>
            <div className="panel-header">
              <div>
                <h4>{provider.displayName}</h4>
                <p>{provider.provider}</p>
              </div>
              <span className={`status ${provider.availability === "available" ? "completed" : "queued"}`}>
                {provider.availability}
              </span>
            </div>
            <div className="meta-grid">
              <span>Credentials</span>
              <strong>{provider.credentialState}</strong>
              <span>Capabilities</span>
              <strong>{provider.supportedOperations.join(", ") || "none"}</strong>
            </div>
            {provider.recoverableError && <p className="error-text">{provider.recoverableError}</p>}
          </div>
        ))}
      </div>
    </div>
  );
}

function SettingsUpdatesView({
  updateState,
  onCheckUpdate,
  onInstallUpdate,
  onRestartApp,
}: {
  updateState: UpdateState;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
}) {
  const update = updateState.availableUpdate;
  const busy = updateState.checking || updateState.installing;
  const statusText =
    updateState.status === "checking"
      ? "Checking"
      : updateState.status === "available"
        ? "Update available"
        : updateState.status === "installing"
          ? "Installing"
          : updateState.status === "pendingRestart"
            ? "Restart required"
            : updateState.status === "error"
              ? "Needs attention"
              : updateState.status === "upToDate"
                ? "Up to date"
                : "Idle";

  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>App Updates</h3>
          <p>
            Current version {updateState.currentVersion}
            {updateState.lastCheckedAt ? ` · checked ${displayDate(updateState.lastCheckedAt)}` : ""}
          </p>
        </div>
        <span className={`status ${updateState.status === "error" ? "failed" : updateState.status === "available" ? "queued" : "completed"}`}>
          {statusText}
        </span>
      </div>
      <div className="updates-panel">
        <div className="meta-grid">
          <span>Current</span>
          <strong>{updateState.currentVersion}</strong>
          <span>Latest</span>
          <strong>{update?.version ?? (updateState.status === "upToDate" ? updateState.currentVersion : "unknown")}</strong>
          <span>Last checked</span>
          <strong>{updateState.lastCheckedAt ? displayDate(updateState.lastCheckedAt) : "never"}</strong>
        </div>
        {update?.body && (
          <div className="update-notes">
            <h4>Release Notes</h4>
            <p>{update.body}</p>
          </div>
        )}
        {updateState.error && <p className="error-text">{updateState.error}</p>}
        <div className="row-actions">
          <button onClick={onCheckUpdate} disabled={busy}>
            {updateState.checking ? "Checking..." : "Check for Updates"}
          </button>
          <button onClick={onInstallUpdate} disabled={busy || !update || updateState.pendingRestart}>
            {updateState.installing ? "Installing..." : "Download and Install"}
          </button>
          <button onClick={onRestartApp} disabled={!updateState.pendingRestart}>
            Restart
          </button>
        </div>
        <p className="settings-note">
          Updates are verified with the Tauri updater public key. Ad-hoc macOS signing is not Apple notarization.
        </p>
      </div>
    </div>
  );
}

function SettingsLogsView({
  logs,
  logsLoading,
  selectedLogPath,
  selectedLogContent,
  logContentLoading,
  onRefreshLogs,
  onSelectLog,
}: {
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
}) {
  return (
    <div className="settings-section settings-logs-panel">
      <div className="panel-header">
        <div>
          <h3>Logs</h3>
          <p>{logsLoading ? "Loading logs..." : `${logs.length} recent log${logs.length === 1 ? "" : "s"}`}</p>
        </div>
        <button onClick={onRefreshLogs} disabled={logsLoading}>
          {logsLoading ? "Refreshing..." : "Refresh"}
        </button>
      </div>
      {logs.length === 0 ? (
        <div className="empty-state compact">No app logs found.</div>
      ) : (
        <div className="logs-browser">
          <div className="logs-list">
            {logs.map((log) => (
              <button
                key={log.path}
                className={log.path === selectedLogPath ? "log-list-item selected" : "log-list-item"}
                onClick={() => onSelectLog(log.path)}
              >
                <span className="log-list-heading">
                  <strong>{log.kind}</strong>
                  <span>{formatBytes(log.sizeBytes)}</span>
                </span>
                <span className="log-list-meta">
                  <span>{displayDate(log.modifiedAt)}</span>
                </span>
              </button>
            ))}
          </div>
          <div className="log-preview">
            {logContentLoading ? (
              <div className="empty-state compact">Loading log preview...</div>
            ) : selectedLogContent ? (
              <>
                <div className="log-preview-meta">
                  <span className="mono-line">{selectedLogContent.path}</span>
                  {selectedLogContent.truncated && <strong>Truncated</strong>}
                </div>
                <pre>{selectedLogContent.content || "Log is empty."}</pre>
              </>
            ) : (
              <div className="empty-state compact">Select a log to preview.</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
