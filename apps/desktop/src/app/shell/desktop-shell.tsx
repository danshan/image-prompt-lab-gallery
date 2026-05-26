import React, { useEffect, useRef } from "react";
import { Icon, type IconName } from "../../studio-icons.js";
import type { Dictionary, Locale } from "../i18n/dictionaries.js";
import type { Library, View } from "../types.js";
import type { ThemePreference } from "../design-system/theme.js";

const viewIcons: Record<View, IconName> = {
  gallery: "image",
  albums: "album",
  prompts: "fileText",
  schedules: "queue",
  review: "review",
  queue: "queue",
  settings: "settings",
};

const views: View[] = ["gallery", "albums", "prompts", "schedules", "review", "queue", "settings"];

type SidebarCounts = Partial<Record<View, number>>;

function formatSidebarCount(count: number) {
  if (count > 999) {
    return "999+";
  }
  return count.toString();
}

export function CommandBar({
  dictionary,
  locale,
  theme,
  library,
  libraries,
  status,
  assetCount,
  reviewCount,
  queueCount,
  runningTaskCount,
  failedTaskCount,
  onGenerate,
  onSwitchLibrary,
  onThemeToggle,
  onLocaleToggle,
  onViewChange,
}: {
  dictionary: Dictionary;
  locale: Locale;
  theme: ThemePreference;
  library: Library | null;
  libraries: Library[];
  status: string;
  assetCount: number;
  reviewCount: number;
  queueCount: number;
  runningTaskCount: number;
  failedTaskCount: number;
  onGenerate: () => void;
  onSwitchLibrary: (libraryId: string) => void;
  onThemeToggle: () => void;
  onLocaleToggle: () => void;
  onViewChange: (view: View) => void;
}) {
  return (
    <header className="command-bar">
      <label className="command-scope">
        <span>{dictionary.currentLibrary}</span>
        <select
          aria-label={dictionary.currentLibrary}
          title={library?.rootPath}
          value={library?.id ?? ""}
          onChange={(event) => onSwitchLibrary(event.target.value)}
          disabled={libraries.length === 0}
        >
          <option value="">{libraries.length === 0 ? dictionary.workflow.noLibraryRegistered : dictionary.noLibrary}</option>
          {libraries.map((item) => (
            <option key={item.id} value={item.id}>{item.name}</option>
          ))}
        </select>
      </label>
      <label className="command-search">
        <Icon name="search" />
        <input aria-label={dictionary.commandSearch} placeholder={dictionary.commandSearch} />
      </label>
      <div className="command-stats" aria-label={dictionary.overview.label}>
        <span><strong>{assetCount}</strong> {dictionary.overview.assets}</span>
        <span><strong>{runningTaskCount}</strong> {dictionary.running}</span>
        <span className={failedTaskCount > 0 ? "danger" : ""}><strong>{failedTaskCount}</strong> {dictionary.failed}</span>
      </div>
      <div className="command-actions">
        <button className="primary-button command-generate" aria-label={dictionary.generate} title={dictionary.generate} onClick={onGenerate}>
          <Icon name="spark" />
        </button>
        <button className="icon-button" aria-label={dictionary.themeToggle} title={dictionary.themeToggle} onClick={onThemeToggle}>
          <Icon name={theme === "dark" ? "sun" : "moon"} />
        </button>
        <button className="icon-button" aria-label={dictionary.localeToggle} title={dictionary.localeToggle} onClick={onLocaleToggle}>
          <Icon name="languages" />
          <span className="sr-only">{locale}</span>
        </button>
        <button className="icon-button" aria-label={dictionary.views.settings.title} title={dictionary.views.settings.title} onClick={() => onViewChange("settings")}>
          <Icon name="settings" />
        </button>
      </div>
      <div className="command-status" aria-live="polite">
        <span>{status}</span>
        <span>{runningTaskCount} {dictionary.running}</span>
        <span className={failedTaskCount > 0 ? "danger" : ""}>{failedTaskCount} {dictionary.failed}</span>
      </div>
    </header>
  );
}

export function WorkspaceSidebar({
  activeView,
  collapsed,
  counts,
  dictionary,
  locale,
  theme,
  onCollapsedChange,
  onGenerate,
  onLocaleToggle,
  onThemeToggle,
  onViewChange,
}: {
  activeView: View;
  collapsed: boolean;
  counts: SidebarCounts;
  dictionary: Dictionary;
  locale: Locale;
  theme: ThemePreference;
  onCollapsedChange: (collapsed: boolean) => void;
  onGenerate: () => void;
  onLocaleToggle: () => void;
  onThemeToggle: () => void;
  onViewChange: (view: View) => void;
}) {
  return (
    <aside className="workspace-sidebar" aria-label={dictionary.workflow.workspaceNavigation}>
      <div className="workspace-sidebar-header">
        <span className="app-mark sidebar-mark">IP</span>
        <span className="workspace-sidebar-title">
          <strong>{dictionary.appName}</strong>
          <small>{dictionary.workflow.workspaceNavigation}</small>
        </span>
        <button
          className="icon-button sidebar-collapse-button"
          aria-label={collapsed ? dictionary.workflow.expandSidebar : dictionary.workflow.collapseSidebar}
          title={collapsed ? dictionary.workflow.expandSidebar : dictionary.workflow.collapseSidebar}
          onClick={() => onCollapsedChange(!collapsed)}
          type="button"
        >
          <Icon name="menu" />
        </button>
      </div>
      <button
        className="primary-button sidebar-generate-button"
        aria-label={dictionary.generate}
        title={dictionary.generate}
        type="button"
        onClick={onGenerate}
      >
        <Icon name="spark" />
        <span>{dictionary.generate}</span>
      </button>
      <nav className="workspace-sidebar-nav" aria-label={dictionary.workflow.workspaceNavigation}>
        {views.map((view) => {
          const count = counts[view];
          const countLabel = typeof count === "number" ? formatSidebarCount(count) : null;
          const title = countLabel ? `${dictionary.views[view].title} ${countLabel}` : dictionary.views[view].title;
          return (
            <button
              key={view}
              className={view === activeView ? "workspace-sidebar-item active" : "workspace-sidebar-item"}
              aria-label={title}
              title={title}
              onClick={() => onViewChange(view)}
              type="button"
            >
              <span className="workspace-sidebar-icon">
                <Icon name={viewIcons[view]} />
              </span>
              <span className="workspace-sidebar-label">{dictionary.views[view].title}</span>
              {countLabel && <span className="workspace-sidebar-count-label">{countLabel}</span>}
            </button>
          );
        })}
      </nav>
      <div className="sidebar-global-actions" aria-label={dictionary.workflow.globalActions}>
        <button
          className="sidebar-global-button"
          aria-label={dictionary.themeToggle}
          title={dictionary.themeToggle}
          type="button"
          onClick={onThemeToggle}
        >
          <Icon name={theme === "dark" ? "sun" : "moon"} />
          <span>{dictionary.themeToggle}</span>
        </button>
        <button
          className="sidebar-global-button"
          aria-label={dictionary.localeToggle}
          title={dictionary.localeToggle}
          type="button"
          onClick={onLocaleToggle}
        >
          <Icon name="languages" />
          <span>{dictionary.localeToggle}</span>
          <small>{locale}</small>
        </button>
      </div>
    </aside>
  );
}

export function ContextDrawer({
  open,
  title,
  subtitle,
  closeLabel,
  onClose,
  children,
}: {
  open: boolean;
  title: string;
  subtitle: string;
  closeLabel: string;
  onClose: () => void;
  children: React.ReactNode;
}) {
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!open || typeof document === "undefined") {
      return;
    }
    previousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  }, [open]);

  useEffect(() => {
    if (!open || typeof window === "undefined") {
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
  }, [onClose, open]);

  useEffect(() => {
    if (open || typeof document === "undefined") {
      return;
    }
    const previousFocus = previousFocusRef.current;
    if (previousFocus && document.contains(previousFocus)) {
      previousFocus.focus();
    }
  }, [open]);

  return (
    <>
      <button
        className={open ? "context-drawer-backdrop open" : "context-drawer-backdrop"}
        aria-label={closeLabel}
        type="button"
        onClick={onClose}
      />
      <aside className={open ? "context-drawer open" : "context-drawer"} aria-label={title} aria-hidden={!open}>
        <header className="context-drawer-header">
          <span>
            <strong>{title}</strong>
            <small>{subtitle}</small>
          </span>
          <button className="icon-button" aria-label={closeLabel} onClick={onClose}>
            <Icon name="close" />
          </button>
        </header>
        <div className="context-drawer-body">{children}</div>
      </aside>
    </>
  );
}
