export type ResponsiveMode = "wide" | "compact" | "narrow";
export type DrawerPresentation = "docked" | "overlay" | "bottomSheet";

export function responsiveModeForWidth(width: number): ResponsiveMode {
  if (width >= 1280) {
    return "wide";
  }
  if (width >= 960) {
    return "compact";
  }
  return "narrow";
}

export function drawerPresentationForMode(mode: ResponsiveMode): DrawerPresentation {
  if (mode === "wide") {
    return "docked";
  }
  if (mode === "compact") {
    return "overlay";
  }
  return "bottomSheet";
}

export function closeDrawerForWorkspaceChange(open: boolean): boolean {
  return open ? false : open;
}

export function sidebarCollapsedByDefaultForMode(mode: ResponsiveMode): boolean {
  return mode !== "wide";
}
