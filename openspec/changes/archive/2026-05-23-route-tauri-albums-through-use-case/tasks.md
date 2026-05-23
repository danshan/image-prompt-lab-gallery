# Tasks: Route Tauri Albums Through Use Case

## 1. OpenSpec Setup

- [x] 1.1 Create proposal, design, tasks, and delta specs.
- [x] 1.2 Confirm Tauri command and persistence contracts remain stable.

## 2. Album Application Boundary

- [x] 2.1 Add path-scoped album list and manual-create methods to `AlbumUseCase`.
- [x] 2.2 Add matching `AlbumRepository` port methods.
- [x] 2.3 Implement the SQLite adapter using existing album behavior.

## 3. Tauri Migration

- [x] 3.1 Route `list_albums` through `desktop_app().albums()`.
- [x] 3.2 Route `create_manual_album` through `desktop_app().albums()`.

## 4. Documentation

- [x] 4.1 Update architecture inventory for Tauri album command ownership.
- [x] 4.2 Update specs with the album runtime owner boundary.

## 5. Verification

- [x] 5.1 Run desktop Rust tests.
- [x] 5.2 Run architecture guardrails.
- [x] 5.3 Run OpenSpec validation.
