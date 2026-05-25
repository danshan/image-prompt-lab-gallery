export type SettingsSection = "libraries" | "archived" | "automation" | "taskQueue" | "providers" | "updates" | "logs";

export const defaultSettingsSection: SettingsSection = "libraries";
export const settingsSections: SettingsSection[] = ["libraries", "archived", "automation", "taskQueue", "providers", "updates", "logs"];

export function libraryPathExists(rootPath: string, missingPaths: string[]): boolean {
  return !missingPaths.includes(rootPath);
}

export function libraryMaintenanceActions(rootPath: string, missingPaths: string[]) {
  const exists = libraryPathExists(rootPath, missingPaths);
  return {
    canClose: true,
    canExport: exists,
    canReveal: exists,
  };
}
