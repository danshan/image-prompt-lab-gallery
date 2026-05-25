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

export function CommandBar({
  dictionary,
  locale,
  theme,
  library,
  status,
  assetCount,
  reviewCount,
  queueCount,
  runningTaskCount,
  failedTaskCount,
  onGenerate,
  onThemeToggle,
  onLocaleToggle,
  onViewChange,
}: {
  dictionary: Dictionary;
  locale: Locale;
  theme: ThemePreference;
  library: Library | null;
  status: string;
  assetCount: number;
  reviewCount: number;
  queueCount: number;
  runningTaskCount: number;
  failedTaskCount: number;
  onGenerate: () => void;
  onThemeToggle: () => void;
  onLocaleToggle: () => void;
  onViewChange: (view: View) => void;
}) {
  return (
    <header className="command-bar">
      <div className="command-brand">
        <span className="app-mark">IP</span>
        <span className="command-brand-copy">
          <strong>{dictionary.appName}</strong>
          <small title={library?.rootPath}>{library?.name ?? dictionary.noLibrary}</small>
        </span>
      </div>
      <label className="command-search">
        <Icon name="search" />
        <input aria-label={dictionary.commandSearch} placeholder={dictionary.commandSearch} />
      </label>
      <div className="command-stats" aria-label={dictionary.overview.label}>
        <span><strong>{assetCount}</strong> {dictionary.overview.assets}</span>
        <span><strong>{reviewCount}</strong> {dictionary.review}</span>
        <span><strong>{queueCount}</strong> {dictionary.queue}</span>
      </div>
      <div className="command-actions">
        <button className="primary-button command-generate" aria-label={dictionary.generate} title={dictionary.generate} onClick={onGenerate}>
          <Icon name="spark" />
        </button>
        <button className="command-indicator" aria-label={dictionary.review} title={dictionary.review} onClick={() => onViewChange("review")}>
          <Icon name="review" />
          <strong>{reviewCount}</strong>
        </button>
        <button className="command-indicator" aria-label={dictionary.queue} title={dictionary.queue} onClick={() => onViewChange("queue")}>
          <Icon name="queue" />
          <strong>{queueCount}</strong>
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

export function WorkspaceSwitcher({
  activeView,
  dictionary,
  reviewCount,
  queueCount,
  onViewChange,
}: {
  activeView: View;
  dictionary: Dictionary;
  reviewCount: number;
  queueCount: number;
  onViewChange: (view: View) => void;
}) {
  return (
    <nav className="workspace-switcher" aria-label="Workspace">
      {views.map((view) => {
        const count = view === "review" ? reviewCount : view === "queue" ? queueCount : 0;
        return (
          <button
            key={view}
            className={view === activeView ? "workspace-switcher-item active" : "workspace-switcher-item"}
            title={dictionary.views[view].title}
            onClick={() => onViewChange(view)}
          >
            <span className="workspace-switcher-icon">
              <Icon name={viewIcons[view]} />
            </span>
            <span>{dictionary.views[view].title}</span>
            {count > 0 && <strong>{count}</strong>}
          </button>
        );
      })}
    </nav>
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
