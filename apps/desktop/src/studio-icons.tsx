import React from "react";

export type IconName =
  | "album"
  | "chevronDown"
  | "close"
  | "database"
  | "grid"
  | "image"
  | "list"
  | "menu"
  | "panelRight"
  | "plus"
  | "queue"
  | "review"
  | "search"
  | "settings";

export function Icon({ name }: { name: IconName }) {
  const paths: Record<IconName, React.ReactNode> = {
    album: (
      <>
        <rect x="5" y="6" width="12" height="12" rx="2" />
        <path d="M8 3h10a1 1 0 0 1 1 1v11" />
        <path d="m8 15 2.5-3 2 2 1.5-1.5 3 3.5" />
      </>
    ),
    chevronDown: <path d="m6 9 6 6 6-6" />,
    close: (
      <>
        <path d="M18 6 6 18" />
        <path d="m6 6 12 12" />
      </>
    ),
    database: (
      <>
        <ellipse cx="12" cy="5" rx="7" ry="3" />
        <path d="M5 5v6c0 1.7 3.1 3 7 3s7-1.3 7-3V5" />
        <path d="M5 11v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6" />
      </>
    ),
    grid: (
      <>
        <rect x="4" y="4" width="6" height="6" rx="1" />
        <rect x="14" y="4" width="6" height="6" rx="1" />
        <rect x="4" y="14" width="6" height="6" rx="1" />
        <rect x="14" y="14" width="6" height="6" rx="1" />
      </>
    ),
    image: (
      <>
        <rect x="4" y="5" width="16" height="14" rx="2" />
        <circle cx="9" cy="10" r="1.5" />
        <path d="m5 17 4.5-4.5 3 3 2-2L19 18" />
      </>
    ),
    list: (
      <>
        <path d="M8 6h12" />
        <path d="M8 12h12" />
        <path d="M8 18h12" />
        <path d="M4 6h.01" />
        <path d="M4 12h.01" />
        <path d="M4 18h.01" />
      </>
    ),
    menu: (
      <>
        <path d="M4 7h16" />
        <path d="M4 12h16" />
        <path d="M4 17h16" />
      </>
    ),
    panelRight: (
      <>
        <rect x="4" y="5" width="16" height="14" rx="2" />
        <path d="M14 5v14" />
      </>
    ),
    plus: (
      <>
        <path d="M12 5v14" />
        <path d="M5 12h14" />
      </>
    ),
    queue: (
      <>
        <path d="M6 7h7" />
        <path d="M6 12h11" />
        <path d="M6 17h5" />
        <path d="M16 6l3 3-3 3" />
        <path d="M13 9h6" />
      </>
    ),
    review: (
      <>
        <rect x="5" y="4" width="14" height="16" rx="2" />
        <path d="M9 8h6" />
        <path d="M9 12h3" />
        <path d="m9 16 1.5 1.5L15 13" />
      </>
    ),
    search: (
      <>
        <circle cx="11" cy="11" r="7" />
        <path d="m16 16 4 4" />
      </>
    ),
    settings: (
      <>
        <path d="M5 7h14" />
        <path d="M5 12h14" />
        <path d="M5 17h14" />
        <circle cx="9" cy="7" r="2" />
        <circle cx="15" cy="12" r="2" />
        <circle cx="11" cy="17" r="2" />
      </>
    ),
  };
  return (
    <svg className="button-icon" aria-hidden="true" viewBox="0 0 24 24">
      {paths[name]}
    </svg>
  );
}
