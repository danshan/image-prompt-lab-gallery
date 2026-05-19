import React from "react";

export function AppShell({
  sidebar,
  workspace,
  inspector,
  sidebarExpanded,
  inspectorExpanded,
  activityStrip,
  children,
}: {
  sidebar: React.ReactNode;
  workspace: React.ReactNode;
  inspector: React.ReactNode;
  sidebarExpanded: boolean;
  inspectorExpanded: boolean;
  activityStrip?: React.ReactNode;
  children?: React.ReactNode;
}) {
  return (
    <main className={`workbench${sidebarExpanded ? " sidebar-expanded" : ""}${inspectorExpanded ? " inspector-expanded" : ""}`}>
      <header className="studio-command-bar">
        <div className="studio-brand">
          <strong>Image Prompt Lab</strong>
          <span>Studio Console</span>
        </div>
        <div className="studio-command-meta">
          <span>Local-first library</span>
          <span>Review-gated metadata</span>
          <span>Task-backed generation</span>
        </div>
      </header>
      <div className="studio-sidebar">{sidebar}</div>
      <WorkspaceFrame>{workspace}</WorkspaceFrame>
      <InspectorFrame>{inspector}</InspectorFrame>
      {activityStrip}
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
