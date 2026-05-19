import { LibraryContextPanel, StudioRail } from "./studio-shell";
import { Icon, type IconName } from "./studio-icons";

export type StudioView = "gallery" | "albums" | "review" | "queue" | "settings";

type LibraryNavItem = {
  id: string;
  name: string;
};

type LibraryStatusSummary = {
  storageSizeBytes: number;
  integrityIssueCount: number;
};

export function Sidebar({
  library,
  libraries,
  libraryStatus,
  activeView,
  reviewCount,
  queueCount,
  expanded,
  onExpandedChange,
  onViewChange,
  onLibraryChange,
}: {
  library: LibraryNavItem | null;
  libraries: LibraryNavItem[];
  libraryStatus: LibraryStatusSummary | null;
  activeView: StudioView;
  reviewCount: number;
  queueCount: number;
  expanded: boolean;
  onExpandedChange: (expanded: boolean) => void;
  onViewChange: (view: StudioView) => void;
  onLibraryChange: (libraryId: string) => void;
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
      </LibraryContextPanel>
    </>
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
