import { LibraryContextPanel, StudioRail } from "./studio-shell";
import { Icon, type IconName } from "./studio-icons";

export type StudioView = "gallery" | "albums" | "prompts" | "schedules" | "review" | "queue" | "settings";

type LibraryNavItem = {
  id: string;
  name: string;
};

type LibraryStatusSummary = {
  storageSizeBytes: number;
  integrityIssueCount: number;
};

type AlbumNavItem = {
  id: string;
  name: string;
  kind: "manual" | "smart";
  itemCount?: number | null;
};

type SettingsSectionNav = "libraries" | "archived" | "automation" | "providers" | "updates" | "logs";

export function Sidebar({
  library,
  libraries,
  libraryStatus,
  albums,
  selectedAlbumId,
  albumSearchValue,
  settingsSection,
  activeView,
  reviewCount,
  queueCount,
  expanded,
  onExpandedChange,
  onViewChange,
  onLibraryChange,
  onAlbumSearchChange,
  onCreateAlbumClick,
  onCloseAlbum,
  onOpenAlbum,
  onSettingsSectionChange,
}: {
  library: LibraryNavItem | null;
  libraries: LibraryNavItem[];
  libraryStatus: LibraryStatusSummary | null;
  albums: AlbumNavItem[];
  selectedAlbumId: string | null;
  albumSearchValue: string;
  settingsSection: SettingsSectionNav;
  activeView: StudioView;
  reviewCount: number;
  queueCount: number;
  expanded: boolean;
  onExpandedChange: (expanded: boolean) => void;
  onViewChange: (view: StudioView) => void;
  onLibraryChange: (libraryId: string) => void;
  onAlbumSearchChange: (value: string) => void;
  onCreateAlbumClick: () => void;
  onCloseAlbum: () => void;
  onOpenAlbum: (albumId: string) => void;
  onSettingsSectionChange: (section: SettingsSectionNav) => void;
}) {
  return (
    <>
      <StudioRail>
        <button className="sidebar-toggle" onClick={() => onExpandedChange(!expanded)}>
          <Icon name="menu" />
          <span>{expanded ? "Collapse" : "Menu"}</span>
        </button>
        <nav className="nav">
          <NavButton active={activeView === "gallery"} icon="image" label="Gallery" onClick={() => onViewChange("gallery")} />
          <NavButton active={activeView === "albums"} icon="album" label="Albums" onClick={() => onViewChange("albums")} />
          <NavButton active={activeView === "prompts"} icon="list" label="Prompts" onClick={() => onViewChange("prompts")} />
          <NavButton active={activeView === "schedules"} icon="queue" label="Schedules" onClick={() => onViewChange("schedules")} />
          <NavButton
            active={activeView === "review"}
            icon="review"
            label="Review Inbox"
            count={reviewCount}
            onClick={() => onViewChange("review")}
          />
          <NavButton
            active={activeView === "queue"}
            icon="queue"
            label="Tasks Queue"
            count={queueCount}
            onClick={() => onViewChange("queue")}
          />
          <NavButton active={activeView === "settings"} icon="settings" label="Settings" onClick={() => onViewChange("settings")} />
        </nav>
        <small className="app-version">Image Prompt Lab 1.2.0</small>
      </StudioRail>
      <LibraryContextPanel>
        {activeView === "albums" ? (
          <AlbumsContextPanel
            albums={albums}
            selectedAlbumId={selectedAlbumId}
            searchValue={albumSearchValue}
            onSearchChange={onAlbumSearchChange}
            onCreateAlbumClick={onCreateAlbumClick}
            onCloseAlbum={onCloseAlbum}
            onOpenAlbum={onOpenAlbum}
          />
        ) : activeView === "settings" ? (
          <SettingsContextPanel
            activeSection={settingsSection}
            onSectionChange={onSettingsSectionChange}
          />
        ) : (
          <LibrarySummaryPanel
            library={library}
            libraries={libraries}
            libraryStatus={libraryStatus}
            onLibraryChange={onLibraryChange}
          />
        )}
      </LibraryContextPanel>
    </>
  );
}

function LibrarySummaryPanel({
  library,
  libraries,
  libraryStatus,
  onLibraryChange,
}: {
  library: LibraryNavItem | null;
  libraries: LibraryNavItem[];
  libraryStatus: LibraryStatusSummary | null;
  onLibraryChange: (libraryId: string) => void;
}) {
  return (
    <>
      <label className="library-card library-selector-card">
        <span className="database-icon" aria-hidden="true">
          <Icon name="database" />
        </span>
        <span>
          <strong>{library?.name ?? "No library"}</strong>
          <small>Library</small>
        </span>
        <select
          className="library-picker"
          aria-label="Switch library"
          value={library?.id ?? ""}
          onChange={(event) => onLibraryChange(event.target.value)}
        >
          {libraries.length === 0 ? (
            <option value="">No library registered</option>
          ) : (
            libraries.map((item) => (
              <option key={item.id} value={item.id}>
                {item.name}
              </option>
            ))
          )}
        </select>
        <span className="library-chevron" aria-hidden="true">
          <Icon name="chevronDown" />
        </span>
      </label>
      <section className="library-status">
        <div>
          <span>Library Status</span>
          <strong className="healthy">Healthy</strong>
        </div>
        <div>
          <span>Storage</span>
          <span>{formatBytes(libraryStatus?.storageSizeBytes ?? null)}</span>
        </div>
        <div>
          <span>Integrity Check</span>
          <strong className={libraryStatus?.integrityIssueCount ? "warning" : "healthy"}>
            {libraryStatus?.integrityIssueCount ? `${libraryStatus.integrityIssueCount} issue(s)` : "All good"}
          </strong>
        </div>
        <button>Run Integrity Check</button>
      </section>
    </>
  );
}

function AlbumsContextPanel({
  albums,
  selectedAlbumId,
  searchValue,
  onSearchChange,
  onCreateAlbumClick,
  onCloseAlbum,
  onOpenAlbum,
}: {
  albums: AlbumNavItem[];
  selectedAlbumId: string | null;
  searchValue: string;
  onSearchChange: (value: string) => void;
  onCreateAlbumClick: () => void;
  onCloseAlbum: () => void;
  onOpenAlbum: (albumId: string) => void;
}) {
  const needle = searchValue.trim().toLocaleLowerCase();
  const visibleAlbums = needle
    ? albums.filter((album) => album.name.toLocaleLowerCase().includes(needle))
    : albums;
  return (
    <section className="context-section">
      <div className="context-panel-header">
        <div>
          <strong>Albums</strong>
          <small>{albums.length} total</small>
        </div>
        <button className="icon-button" aria-label="Create album" onClick={onCreateAlbumClick}>
          <Icon name="plus" />
        </button>
      </div>
      <input
        className="context-search"
        value={searchValue}
        onChange={(event) => onSearchChange(event.target.value)}
        placeholder="Search albums"
      />
      <div className="context-nav-list">
        <button
          className={selectedAlbumId === null ? "context-nav-item active" : "context-nav-item"}
          onClick={onCloseAlbum}
        >
          <span>
            <strong>All albums</strong>
            <small>Overview</small>
          </span>
          <span>{albums.length}</span>
        </button>
        {visibleAlbums.map((album) => (
          <button
            key={album.id}
            className={album.id === selectedAlbumId ? "context-nav-item active" : "context-nav-item"}
            onClick={() => onOpenAlbum(album.id)}
          >
            <span>
              <strong>{album.name}</strong>
              <small>{album.kind}</small>
            </span>
            <span>{album.itemCount ?? "-"}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

function SettingsContextPanel({
  activeSection,
  onSectionChange,
}: {
  activeSection: SettingsSectionNav;
  onSectionChange: (section: SettingsSectionNav) => void;
}) {
  const sections: Array<{ id: SettingsSectionNav; label: string }> = [
    { id: "libraries", label: "Libraries" },
    { id: "archived", label: "Archived" },
    { id: "automation", label: "Automation" },
    { id: "providers", label: "Providers" },
    { id: "updates", label: "Updates" },
    { id: "logs", label: "Logs" },
  ];
  return (
    <section className="context-section">
      <div className="context-panel-header">
        <div>
          <strong>Settings</strong>
          <small>Sections</small>
        </div>
      </div>
      <div className="context-nav-list">
        {sections.map((section) => (
          <button
            key={section.id}
            className={section.id === activeSection ? "context-nav-item active" : "context-nav-item"}
            onClick={() => onSectionChange(section.id)}
          >
            <span>{section.label}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

function NavButton({
  active,
  icon,
  label,
  count,
  onClick,
}: {
  active: boolean;
  icon: IconName;
  label: string;
  count?: number;
  onClick: () => void;
}) {
  return (
    <button className={active ? "nav-button active" : "nav-button"} title={label} onClick={onClick}>
      <span className="nav-icon">
        <Icon name={icon} />
      </span>
      <span className="nav-label">{label}</span>
      {typeof count === "number" && count > 0 && <strong>{count}</strong>}
    </button>
  );
}

function formatBytes(value: number | null) {
  if (!value) {
    return "-";
  }
  if (value >= 1024 * 1024 * 1024) {
    return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
  if (value > 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
  }
  return `${Math.round(value / 1024)} KB`;
}
