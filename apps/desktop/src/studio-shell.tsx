import React from "react";

export function AppShell({
  commandBar,
  workspaceSidebar,
  workspace,
  contextDrawer,
  drawerOpen,
  sidebarCollapsed,
  children,
}: {
  commandBar: React.ReactNode;
  workspaceSidebar: React.ReactNode;
  workspace: React.ReactNode;
  contextDrawer: React.ReactNode;
  drawerOpen: boolean;
  sidebarCollapsed: boolean;
  children?: React.ReactNode;
}) {
  const className = [
    "desktop-app-shell",
    drawerOpen ? "drawer-open" : "",
    sidebarCollapsed ? "sidebar-collapsed" : "sidebar-expanded",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <main className={className}>
      {commandBar}
      {workspaceSidebar}
      <WorkflowSurface>{workspace}</WorkflowSurface>
      {contextDrawer}
      {children}
    </main>
  );
}

export function StudioRail({ children }: { children: React.ReactNode }) {
  return <aside className="studio-rail">{children}</aside>;
}

export function LibraryContextPanel({ children }: { children: React.ReactNode }) {
  return <aside className="library-context-panel">{children}</aside>;
}

export function WorkspaceFrame({ children }: { children: React.ReactNode }) {
  return <section className="workspace">{children}</section>;
}

export function InspectorFrame({ children }: { children: React.ReactNode }) {
  return <div className="inspector-shell">{children}</div>;
}

export function ActivityStrip({ children }: { children: React.ReactNode }) {
  return <div className="activity-strip">{children}</div>;
}

export function WorkflowSurface({ children }: { children: React.ReactNode }) {
  return <section className="workflow-surface">{children}</section>;
}
