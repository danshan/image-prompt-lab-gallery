import React from "react";

export type IconName =
  | "album"
  | "chevronDown"
  | "close"
  | "database"
  | "grid"
  | "image"
  | "fileText"
  | "languages"
  | "list"
  | "menu"
  | "moon"
  | "panelRight"
  | "plus"
  | "queue"
  | "review"
  | "search"
  | "settings"
  | "spark"
  | "sun";

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
    fileText: (
      <>
        <path d="M14 3v5h5" />
        <path d="M6 3h8l5 5v13H6z" />
        <path d="M9 13h6" />
        <path d="M9 17h4" />
      </>
    ),
    image: (
      <>
        <rect x="4" y="5" width="16" height="14" rx="2" />
        <circle cx="9" cy="10" r="1.5" />
        <path d="m5 17 4.5-4.5 3 3 2-2L19 18" />
      </>
    ),
    languages: (
      <>
        <path d="M5 5h8" />
        <path d="M9 3v2" />
        <path d="M6 9c1.5 3 4.5 5 8 5" />
        <path d="M13 5c-.7 4-3 7-7 9" />
        <path d="M14 21 18 11l4 10" />
        <path d="M15.5 17h5" />
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
    moon: (
      <>
        <path d="M20 14.5A7.5 7.5 0 0 1 9.5 4 8 8 0 1 0 20 14.5z" />
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
    spark: (
      <>
        <path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z" />
        <path d="M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z" />
      </>
    ),
    sun: (
      <>
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2" />
        <path d="M12 20v2" />
        <path d="m4.93 4.93 1.41 1.41" />
        <path d="m17.66 17.66 1.41 1.41" />
        <path d="M2 12h2" />
        <path d="M20 12h2" />
        <path d="m6.34 17.66-1.41 1.41" />
        <path d="m19.07 4.93-1.41 1.41" />
      </>
    ),
  };
  return (
    <svg className="button-icon" aria-hidden="true" viewBox="0 0 24 24">
      {paths[name]}
    </svg>
  );
}
