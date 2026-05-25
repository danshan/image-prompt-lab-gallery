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
import { displayDate, formatBytes } from "./common";
import type {
  Album,
  AlbumListItem,
  AppLog,
  AppLogContent,
  AssetDetail,
  AutomationDaemonStatus,
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
import { libraryMaintenanceActions, type SettingsSection } from "../../workflows/settings";
import type { Dictionary } from "../../i18n/dictionaries";
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
  automationDaemonStatus,
  automationDaemonLoading,
  onRefreshLogs,
  onSelectLog,
  onCheckUpdate,
  onInstallUpdate,
  onRestartApp,
  onRefreshAutomationDaemon,
  onStartAutomationDaemon,
  onStopAutomationDaemon,
  onRestartAutomationDaemon,
  onRepairAutomationDaemon,
  onSetLibraryAutomationEnabled,
  dictionary,
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
  automationDaemonStatus: AutomationDaemonStatus | null;
  automationDaemonLoading: boolean;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
  onRefreshAutomationDaemon: () => void;
  onStartAutomationDaemon: () => void;
  onStopAutomationDaemon: () => void;
  onRestartAutomationDaemon: () => void;
  onRepairAutomationDaemon: () => void;
  onSetLibraryAutomationEnabled: (library: Library, enabled: boolean) => void;
  dictionary: Dictionary;
}) {
  const sections: SettingsSection[] = ["libraries", "automation", "providers", "updates", "logs"];
  return (
    <section className="settings-workspace">
      <nav className="settings-tabs" aria-label={dictionary.views.settings.title}>
        {sections.map((section) => (
          <button
            key={section}
            className={section === activeSection ? "active" : ""}
            type="button"
            onClick={() => onSectionChange(section)}
          >
            {section === "libraries"
              ? dictionary.workflow.libraries
              : section === "automation"
                ? dictionary.workflow.automation
                : section === "providers"
                  ? dictionary.workflow.providers
                  : section === "updates"
                    ? dictionary.workflow.appUpdates
                    : dictionary.workflow.logs}
          </button>
        ))}
      </nav>
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
          dictionary={dictionary}
        />
      ) : activeSection === "automation" ? (
        <SettingsAutomationView
          libraries={libraries}
          automationDaemonStatus={automationDaemonStatus}
          automationDaemonLoading={automationDaemonLoading}
          onRefreshAutomationDaemon={onRefreshAutomationDaemon}
          onStartAutomationDaemon={onStartAutomationDaemon}
          onStopAutomationDaemon={onStopAutomationDaemon}
          onRestartAutomationDaemon={onRestartAutomationDaemon}
          onRepairAutomationDaemon={onRepairAutomationDaemon}
          onSetLibraryAutomationEnabled={onSetLibraryAutomationEnabled}
          dictionary={dictionary}
        />
      ) : activeSection === "providers" ? (
        <SettingsProvidersView
          providerHealth={providerHealth}
          daemonOnline={daemonOnline}
          libraryStatus={libraryStatus}
          dictionary={dictionary}
        />
      ) : activeSection === "updates" ? (
        <SettingsUpdatesView
          updateState={updateState}
          onCheckUpdate={onCheckUpdate}
          onInstallUpdate={onInstallUpdate}
          onRestartApp={onRestartApp}
          dictionary={dictionary}
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
          dictionary={dictionary}
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
  dictionary,
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
  dictionary: Dictionary;
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.libraries}</h3>
          <p>
            {libraries.length} {dictionary.workflow.registered}{library ? `, ${dictionary.workflow.current}: ${library.name}` : ""}
          </p>
        </div>
        <div className="row-actions">
          <button onClick={onOpenExisting}>{dictionary.workflow.openExistingLibrary}</button>
          <button onClick={onImportZip} disabled={pendingLibraryActions.includes("import")}>
            {pendingLibraryActions.includes("import") ? dictionary.workflow.importing : dictionary.workflow.importZip}
          </button>
        </div>
      </div>
      <div className="library-create-strip">
        <div className="library-section-heading">
          <h4>{dictionary.workflow.createLibrary}</h4>
          <p>{dictionary.workflow.createLibraryHint}</p>
        </div>
        <div className="library-create-controls">
          <label>
            <span>{dictionary.workflow.libraryName}</span>
            <input value={libraryName} onChange={(event) => onLibraryNameChange(event.target.value)} />
          </label>
          <label>
            <span>{dictionary.workflow.folderName}</span>
            <input value={libraryFolderName} onChange={(event) => onLibraryFolderNameChange(event.target.value)} />
          </label>
          <button onClick={onCreate}>{dictionary.workflow.createWithEllipsis}</button>
        </div>
      </div>
      <div className="library-section-heading library-list-heading">
        <h4>{dictionary.workflow.registeredLibraries}</h4>
        <p>{dictionary.workflow.registeredLibrariesHint}</p>
      </div>
      {libraries.length === 0 ? (
        <div className="empty-state compact">{dictionary.workflow.noLibraryRegistered}</div>
      ) : (
        <div className="library-table" role="table" aria-label={dictionary.workflow.registeredLibrariesAria}>
          <div className="library-table-row header" role="row">
            <span>{dictionary.workflow.name}</span>
            <span>{dictionary.workflow.path}</span>
            <span>{dictionary.workflow.actions}</span>
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
                aria-label={isCurrent ? `${item.name}, ${dictionary.workflow.currentLibraryAria}` : `${dictionary.workflow.switchToLibraryAria} ${item.name}`}
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
                  {isCurrent && <small>{dictionary.workflow.current}</small>}
                  {!actions.canReveal && <small>{dictionary.workflow.missingOnDisk}</small>}
                </span>
                <span className="mono-line" title={item.rootPath}>
                  {item.rootPath}
                </span>
                <span className="row-actions library-row-actions">
                  <button
                    className="icon-button tooltip-button"
                    aria-label={dictionary.workflow.rename}
                    data-tooltip={busy("rename") ? dictionary.workflow.renaming : dictionary.workflow.rename}
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
                    aria-label={dictionary.workflow.exportZip}
                    data-tooltip={busy("export") ? dictionary.workflow.exporting : dictionary.workflow.exportZip}
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
                    aria-label={dictionary.workflow.revealInFinder}
                    data-tooltip={dictionary.workflow.revealInFinder}
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
                    aria-label={dictionary.workflow.close}
                    data-tooltip={busy("close") ? dictionary.workflow.closing : dictionary.workflow.close}
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

function SettingsAutomationView({
  libraries,
  automationDaemonStatus,
  automationDaemonLoading,
  onRefreshAutomationDaemon,
  onStartAutomationDaemon,
  onStopAutomationDaemon,
  onRestartAutomationDaemon,
  onRepairAutomationDaemon,
  onSetLibraryAutomationEnabled,
  dictionary,
}: {
  libraries: Library[];
  automationDaemonStatus: AutomationDaemonStatus | null;
  automationDaemonLoading: boolean;
  onRefreshAutomationDaemon: () => void;
  onStartAutomationDaemon: () => void;
  onStopAutomationDaemon: () => void;
  onRestartAutomationDaemon: () => void;
  onRepairAutomationDaemon: () => void;
  onSetLibraryAutomationEnabled: (library: Library, enabled: boolean) => void;
  dictionary: Dictionary;
}) {
  const enabledLibraries = libraries.filter((item) => item.automationEnabled).length;
  const daemonEnabled = automationDaemonStatus?.enabled ?? false;
  const daemonHealthy = automationDaemonStatus?.healthy ?? false;
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.automation}</h3>
          <p>
            {enabledLibraries} {enabledLibraries === 1 ? dictionary.workflow.automationLibrarySingular : dictionary.workflow.automationLibraryPlural}
          </p>
        </div>
        <span className={`status ${daemonHealthy ? "completed" : daemonEnabled ? "queued" : "failed"}`}>
          {daemonHealthy ? dictionary.workflow.online : daemonEnabled ? dictionary.workflow.enabled : dictionary.workflow.disabled}
        </span>
      </div>
      <div className="automation-panel">
        <div className="automation-status-grid">
          <div className="meta-grid">
            <span>{dictionary.workflow.daemon}</span>
            <strong>{daemonEnabled ? dictionary.workflow.enabled : dictionary.workflow.disabled}</strong>
            <span>{dictionary.workflow.health}</span>
            <strong>{daemonHealthy ? dictionary.workflow.online : dictionary.workflow.offline}</strong>
            <span>{dictionary.workflow.runtime}</span>
            <strong className="mono-line">{automationDaemonStatus?.runtimePath ?? "-"}</strong>
            <span>{dictionary.workflow.launchAgent}</span>
            <strong className="mono-line">{automationDaemonStatus?.launchAgentPath ?? "-"}</strong>
          </div>
          {automationDaemonStatus?.recoverableError && (
            <p className="error-text">{automationDaemonStatus.recoverableError}</p>
          )}
          <div className="row-actions">
            <button onClick={onRefreshAutomationDaemon} disabled={automationDaemonLoading}>
              {automationDaemonLoading ? dictionary.workflow.refreshingWithEllipsis : dictionary.workflow.refresh}
            </button>
            <button onClick={onStartAutomationDaemon} disabled={automationDaemonLoading || daemonEnabled}>
              {dictionary.workflow.startDaemon}
            </button>
            <button onClick={onStopAutomationDaemon} disabled={automationDaemonLoading || !daemonEnabled}>
              {dictionary.workflow.stopDaemon}
            </button>
            <button onClick={onRestartAutomationDaemon} disabled={automationDaemonLoading || !daemonEnabled}>
              {dictionary.workflow.restart}
            </button>
            <button onClick={onRepairAutomationDaemon} disabled={automationDaemonLoading}>
              {dictionary.workflow.repair}
            </button>
          </div>
        </div>
        <div className="library-section-heading library-list-heading">
          <h4>{dictionary.workflow.automationLibraries}</h4>
          <p>{dictionary.workflow.automationLibrariesHint}</p>
        </div>
        {libraries.length === 0 ? (
          <div className="empty-state compact">{dictionary.workflow.noLibraryRegistered}</div>
        ) : (
          <div className="library-table automation-library-table" role="table" aria-label={dictionary.workflow.automationLibraries}>
            <div className="library-table-row header" role="row">
              <span>{dictionary.workflow.name}</span>
              <span>{dictionary.workflow.path}</span>
              <span>{dictionary.workflow.automation}</span>
            </div>
            {libraries.map((item) => (
              <div key={item.id} className="library-table-row" role="row">
                <span className="library-row-main">
                  <strong>{item.name}</strong>
                  <small>{item.automationEnabled ? dictionary.workflow.enabled : dictionary.workflow.disabled}</small>
                </span>
                <span className="mono-line" title={item.rootPath}>
                  {item.rootPath}
                </span>
                <span className="row-actions library-row-actions">
                  <label className="toggle-row">
                    <input
                      type="checkbox"
                      checked={Boolean(item.automationEnabled)}
                      onChange={(event) => onSetLibraryAutomationEnabled(item, event.target.checked)}
                    />
                    <span>{item.automationEnabled ? dictionary.workflow.enabled : dictionary.workflow.disabled}</span>
                  </label>
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function SettingsProvidersView({
  providerHealth,
  daemonOnline,
  libraryStatus,
  dictionary,
}: {
  providerHealth: ProviderHealth[];
  daemonOnline: boolean;
  libraryStatus: LibraryStatus | null;
  dictionary: Dictionary;
}) {
  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.providersDiagnostics}</h3>
          <p>
            {dictionary.workflow.daemon} {daemonOnline ? dictionary.workflow.online : dictionary.workflow.offline} · {dictionary.workflow.integrity}{" "}
            {libraryStatus?.integrityStatus ?? dictionary.workflow.unknown}
          </p>
        </div>
        <span className={daemonOnline ? "status completed" : "status failed"}>
          {daemonOnline ? dictionary.workflow.online : dictionary.workflow.offline}
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
              <span>{dictionary.workflow.credentials}</span>
              <strong>{provider.credentialState}</strong>
              <span>{dictionary.workflow.capabilities}</span>
              <strong>{provider.supportedOperations.join(", ") || dictionary.workflow.none}</strong>
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
  dictionary,
}: {
  updateState: UpdateState;
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
  onRestartApp: () => void;
  dictionary: Dictionary;
}) {
  const update = updateState.availableUpdate;
  const busy = updateState.checking || updateState.installing;
  const statusText =
    updateState.status === "checking"
      ? dictionary.workflow.checking
      : updateState.status === "available"
        ? dictionary.workflow.updateAvailable
        : updateState.status === "installing"
          ? dictionary.workflow.installing
          : updateState.status === "pendingRestart"
            ? dictionary.workflow.restartRequired
            : updateState.status === "error"
              ? dictionary.workflow.needsAttention
              : updateState.status === "upToDate"
                ? dictionary.workflow.upToDate
                : dictionary.workflow.idle;

  return (
    <div className="settings-section">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.appUpdates}</h3>
          <p>
            {dictionary.workflow.currentVersion} {updateState.currentVersion}
            {updateState.lastCheckedAt ? ` · ${dictionary.workflow.checked} ${displayDate(updateState.lastCheckedAt)}` : ""}
          </p>
        </div>
        <span className={`status ${updateState.status === "error" ? "failed" : updateState.status === "available" ? "queued" : "completed"}`}>
          {statusText}
        </span>
      </div>
      <div className="updates-panel">
        <div className="meta-grid">
          <span>{dictionary.workflow.current}</span>
          <strong>{updateState.currentVersion}</strong>
          <span>{dictionary.workflow.latest}</span>
          <strong>{update?.version ?? (updateState.status === "upToDate" ? updateState.currentVersion : dictionary.workflow.unknown)}</strong>
          <span>{dictionary.workflow.lastChecked}</span>
          <strong>{updateState.lastCheckedAt ? displayDate(updateState.lastCheckedAt) : dictionary.workflow.never}</strong>
        </div>
        {update?.body && (
          <div className="update-notes">
            <h4>{dictionary.workflow.releaseNotes}</h4>
            <p>{update.body}</p>
          </div>
        )}
        {updateState.error && <p className="error-text">{updateState.error}</p>}
        <div className="row-actions">
          <button onClick={onCheckUpdate} disabled={busy}>
            {updateState.checking ? dictionary.workflow.checkingWithEllipsis : dictionary.workflow.checkForUpdates}
          </button>
          <button onClick={onInstallUpdate} disabled={busy || !update || updateState.pendingRestart}>
            {updateState.installing ? dictionary.workflow.installingWithEllipsis : dictionary.workflow.downloadAndInstall}
          </button>
          <button onClick={onRestartApp} disabled={!updateState.pendingRestart}>
            {dictionary.workflow.restart}
          </button>
        </div>
        <p className="settings-note">
          {dictionary.workflow.updateVerificationNote}
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
  dictionary,
}: {
  logs: AppLog[];
  logsLoading: boolean;
  selectedLogPath: string | null;
  selectedLogContent: AppLogContent | null;
  logContentLoading: boolean;
  onRefreshLogs: () => void;
  onSelectLog: (path: string) => void;
  dictionary: Dictionary;
}) {
  return (
    <div className="settings-section settings-logs-panel">
      <div className="panel-header">
        <div>
          <h3>{dictionary.workflow.logs}</h3>
          <p>{logsLoading ? dictionary.workflow.loadingLogs : `${logs.length} ${logs.length === 1 ? dictionary.workflow.recentLogSingular : dictionary.workflow.recentLogPlural}`}</p>
        </div>
        <button onClick={onRefreshLogs} disabled={logsLoading}>
          {logsLoading ? dictionary.workflow.refreshingWithEllipsis : dictionary.workflow.refresh}
        </button>
      </div>
      {logs.length === 0 ? (
        <div className="empty-state compact">{dictionary.workflow.noAppLogsFound}</div>
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
              <div className="empty-state compact">{dictionary.workflow.loadingLogPreview}</div>
            ) : selectedLogContent ? (
              <>
                <div className="log-preview-meta">
                  <span className="mono-line">{selectedLogContent.path}</span>
                  {selectedLogContent.truncated && <strong>{dictionary.workflow.truncated}</strong>}
                </div>
                <pre>{selectedLogContent.content || dictionary.workflow.logIsEmpty}</pre>
              </>
            ) : (
              <div className="empty-state compact">{dictionary.workflow.selectLogPreview}</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
