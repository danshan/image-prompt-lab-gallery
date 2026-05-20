import { convertFileSrc, invoke as tauriInvoke, isTauri as tauriIsTauri } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type { CommandError } from "./types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function errorMessage(error: unknown) {
  if (typeof error === "object" && error) {
    const commandError = error as CommandError;
    if (commandError.message) {
      return commandError.message;
    }
  }
  return String(error);
}

export function convertImagePath(path: string) {
  if (!hasTauriRuntime()) {
    return path;
  }
  return convertFileSrc(path);
}

export async function invokeCommand<T>(command: string, args?: Record<string, unknown>) {
  if (!hasTauriRuntime()) {
    throw new Error("This action requires the Tauri desktop runtime. Start with npm run tauri dev.");
  }
  return tauriInvoke<T>(command, args);
}

export async function pickDirectory(title: string, defaultPath = "") {
  if (!hasTauriRuntime()) {
    return window.prompt(title, defaultPath);
  }
  const selected = await openDialog({
    title,
    directory: true,
    multiple: false,
    defaultPath: defaultPath || undefined,
  });
  return typeof selected === "string" ? selected : null;
}

export async function pickZipFile(title: string) {
  if (!hasTauriRuntime()) {
    return window.prompt(title);
  }
  const selected = await openDialog({
    title,
    multiple: false,
    filters: [{ name: "Library Backup", extensions: ["zip"] }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function pickImageFile(title: string, defaultPath = "") {
  if (!hasTauriRuntime()) {
    return window.prompt(title, defaultPath);
  }
  const selected = await openDialog({
    title,
    multiple: false,
    defaultPath: defaultPath || undefined,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif"],
      },
    ],
  });
  if (Array.isArray(selected)) {
    return selected[0] ?? null;
  }
  return typeof selected === "string" ? selected : null;
}

export async function pickSaveZipPath(title: string, defaultPath: string) {
  if (!hasTauriRuntime()) {
    return window.prompt(title, defaultPath);
  }
  return saveDialog({
    title,
    defaultPath,
    filters: [{ name: "Library Backup", extensions: ["zip"] }],
  });
}

export function hasTauriRuntime() {
  return typeof window !== "undefined" && tauriIsTauri();
}
