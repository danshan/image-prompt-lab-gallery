# Proposal: Route Tauri Albums Through Use Case

## Problem

The Tauri album command module still routes `list_albums` and `create_manual_album` through `desktop_app().library()`, while the rest of album workflows already use `desktop_app().albums()`.

This leaves one desktop runtime adapter with two competing album entrypoints and keeps legacy service access in a user-facing workflow.

## Goals

- Route Tauri `list_albums` and `create_manual_album` through `AlbumUseCase`.
- Add path-scoped album methods to the album application port for desktop commands that operate on a selected library path.
- Preserve Tauri command signatures, response views, SQLite schema, and album behavior.
- Update architecture inventory and specs to remove this remaining Tauri album exception.

## Non-Goals

- Do not redesign Albums UI behavior.
- Do not change album identity, smart album query semantics, or manual album ordering.
- Do not change SQLite schema or resource library layout.

## Impact

- Tauri album commands use one album application owner.
- The SQLite adapter keeps existing implementation behavior behind the repository port.
- Remaining runtime legacy cleanup can focus on metadata commands and broad type imports.
