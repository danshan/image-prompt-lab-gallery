import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { initialUpdateState } from "../../types";
import {
  errorMessage,
  invokeCommand,
} from "../../tauri-adapter";
import type {
  AppLog,
  AppLogContent,
  UpdateCheck,
  UpdateState,
} from "../../types";

export function useAppOperationsActions({
  runningInTauri,
  logReadRequestRef,
  setAppLogs,
  setLogsLoading,
  setSelectedLogPath,
  setSelectedLogContent,
  setLogContentLoading,
  setUpdateState,
  setStatus,
  setRecoverableError,
}: {
  runningInTauri: boolean;
  logReadRequestRef: MutableRefObject<string | null>;
  setAppLogs: Dispatch<SetStateAction<AppLog[]>>;
  setLogsLoading: Dispatch<SetStateAction<boolean>>;
  setSelectedLogPath: Dispatch<SetStateAction<string | null>>;
  setSelectedLogContent: Dispatch<SetStateAction<AppLogContent | null>>;
  setLogContentLoading: Dispatch<SetStateAction<boolean>>;
  setUpdateState: Dispatch<SetStateAction<UpdateState>>;
  setStatus: Dispatch<SetStateAction<string>>;
  setRecoverableError: Dispatch<SetStateAction<string | null>>;
}) {
  async function refreshAppLogs() {
    setLogsLoading(true);
    try {
      if (!runningInTauri) {
        setAppLogs([]);
        setSelectedLogPath(null);
        setSelectedLogContent(null);
        setRecoverableError(null);
        return;
      }
      const logs = await invokeCommand<AppLog[]>("list_app_logs");
      setAppLogs(logs);
      setSelectedLogPath((current) => {
        const next = current && logs.some((log) => log.path === current) ? current : logs[0]?.path ?? null;
        if (!next) {
          setSelectedLogContent(null);
        } else if (next !== current) {
          void readAppLog(next);
        }
        return next;
      });
      setRecoverableError(null);
    } catch (error) {
      setAppLogs([]);
      setSelectedLogPath(null);
      setSelectedLogContent(null);
      setRecoverableError(errorMessage(error));
    } finally {
      setLogsLoading(false);
    }
  }

  async function readAppLog(path: string) {
    const requestId = crypto.randomUUID();
    logReadRequestRef.current = requestId;
    setSelectedLogPath(path);
    setLogContentLoading(true);
    try {
      if (!runningInTauri) {
        if (logReadRequestRef.current === requestId) {
          setSelectedLogContent(null);
        }
        setRecoverableError(null);
        return;
      }
      const content = await invokeCommand<AppLogContent>("read_app_log", { input: { path } });
      if (logReadRequestRef.current === requestId && content.path === path) {
        setSelectedLogContent(content);
      }
      setRecoverableError(null);
    } catch (error) {
      if (logReadRequestRef.current === requestId) {
        setSelectedLogContent(null);
      }
      setRecoverableError(errorMessage(error));
    } finally {
      if (logReadRequestRef.current === requestId) {
        setLogContentLoading(false);
      }
    }
  }

  async function checkForAppUpdate({ silent = false }: { silent?: boolean } = {}) {
    setUpdateState((current) => ({
      ...current,
      checking: true,
      error: null,
      status: "checking",
    }));
    try {
      if (!runningInTauri) {
        setUpdateState({
          ...initialUpdateState,
          lastCheckedAt: new Date().toISOString(),
          status: "upToDate",
        });
        return;
      }
      const result = await invokeCommand<UpdateCheck>("check_for_update");
      setUpdateState((current) => ({
        ...current,
        currentVersion: result.currentVersion,
        lastCheckedAt: new Date().toISOString(),
        checking: false,
        availableUpdate: result.update,
        error: null,
        status: result.available ? "available" : "upToDate",
      }));
      if (!silent) {
        setStatus(result.available ? `Update ${result.update?.version ?? ""} available` : "App is up to date");
      }
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        checking: false,
        lastCheckedAt: new Date().toISOString(),
        error: errorMessage(error),
        status: "error",
      }));
      if (!silent) {
        setStatus("Update check failed");
      }
    }
  }

  async function installAppUpdate() {
    setUpdateState((current) => ({
      ...current,
      installing: true,
      error: null,
      status: "installing",
    }));
    try {
      const result = await invokeCommand<{ installed: boolean; version: string | null }>("install_update");
      setUpdateState((current) => ({
        ...current,
        installing: false,
        pendingRestart: result.installed,
        availableUpdate: result.installed ? current.availableUpdate : null,
        error: null,
        status: result.installed ? "pendingRestart" : "upToDate",
      }));
      setStatus(result.installed ? `Update ${result.version ?? ""} installed` : "No update available");
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        installing: false,
        error: errorMessage(error),
        status: "error",
      }));
      setStatus("Update install failed");
    }
  }

  async function restartApp() {
    try {
      await invokeCommand<void>("restart_app");
    } catch (error) {
      setUpdateState((current) => ({
        ...current,
        error: errorMessage(error),
        status: "error",
      }));
    }
  }

  return {
    refreshAppLogs,
    readAppLog,
    checkForAppUpdate,
    installAppUpdate,
    restartApp,
  };
}
